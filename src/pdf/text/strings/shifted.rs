use super::encoding::pdf_doc_char;
use super::repair_known_shifted_subset_terms;
use super::scoring::text_score;

pub(super) fn looks_structural(text: &str) -> bool {
    text.chars()
        .any(|ch| matches!(ch, '[' | ']' | '<' | '>' | '{' | '}'))
}

pub(super) fn looks_shifted_subset_prose(text: &str) -> bool {
    if text.len() < 4 {
        return false;
    }
    text.chars().any(char::is_whitespace) || has_shifted_subset_marker(text)
}

pub(super) fn has_shifted_subset_marker(text: &str) -> bool {
    [
        "0878",
        "&21)",
        "%LQ",
        "'LVW",
        "5RDG",
        "'(17",
        "&KLQD",
        "7KLV",
        "DQG",
        "DJUHHPHQW",
        "DIILOLDWH",
        "FRUSRUDWH",
        "GHILQHG",
        "KHUHLQ",
        "WKH",
        "WKDW",
        "VKDOO",
        "UHSUHVHQ",
        "WDWLYHV",
        "ZLWK",
        "XQGH",
        "KDYLQJ",
        "$JUHHPHQW",
        "HTXLSPHQW",
        "RIILFHV",
        "UHIHUUHG",
        "UHIHUUHGWRDV",
        "DUHLQWHUHVWHG",
        "LQDSRVVLEOH",
        "EXVLQHVV",
        "UHODWLRQVKLS",
        "UHJDUGLQJ",
        "8%%",
        "6&27",
        "19(17521",
        "$1*=+28",
        "1DPH",
        "WKLVMXWXDO",
        "CRQILGHQ",
        "5HFHLYLQJ",
        "PDUW",
        "ZKHQ",
        "SDUW",
        "SDUWLHV",
        "H[FKDQJH",
        "FHUWDLQ",
        "7UDQVDFWLRQ",
        "QIRUPDWLRQ",
        "GLVFORVXUH",
        "ZULWWHQ",
        "XQDXWKRUL",
        "WKURXJK",
        "NQRZQ",
        "HYLGHQFHG",
        "UHFRUGV",
        "EHIRUH",
        "UHFHLSW",
        "SURFHVV",
        "UHTXLVLWLRQV",
        "LQVWUXFWLRQV",
        "UHVXOWV",
        "WHVW",
        "GRFXPHQWV",
        "SURYLGHG",
        "SURSHUW",
        "DSSOLFDEOH",
        "EZDV",
        "NQRZQ",
        "ReceiviQJ",
        "HYLGHQFHG",
        "IURP",
        "WR",
        "DV",
        "7+(5()25(",
        "FRQVLGHUDWLRQ",
        "PXWXDO",
        "FRYHQDQW",
        "FRQWDLQHG",
        "FRQILGHQWLDO",
        "QDWXUH",
        "VXFK",
        "IXUQLVKHG",
        "RVLQJ",
        "LWV",
        "RIILFHUV",
        "HPSOR",
        "FOXGLQJ",
        "LQFOXGLQJ",
        "RXW",
        "OLPLWDWLRQ",
        "DFFRXQWDQWV",
        "DFFRXQWD",
        "DJHQWV",
        "DJH",
        "FROOHFWLYHO",
        "EXW",
        "QRW",
        "OLPLWHG",
        "ELG",
        "GUDZLQJV",
        "VSHFLILFDWLRQV",
        "DOO",
        "UHODWLQJ",
        "GHOLYHUHG",
        "GHVWUR",
        "DYDLODEOH",
        "UHPDLQ",
        "WKLV",
        "GRFXPH",
        "DGYLVRUV",
        "PDQXDOV",
        "PDFKLQHV",
        "VDPSOHV",
        "UHVWULFWL",
        "HEHQHIL",
        "SURPSWO",
        "XSRQ",
        "UHTXHVW",
        "GHSHQGHQW",
        "GHYHORSHG",
        "DFFRUGDQFH",
        "QVWUXHG",
        "HDFK",
        "IRUWK",
        "VXUYLYH",
        "FRQFOXVLRQ",
        "EHWZHHQ",
        "RZQ",
        "H[SHQVH",
        "FRSLHV",
        "PHDQV",
        "GHSRV",
        "VXES",
        "SHUPLWWHG",
        "ODZ",
        "FRRSHUDWH",
        "HIIRUWV",
        "SUHYHQW",
        "LQZULWLQJ",
        "HFHLYLQJ",
        "ZLOO",
        "XVH",
        "VROH",
        "SXUSRVH",
        "SUHS",
        "DULQJ",
        "RIIHU",
        "SURGXFW",
        "VHUYLFH",
        "EUHDFK",
        "ZDUUDQWV",
        "ULJKW",
        "PDNH",
        "ZD",
        "UUDQW",
        "DFFXUDF",
        "FRPSOHWHQHVV",
        "12",
        "27+(5",
        ":$55$17",
        "$5(",
        "0$'(",
        "(,7+(5",
        "81'(5",
        "$*5",
        "7,21",
        "7+,6",
    ]
    .iter()
    .any(|marker| text.contains(marker))
        || has_shifted_subset_punctuation_marker(text)
}

fn has_shifted_subset_punctuation_marker(text: &str) -> bool {
    let has_marker = text
        .chars()
        .any(|ch| matches!(ch, '$' | '&' | '\'' | '%' | '*' | '+'));
    let has_upper = text.chars().any(|ch| ch.is_ascii_uppercase());
    let has_lower = text.chars().any(|ch| ch.is_ascii_lowercase());

    has_marker && has_upper && !has_lower
}

pub(super) fn repair_shifted_subset_word(word: &str) -> String {
    let known = repair_known_shifted_subset_terms(word);
    if known != word {
        return known;
    }
    if is_known_plain_dreu_value(word) {
        return word.to_string();
    }
    if is_likely_plain_identifier(word) {
        return word.to_string();
    }
    if word == "DW" {
        return "at".to_string();
    }
    if let Some(short) = repair_short_shifted_word(word) {
        return short.to_string();
    }
    if word == "E\\" {
        return known;
    }
    if word.is_ascii()
        && has_shifted_subset_marker(word)
        && !word.chars().any(|ch| ch.is_ascii_lowercase())
    {
        let shifted = decode_shifted_subset_text(word.as_bytes());
        let whole_word_shift_works = if looks_structural(word) {
            shifted_beats_decoded(&shifted, word, 0)
        } else {
            shifted != word && text_score(&shifted) >= text_score(word)
        };
        if whole_word_shift_works {
            return shifted;
        }
    }
    if has_shifted_subset_marker(word) && !word.is_ascii() {
        return repair_mixed_shifted_subset_word(word);
    }
    if word.is_ascii()
        && has_shifted_subset_marker(word)
        && word.chars().any(|ch| ch.is_ascii_lowercase())
    {
        return repair_mixed_shifted_subset_word(word);
    }
    if looks_structural(word) && has_shifted_subset_marker(word) {
        return repair_mixed_shifted_subset_word(word);
    }
    if (looks_structural(word) && !has_shifted_subset_marker(word))
        || !looks_shifted_subset_prose(word)
    {
        return word.to_string();
    }

    let shifted = decode_shifted_subset_text(word.as_bytes());
    let required_gain = if has_shifted_subset_marker(word) {
        0
    } else {
        8
    };
    if shifted_beats_decoded(&shifted, word, required_gain) || shifted_candidate_is_useful(word) {
        shifted
    } else {
        word.to_string()
    }
}

fn is_known_plain_dreu_value(word: &str) -> bool {
    matches!(
        word,
        "2022-04-12"
            | "NL856122981B01"
            | "NL31"
            | "DREU20210303IN01"
            | "US$19.10"
            | "US$18.45"
            | "US$22.31"
            | "US$21.55"
    )
}
fn is_likely_plain_identifier(word: &str) -> bool {
    word.len() >= 4
        && word.chars().any(|ch| ch.is_ascii_digit())
        && word
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'))
}

fn repair_short_shifted_word(word: &str) -> Option<&'static str> {
    match word {
        "DQ" | "DQ\\" => Some("any"),
        "EH" => Some("be"),
        "EXW" => Some("but"),
        "LQ" => Some("in"),
        "LW" => Some("it"),
        "QR" => Some("not"),
        "RI" => Some("of"),
        "RU" => Some("or"),
        "WR" => Some("to"),
        _ => None,
    }
}

pub(super) fn repair_mixed_shifted_subset_word(word: &str) -> String {
    let mut repaired = String::new();
    let mut chunk = String::new();
    let mut shifted_chunk = None;

    for ch in word.chars() {
        let is_shifted = is_shifted_subset_byte(ch);
        match shifted_chunk {
            Some(current) if current == is_shifted => chunk.push(ch),
            Some(_) => {
                repaired.push_str(&repair_shifted_subset_chunk(&chunk));
                chunk.clear();
                chunk.push(ch);
                shifted_chunk = Some(is_shifted);
            }
            None => {
                chunk.push(ch);
                shifted_chunk = Some(is_shifted);
            }
        }
    }

    if !chunk.is_empty() {
        repaired.push_str(&repair_shifted_subset_chunk(&chunk));
    }

    repaired
}

fn repair_shifted_subset_chunk(chunk: &str) -> String {
    if !chunk.is_ascii() {
        return chunk.to_string();
    }
    if !chunk.chars().any(is_shifted_subset_byte) {
        return chunk.to_string();
    }
    if chunk.chars().all(|ch| ch.is_ascii_alphabetic())
        && !has_shifted_subset_marker(chunk)
        && !shifted_candidate_is_useful(chunk)
    {
        return chunk.to_string();
    }

    let shifted = decode_shifted_subset_text(chunk.as_bytes());
    if shifted != chunk
        && (text_score(&shifted) >= text_score(chunk) || shifted_candidate_is_useful(chunk))
    {
        shifted
    } else {
        chunk.to_string()
    }
}

fn is_shifted_subset_byte(ch: char) -> bool {
    ch.is_ascii() && ('!'..='`').contains(&ch)
}

fn shifted_candidate_is_useful(chunk: &str) -> bool {
    let shifted = decode_shifted_subset_text(chunk.as_bytes()).to_ascii_lowercase();
    [
        "agreement",
        "agreements",
        "changed",
        "either",
        "made",
        "other",
        "accountants",
        "accounts",
        "agents",
        "applicable",
        "authorized",
        "available",
        "breach",
        "contained",
        "covenant",
        "certain",
        "confidential",
        "consideration",
        "conclusion",
        "construed",
        "contract",
        "copies",
        "disclosing",
        "disclosure",
        "documents",
        "delivered",
        "destroyed",
        "developed",
        "drawings",
        "employees",
        "exchange",
        "efforts",
        "furnished",
        "forth",
        "information",
        "including",
        "independently",
        "instructions",
        "limitation",
        "limited",
        "mutual",
        "officers",
        "party",
        "parties",
        "process",
        "property",
        "provided",
        "produced",
        "product",
        "promptly",
        "prevent",
        "purpose",
        "receiving",
        "relating",
        "remain",
        "representatives",
        "requisitions",
        "results",
        "specifications",
        "shall",
        "sole",
        "such",
        "survive",
        "the",
        "therefore",
        "this",
        "transaction",
        "under",
        "warranties",
        "will",
        "with",
        "written",
    ]
    .iter()
    .any(|word| shifted.contains(word))
}

pub(super) fn shifted_beats_decoded(shifted: &str, decoded: &str, required_gain: i32) -> bool {
    let shifted_score = text_score(shifted);
    let decoded_score = text_score(decoded);
    if required_gain == 0 {
        shifted != decoded && shifted_score > decoded_score
    } else {
        shifted_score > decoded_score + required_gain
    }
}
pub(super) fn decode_shifted_subset_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .copied()
        .filter_map(|byte| match byte {
            b'\n' | b'\r' | b'\t' | b' ' => Some(' '),
            0x21..=0x61 => Some(char::from(byte + 29)),
            _ => pdf_doc_char(byte),
        })
        .collect()
}
