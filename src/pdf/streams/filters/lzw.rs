use crate::ConvertError;

pub(super) fn lzw_decode(data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let mut reader = BitReader::new(data);
    let mut table = initial_lzw_table();
    let mut code_size = 9usize;
    let mut previous: Option<Vec<u8>> = None;
    let mut output = Vec::new();

    while let Some(code) = reader.read_bits(code_size) {
        match code {
            256 => {
                table = initial_lzw_table();
                code_size = 9;
                previous = None;
            }
            257 => break,
            _ => {
                let entry = if let Some(entry) = table.get(code as usize).cloned().flatten() {
                    entry
                } else if code as usize == table.len() {
                    let Some(previous) = previous.as_ref() else {
                        return Err(ConvertError::Pdf("invalid LZW stream".to_string()));
                    };
                    let mut entry = previous.clone();
                    entry.push(previous[0]);
                    entry
                } else {
                    return Err(ConvertError::Pdf("invalid LZW code".to_string()));
                };

                output.extend_from_slice(&entry);
                if let Some(previous) = previous.as_ref() {
                    if table.len() < 4096 {
                        let mut next = previous.clone();
                        next.push(entry[0]);
                        table.push(Some(next));
                        if table.len() == (1usize << code_size) && code_size < 12 {
                            code_size += 1;
                        }
                    }
                }
                previous = Some(entry);
            }
        }
    }

    Ok(output)
}

fn initial_lzw_table() -> Vec<Option<Vec<u8>>> {
    let mut table = (0u8..=255).map(|byte| Some(vec![byte])).collect::<Vec<_>>();
    table.push(None);
    table.push(None);
    table
}

struct BitReader<'a> {
    data: &'a [u8],
    bit_index: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, bit_index: 0 }
    }

    fn read_bits(&mut self, count: usize) -> Option<u16> {
        if self.bit_index + count > self.data.len() * 8 {
            return None;
        }
        let mut value = 0u16;
        for _ in 0..count {
            let byte = self.data[self.bit_index / 8];
            let bit = (byte >> (7 - self.bit_index % 8)) & 1;
            value = (value << 1) | u16::from(bit);
            self.bit_index += 1;
        }
        Some(value)
    }
}
