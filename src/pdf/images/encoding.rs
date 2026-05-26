use std::io::Write;

use flate2::{write::ZlibEncoder, Compression};

use super::super::object::{PdfDictionary, PdfDictionaryExt, PdfValue};

pub(super) fn png_from_raw_image(
    dictionary: &PdfDictionary,
    data: &[u8],
) -> Result<Vec<u8>, String> {
    reject_decode_predictors(dictionary)?;
    let width = image_dimension(dictionary, "Width")?;
    let height = image_dimension(dictionary, "Height")?;
    let bits_per_component = dictionary.integer("BitsPerComponent").unwrap_or(8);
    if bits_per_component != 8 {
        return Err(format!("unsupported image bit depth {bits_per_component}"));
    }
    let color = png_color(dictionary, data, width, height)?;
    let channels = color.channels();
    let row_len = width
        .checked_mul(channels)
        .ok_or_else(|| "image dimensions are too large".to_string())?;
    let expected_len = row_len
        .checked_mul(height)
        .ok_or_else(|| "image dimensions are too large".to_string())?;
    if data.len() < expected_len {
        return Err(format!(
            "raw image stream is shorter than expected ({} < {expected_len})",
            data.len()
        ));
    }

    let mut scanlines = Vec::with_capacity(expected_len + height);
    for row in data[..expected_len].chunks(row_len) {
        scanlines.push(0);
        scanlines.extend_from_slice(row);
    }

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8);
    ihdr.push(color.png_type());
    ihdr.extend_from_slice(&[0, 0, 0]);
    push_png_chunk(&mut png, b"IHDR", &ihdr);
    push_png_chunk(&mut png, b"IDAT", &zlib_compress(&scanlines)?);
    push_png_chunk(&mut png, b"IEND", &[]);
    Ok(png)
}

pub(super) fn png_alpha_from_gray_mask(
    dictionary: &PdfDictionary,
    data: &[u8],
) -> Result<Vec<u8>, String> {
    reject_decode_predictors(dictionary)?;
    let width = image_dimension(dictionary, "Width")?;
    let height = image_dimension(dictionary, "Height")?;
    let bits_per_component = dictionary.integer("BitsPerComponent").unwrap_or(8);
    if bits_per_component != 8 {
        return Err(format!(
            "unsupported image mask bit depth {bits_per_component}"
        ));
    }
    if !matches!(
        dictionary.get("ColorSpace"),
        Some(PdfValue::Name(name)) if name == "DeviceGray" || name == "G"
    ) {
        return Err("image mask is not DeviceGray".to_string());
    }
    let expected_len = width
        .checked_mul(height)
        .ok_or_else(|| "image mask dimensions are too large".to_string())?;
    if data.len() < expected_len {
        return Err(format!(
            "image mask stream is shorter than expected ({} < {expected_len})",
            data.len()
        ));
    }

    let row_len = width * 4;
    let mut scanlines = Vec::with_capacity((row_len + 1) * height);
    for row in data[..expected_len].chunks(width) {
        scanlines.push(0);
        for alpha in row {
            scanlines.extend_from_slice(&[0xff, 0xff, 0xff, *alpha]);
        }
    }

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.extend_from_slice(&[0, 0, 0]);
    push_png_chunk(&mut png, b"IHDR", &ihdr);
    push_png_chunk(&mut png, b"IDAT", &zlib_compress(&scanlines)?);
    push_png_chunk(&mut png, b"IEND", &[]);
    Ok(png)
}

fn reject_decode_predictors(dictionary: &PdfDictionary) -> Result<(), String> {
    let Some(decode_parms) = dictionary.get("DecodeParms") else {
        return Ok(());
    };
    match decode_parms {
        PdfValue::Dictionary(values) => match values.integer("Predictor") {
            None | Some(1) => Ok(()),
            Some(predictor) => Err(format!("unsupported image predictor {predictor}")),
        },
        PdfValue::Array(values) if values.is_empty() => Ok(()),
        _ => Err("unsupported image decode parameters".to_string()),
    }
}

fn image_dimension(dictionary: &PdfDictionary, key: &str) -> Result<usize, String> {
    let value = dictionary
        .integer(key)
        .ok_or_else(|| format!("image is missing {key}"))?;
    usize::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("invalid image {key} {value}"))
}

#[derive(Debug, Clone, Copy)]
enum PngColor {
    Grayscale,
    Rgb,
}

impl PngColor {
    fn channels(self) -> usize {
        match self {
            Self::Grayscale => 1,
            Self::Rgb => 3,
        }
    }

    fn png_type(self) -> u8 {
        match self {
            Self::Grayscale => 0,
            Self::Rgb => 2,
        }
    }
}

fn png_color(
    dictionary: &PdfDictionary,
    data: &[u8],
    width: usize,
    height: usize,
) -> Result<PngColor, String> {
    match dictionary.get("ColorSpace") {
        Some(PdfValue::Name(name)) if name == "DeviceGray" || name == "G" => {
            Ok(PngColor::Grayscale)
        }
        Some(PdfValue::Name(name)) if name == "DeviceRGB" || name == "RGB" => Ok(PngColor::Rgb),
        Some(PdfValue::Reference(_)) => infer_png_color(dictionary, data, width, height),
        Some(PdfValue::Name(name)) => Err(format!("unsupported image color space {name}")),
        Some(PdfValue::Array(values)) => Err(format!(
            "unsupported image color space array with {} entries",
            values.len()
        )),
        _ => infer_png_color(dictionary, data, width, height),
    }
}

fn infer_png_color(
    dictionary: &PdfDictionary,
    data: &[u8],
    width: usize,
    height: usize,
) -> Result<PngColor, String> {
    if let Some(colors) = decode_parameter_colors(dictionary) {
        return match colors {
            1 => Ok(PngColor::Grayscale),
            3 => Ok(PngColor::Rgb),
            _ => Err(format!("unsupported inferred image color count {colors}")),
        };
    }

    let pixels = width
        .checked_mul(height)
        .ok_or_else(|| "image dimensions are too large".to_string())?;
    if pixels > 0 && data.len() == pixels {
        return Ok(PngColor::Grayscale);
    }
    if pixels > 0 && data.len() == pixels * 3 {
        return Ok(PngColor::Rgb);
    }

    Err("image is missing ColorSpace".to_string())
}

fn decode_parameter_colors(dictionary: &PdfDictionary) -> Option<i64> {
    match dictionary.get("DecodeParms")? {
        PdfValue::Dictionary(values) => values.integer("Colors"),
        PdfValue::Array(values) => values.iter().find_map(|value| match value {
            PdfValue::Dictionary(values) => values.integer("Colors"),
            _ => None,
        }),
        _ => None,
    }
}

fn push_png_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let crc = crc32(kind.iter().copied().chain(data.iter().copied()));
    png.extend_from_slice(&crc.to_be_bytes());
}

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|error| format!("could not compress PNG image data ({error})"))?;
    encoder
        .finish()
        .map_err(|error| format!("could not finish PNG image data ({error})"))
}

fn crc32(bytes: impl IntoIterator<Item = u8>) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
pub(super) fn data_uri(media_type: &str, data: &[u8]) -> String {
    format!("data:{media_type};base64,{}", base64(data))
}

pub(super) fn base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[usize::from(a >> 2)] as char);
        output.push(TABLE[usize::from(((a & 0b0000_0011) << 4) | (b >> 4))] as char);
        if chunk.len() > 1 {
            output.push(TABLE[usize::from(((b & 0b0000_1111) << 2) | (c >> 6))] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[usize::from(c & 0b0011_1111)] as char);
        } else {
            output.push('=');
        }
    }
    output
}
