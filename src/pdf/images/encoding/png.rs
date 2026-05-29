use crate::pdf::compression;

pub(in crate::pdf::images::encoding) fn encode_png(
    width: usize,
    height: usize,
    color_type: u8,
    scanlines: &[u8],
) -> Result<Vec<u8>, String> {
    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8);
    ihdr.push(color_type);
    ihdr.extend_from_slice(&[0, 0, 0]);
    push_png_chunk(&mut png, b"IHDR", &ihdr);
    push_png_chunk(&mut png, b"IDAT", &zlib_compress(scanlines)?);
    push_png_chunk(&mut png, b"IEND", &[]);
    Ok(png)
}

fn push_png_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let crc = crc32(kind.iter().copied().chain(data.iter().copied()));
    png.extend_from_slice(&crc.to_be_bytes());
}

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    compression::zlib_encode(data)
        .map_err(|error| format!("could not compress PNG image data ({error})"))
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

pub(in crate::pdf::images) fn data_uri(media_type: &str, data: &[u8]) -> String {
    format!("data:{media_type};base64,{}", base64(data))
}

pub(in crate::pdf::images) fn base64(data: &[u8]) -> String {
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
