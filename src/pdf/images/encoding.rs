mod palette;
mod png;

#[cfg(test)]
pub(super) use png::base64;
pub(super) use png::data_uri;

use super::super::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfValue};
use palette::indexed_palette;

pub(super) fn png_from_raw_image(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
    data: &[u8],
) -> Result<Vec<u8>, String> {
    reject_decode_predictors(dictionary)?;
    let width = image_dimension(dictionary, "Width")?;
    let height = image_dimension(dictionary, "Height")?;
    let bits_per_component = dictionary.integer("BitsPerComponent").unwrap_or(8);
    let prepared = prepare_image(objects, dictionary, data, width, height, bits_per_component)?;
    let row_len = width
        .checked_mul(prepared.color.channels())
        .ok_or_else(|| "image dimensions are too large".to_string())?;

    let mut scanlines = Vec::with_capacity(prepared.pixels.len() + height);
    for row in prepared.pixels.chunks(row_len) {
        scanlines.push(0);
        scanlines.extend_from_slice(row);
    }

    png::encode_png(width, height, prepared.color.png_type(), &scanlines)
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

    png::encode_png(width, height, 6, &scanlines)
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
pub(in crate::pdf::images::encoding) enum PngColor {
    Grayscale,
    Rgb,
}

impl PngColor {
    pub(in crate::pdf::images::encoding) fn channels(self) -> usize {
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

#[derive(Debug)]
struct PreparedImage {
    color: PngColor,
    pixels: Vec<u8>,
}

fn prepare_image(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
    data: &[u8],
    width: usize,
    height: usize,
    bits_per_component: i64,
) -> Result<PreparedImage, String> {
    let bits = usize::try_from(bits_per_component)
        .ok()
        .filter(|bits| matches!(*bits, 1 | 2 | 4 | 8))
        .ok_or_else(|| format!("unsupported image bit depth {bits_per_component}"))?;

    if let Some(palette) = indexed_palette(objects, dictionary)? {
        let indices = unpack_component_samples(data, width, height, 1, bits)?;
        let channels = palette.color.channels();
        let mut pixels = Vec::with_capacity(indices.len() * channels);
        for index in indices {
            let offset = usize::from(index)
                .checked_mul(channels)
                .ok_or_else(|| "indexed image palette offset is too large".to_string())?;
            let color = palette
                .lookup
                .get(offset..offset + channels)
                .ok_or_else(|| format!("indexed image palette is missing entry {}", index))?;
            pixels.extend_from_slice(color);
        }
        return Ok(PreparedImage {
            color: palette.color,
            pixels,
        });
    }

    let color = png_color(dictionary, data, width, height)?;
    let channels = color.channels();
    let mut pixels = unpack_component_samples(data, width, height, channels, bits)?;
    if bits < 8 {
        let max = ((1u16 << bits) - 1) as u8;
        for sample in &mut pixels {
            *sample = ((*sample as u16 * 255) / u16::from(max)) as u8;
        }
    }
    Ok(PreparedImage { color, pixels })
}

fn unpack_component_samples(
    data: &[u8],
    width: usize,
    height: usize,
    samples_per_pixel: usize,
    bits: usize,
) -> Result<Vec<u8>, String> {
    let samples_per_row = width
        .checked_mul(samples_per_pixel)
        .ok_or_else(|| "image dimensions are too large".to_string())?;
    let row_bits = samples_per_row
        .checked_mul(bits)
        .ok_or_else(|| "image dimensions are too large".to_string())?;
    let row_bytes = row_bits.div_ceil(8);
    let expected_len = row_bytes
        .checked_mul(height)
        .ok_or_else(|| "image dimensions are too large".to_string())?;
    if data.len() < expected_len {
        return Err(format!(
            "raw image stream is shorter than expected ({} < {expected_len})",
            data.len()
        ));
    }
    if bits == 8 {
        let pixel_len = samples_per_row
            .checked_mul(height)
            .ok_or_else(|| "image dimensions are too large".to_string())?;
        let mut pixels = Vec::with_capacity(pixel_len);
        for row in data[..expected_len].chunks(row_bytes) {
            pixels.extend_from_slice(&row[..samples_per_row]);
        }
        return Ok(pixels);
    }

    let mask = (1u8 << bits) - 1;
    let mut samples = Vec::with_capacity(samples_per_row * height);
    for row in data[..expected_len].chunks(row_bytes) {
        for index in 0..samples_per_row {
            let bit_index = index * bits;
            let byte = row[bit_index / 8];
            let shift = 8 - bits - (bit_index % 8);
            samples.push((byte >> shift) & mask);
        }
    }
    Ok(samples)
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
