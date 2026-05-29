mod basic;
mod lzw;
mod predictor;

use crate::ConvertError;

use super::super::object::{PdfDictionary, PdfValue};
use basic::decode_one_filter;
use predictor::{apply_predictor, DecodeParams};

#[cfg(test)]
use lzw::lzw_decode;
#[cfg(test)]
use predictor::tiff_predictor;

pub(super) fn decode_legacy_stream(header: &[u8], data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let filters = legacy_stream_filters(header);
    decode_filters(&filters, data)
}

pub(super) fn decode_stream(
    dictionary: &PdfDictionary,
    data: &[u8],
) -> Result<Vec<u8>, ConvertError> {
    let mut decoded = data.to_vec();
    for filter in stream_filter_specs(dictionary) {
        decoded = decode_one_filter(&filter.name, &decoded)?;
        decoded = apply_predictor(&decoded, &filter.params)?;
    }
    Ok(decoded)
}

pub(super) fn decode_filters(filters: &[String], data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let mut decoded = data.to_vec();
    for filter in filters {
        decoded = decode_one_filter(filter, &decoded)?;
    }
    Ok(decoded)
}

pub(super) fn stream_filters(dictionary: &PdfDictionary) -> Vec<String> {
    match dictionary.get("Filter") {
        Some(PdfValue::Name(name)) => vec![name.clone()],
        Some(PdfValue::Array(values)) => values
            .iter()
            .filter_map(|value| match value {
                PdfValue::Name(name) => Some(name.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn legacy_stream_filters(header: &[u8]) -> Vec<String> {
    let names = [
        "ASCIIHexDecode",
        "ASCII85Decode",
        "LZWDecode",
        "FlateDecode",
        "RunLengthDecode",
        "DCTDecode",
        "JPXDecode",
        "CCITTFaxDecode",
        "JBIG2Decode",
    ];
    let header = String::from_utf8_lossy(header);
    names
        .into_iter()
        .filter(|name| header.contains(&format!("/{name}")))
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone)]
struct StreamFilterSpec {
    name: String,
    params: Option<DecodeParams>,
}

fn stream_filter_specs(dictionary: &PdfDictionary) -> Vec<StreamFilterSpec> {
    let filters = stream_filters(dictionary);
    let params = decode_params(dictionary);
    filters
        .into_iter()
        .enumerate()
        .map(|(index, name)| StreamFilterSpec {
            name,
            params: params_for_filter(&params, index),
        })
        .collect()
}

fn decode_params(dictionary: &PdfDictionary) -> Vec<Option<DecodeParams>> {
    match dictionary
        .get("DecodeParms")
        .or_else(|| dictionary.get("DP"))
    {
        Some(PdfValue::Dictionary(params)) => vec![Some(decode_params_dictionary(params))],
        Some(PdfValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                PdfValue::Dictionary(params) => Some(decode_params_dictionary(params)),
                PdfValue::Null => None,
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn params_for_filter(params: &[Option<DecodeParams>], index: usize) -> Option<DecodeParams> {
    params
        .get(index)
        .cloned()
        .flatten()
        .or_else(|| (params.len() == 1).then(|| params[0].clone()).flatten())
}

fn decode_params_dictionary(dictionary: &PdfDictionary) -> DecodeParams {
    DecodeParams {
        predictor: integer(dictionary, "Predictor").unwrap_or(1),
        colors: integer(dictionary, "Colors")
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(1),
        bits_per_component: integer(dictionary, "BitsPerComponent")
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(8),
        columns: integer(dictionary, "Columns")
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(1),
    }
}

fn integer(dictionary: &PdfDictionary, key: &str) -> Option<i64> {
    match dictionary.get(key)? {
        PdfValue::Integer(value) => Some(*value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_simple_lzw_stream() {
        assert_eq!(
            lzw_decode(&[0x80, 0x10, 0x60, 0x20]).unwrap(),
            b"A".to_vec()
        );
    }

    #[test]
    fn applies_png_sub_predictor() {
        let params = Some(DecodeParams {
            predictor: 15,
            colors: 1,
            bits_per_component: 8,
            columns: 3,
        });

        assert_eq!(
            apply_predictor(&[1, 10, 2, 3], &params).unwrap(),
            vec![10, 12, 15]
        );
    }

    #[test]
    fn applies_tiff_predictor() {
        let params = DecodeParams {
            predictor: 2,
            colors: 1,
            bits_per_component: 8,
            columns: 3,
        };

        assert_eq!(
            tiff_predictor(&[10, 2, 3], &params).unwrap(),
            vec![10, 12, 15]
        );
    }
}
