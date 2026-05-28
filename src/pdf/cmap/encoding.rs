use crate::pdf::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfValue};

pub(super) fn simple_font_mappings(
    objects: &PdfObjects,
    font_dictionary: &PdfDictionary,
) -> Vec<(u8, String)> {
    let Some(encoding) = resolved_value(objects, font_dictionary.get("Encoding")) else {
        return Vec::new();
    };

    match encoding {
        PdfValue::Name(name) => base_encoding_mappings(name),
        PdfValue::Dictionary(dictionary) => dictionary_encoding_mappings(objects, dictionary),
        _ => Vec::new(),
    }
}

fn dictionary_encoding_mappings(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
) -> Vec<(u8, String)> {
    let mut mappings = dictionary
        .name("BaseEncoding")
        .map(base_encoding_mappings)
        .unwrap_or_default();

    let Some(differences) = resolved_value(objects, dictionary.get("Differences")) else {
        return mappings;
    };
    let PdfValue::Array(differences) = differences else {
        return mappings;
    };

    let mut code = 0u8;
    for difference in differences {
        match difference {
            PdfValue::Integer(value) => {
                if let Ok(next) = u8::try_from(*value) {
                    code = next;
                }
            }
            PdfValue::Name(name) => {
                mappings.retain(|(existing, _)| *existing != code);
                if let Some(unicode) = glyph_name_to_unicode(name) {
                    mappings.push((code, unicode));
                }
                code = code.saturating_add(1);
            }
            _ => {}
        }
    }

    mappings
}

fn resolved_value<'a>(
    objects: &'a PdfObjects,
    value: Option<&'a PdfValue>,
) -> Option<&'a PdfValue> {
    match value? {
        PdfValue::Reference(reference) => Some(&objects.get(*reference)?.value),
        value => Some(value),
    }
}

fn base_encoding_mappings(name: &str) -> Vec<(u8, String)> {
    (0u8..=255)
        .filter_map(|code| match name {
            "WinAnsiEncoding" => win_ansi_char(code).map(|ch| (code, ch.to_string())),
            "MacRomanEncoding" => mac_roman_char(code).map(|ch| (code, ch.to_string())),
            "StandardEncoding" => standard_encoding_name(code)
                .and_then(glyph_name_to_unicode)
                .map(|text| (code, text)),
            _ => None,
        })
        .collect()
}

fn standard_encoding_name(code: u8) -> Option<&'static str> {
    match code {
        0x20 => Some("space"),
        0x21 => Some("exclam"),
        0x22 => Some("quotedbl"),
        0x23 => Some("numbersign"),
        0x24 => Some("dollar"),
        0x25 => Some("percent"),
        0x26 => Some("ampersand"),
        0x27 => Some("quoteright"),
        0x28 => Some("parenleft"),
        0x29 => Some("parenright"),
        0x2a => Some("asterisk"),
        0x2b => Some("plus"),
        0x2c => Some("comma"),
        0x2d => Some("hyphen"),
        0x2e => Some("period"),
        0x2f => Some("slash"),
        0x30..=0x39 => Some(
            [
                "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
            ][(code - 0x30) as usize],
        ),
        0x3a => Some("colon"),
        0x3b => Some("semicolon"),
        0x3c => Some("less"),
        0x3d => Some("equal"),
        0x3e => Some("greater"),
        0x3f => Some("question"),
        0x40 => Some("at"),
        0x41..=0x5a => Some(ASCII_UPPER_NAMES[(code - 0x41) as usize]),
        0x5b => Some("bracketleft"),
        0x5c => Some("backslash"),
        0x5d => Some("bracketright"),
        0x5e => Some("asciicircum"),
        0x5f => Some("underscore"),
        0x60 => Some("quoteleft"),
        0x61..=0x7a => Some(ASCII_LOWER_NAMES[(code - 0x61) as usize]),
        0x7b => Some("braceleft"),
        0x7c => Some("bar"),
        0x7d => Some("braceright"),
        0x7e => Some("asciitilde"),
        _ => None,
    }
}

fn win_ansi_char(code: u8) -> Option<char> {
    match code {
        0x20..=0x7e | 0xa0..=0xff => Some(char::from(code)),
        0x80 => Some('€'),
        0x82 => Some('‚'),
        0x83 => Some('ƒ'),
        0x84 => Some('„'),
        0x85 => Some('…'),
        0x86 => Some('†'),
        0x87 => Some('‡'),
        0x88 => Some('ˆ'),
        0x89 => Some('‰'),
        0x8a => Some('Š'),
        0x8b => Some('‹'),
        0x8c => Some('Œ'),
        0x8e => Some('Ž'),
        0x91 => Some('‘'),
        0x92 => Some('’'),
        0x93 => Some('“'),
        0x94 => Some('”'),
        0x95 => Some('•'),
        0x96 => Some('–'),
        0x97 => Some('—'),
        0x98 => Some('˜'),
        0x99 => Some('™'),
        0x9a => Some('š'),
        0x9b => Some('›'),
        0x9c => Some('œ'),
        0x9e => Some('ž'),
        0x9f => Some('Ÿ'),
        _ => None,
    }
}

fn mac_roman_char(code: u8) -> Option<char> {
    const HIGH: [char; 128] = [
        'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á', 'à', 'â', 'ä', 'ã', 'å', 'ç', 'é', 'è', 'ê', 'ë',
        'í', 'ì', 'î', 'ï', 'ñ', 'ó', 'ò', 'ô', 'ö', 'õ', 'ú', 'ù', 'û', 'ü', '†', '°', '¢', '£',
        '§', '•', '¶', 'ß', '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø', '∞', '±', '≤', '≥', '¥', 'µ',
        '∂', '∑', '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø', '¿', '¡', '¬', '√', 'ƒ', '≈', '∆', '«',
        '»', '…', '\u{a0}', 'À', 'Ã', 'Õ', 'Œ', 'œ', '–', '—', '“', '”', '‘', '’', '÷', '◊', 'ÿ',
        'Ÿ', '⁄', '€', '‹', '›', 'ﬁ', 'ﬂ', '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á', 'Ë', 'È', 'Í',
        'Î', 'Ï', 'Ì', 'Ó', 'Ô', '\u{f8ff}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜', '¯', '˘', '˙', '˚',
        '¸', '˝', '˛', 'ˇ',
    ];

    match code {
        0x20..=0x7e => Some(char::from(code)),
        0x80..=0xff => Some(HIGH[(code - 0x80) as usize]),
        _ => None,
    }
}

fn glyph_name_to_unicode(name: &str) -> Option<String> {
    let name = name.split_once('.').map_or(name, |(base, _)| base);
    if name.contains('_') {
        let text = name
            .split('_')
            .map(glyph_name_to_unicode)
            .collect::<Option<Vec<_>>>()?
            .join("");
        return (!text.is_empty()).then_some(text);
    }
    if let Some(text) = unicode_name_sequence(name) {
        return Some(text);
    }

    glyph_name_char(name).map(|ch| ch.to_string())
}

fn unicode_name_sequence(name: &str) -> Option<String> {
    if let Some(hex) = name.strip_prefix("uni") {
        if hex.len() >= 4 && hex.len() % 4 == 0 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return hex
                .as_bytes()
                .chunks_exact(4)
                .map(|chunk| {
                    let hex = std::str::from_utf8(chunk).ok()?;
                    char::from_u32(u32::from_str_radix(hex, 16).ok()?)
                })
                .collect();
        }
    }
    if let Some(hex) = name.strip_prefix('u') {
        if (4..=6).contains(&hex.len()) && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return char::from_u32(u32::from_str_radix(hex, 16).ok()?).map(|ch| ch.to_string());
        }
    }
    None
}

const ASCII_UPPER_NAMES: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

const ASCII_LOWER_NAMES: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

fn glyph_name_char(name: &str) -> Option<char> {
    match name {
        "A" => Some('A'),
        "AE" => Some('Æ'),
        "Aacute" => Some('Á'),
        "Acircumflex" => Some('Â'),
        "Adieresis" => Some('Ä'),
        "Agrave" => Some('À'),
        "Aring" => Some('Å'),
        "Atilde" => Some('Ã'),
        "B" => Some('B'),
        "C" => Some('C'),
        "Ccedilla" => Some('Ç'),
        "D" => Some('D'),
        "E" => Some('E'),
        "Eacute" => Some('É'),
        "Ecircumflex" => Some('Ê'),
        "Edieresis" => Some('Ë'),
        "Egrave" => Some('È'),
        "Eth" => Some('Ð'),
        "Euro" => Some('€'),
        "F" => Some('F'),
        "G" => Some('G'),
        "H" => Some('H'),
        "I" => Some('I'),
        "Iacute" => Some('Í'),
        "Icircumflex" => Some('Î'),
        "Idieresis" => Some('Ï'),
        "Igrave" => Some('Ì'),
        "J" => Some('J'),
        "K" => Some('K'),
        "L" => Some('L'),
        "Lslash" => Some('Ł'),
        "M" => Some('M'),
        "N" => Some('N'),
        "Ntilde" => Some('Ñ'),
        "O" => Some('O'),
        "OE" => Some('Œ'),
        "Oacute" => Some('Ó'),
        "Ocircumflex" => Some('Ô'),
        "Odieresis" => Some('Ö'),
        "Ograve" => Some('Ò'),
        "Oslash" => Some('Ø'),
        "Otilde" => Some('Õ'),
        "P" => Some('P'),
        "Q" => Some('Q'),
        "R" => Some('R'),
        "S" => Some('S'),
        "Scaron" => Some('Š'),
        "T" => Some('T'),
        "Thorn" => Some('Þ'),
        "U" => Some('U'),
        "Uacute" => Some('Ú'),
        "Ucircumflex" => Some('Û'),
        "Udieresis" => Some('Ü'),
        "Ugrave" => Some('Ù'),
        "V" => Some('V'),
        "W" => Some('W'),
        "X" => Some('X'),
        "Y" => Some('Y'),
        "Yacute" => Some('Ý'),
        "Ydieresis" => Some('Ÿ'),
        "Z" => Some('Z'),
        "Zcaron" => Some('Ž'),
        "a" => Some('a'),
        "aacute" => Some('á'),
        "acircumflex" => Some('â'),
        "acute" => Some('´'),
        "adieresis" => Some('ä'),
        "ae" => Some('æ'),
        "agrave" => Some('à'),
        "ampersand" => Some('&'),
        "aring" => Some('å'),
        "asciicircum" => Some('^'),
        "asciitilde" => Some('~'),
        "asterisk" => Some('*'),
        "at" => Some('@'),
        "atilde" => Some('ã'),
        "b" => Some('b'),
        "backslash" => Some('\\'),
        "bar" => Some('|'),
        "braceleft" => Some('{'),
        "braceright" => Some('}'),
        "bracketleft" => Some('['),
        "bracketright" => Some(']'),
        "breve" => Some('˘'),
        "brokenbar" => Some('¦'),
        "bullet" => Some('•'),
        "c" => Some('c'),
        "caron" => Some('ˇ'),
        "ccedilla" => Some('ç'),
        "cedilla" => Some('¸'),
        "cent" => Some('¢'),
        "circumflex" => Some('ˆ'),
        "colon" => Some(':'),
        "comma" => Some(','),
        "copyright" => Some('©'),
        "currency" => Some('¤'),
        "d" => Some('d'),
        "dagger" => Some('†'),
        "daggerdbl" => Some('‡'),
        "degree" => Some('°'),
        "dieresis" => Some('¨'),
        "divide" => Some('÷'),
        "dollar" => Some('$'),
        "dotaccent" => Some('˙'),
        "dotlessi" => Some('ı'),
        "e" => Some('e'),
        "eacute" => Some('é'),
        "ecircumflex" => Some('ê'),
        "edieresis" => Some('ë'),
        "egrave" => Some('è'),
        "eight" => Some('8'),
        "ellipsis" => Some('…'),
        "emdash" => Some('—'),
        "endash" => Some('–'),
        "equal" => Some('='),
        "eth" => Some('ð'),
        "exclam" => Some('!'),
        "exclamdown" => Some('¡'),
        "f" => Some('f'),
        "fi" => Some('ﬁ'),
        "five" => Some('5'),
        "fl" => Some('ﬂ'),
        "florin" => Some('ƒ'),
        "four" => Some('4'),
        "fraction" => Some('⁄'),
        "g" => Some('g'),
        "germandbls" => Some('ß'),
        "grave" => Some('`'),
        "greater" => Some('>'),
        "guillemotleft" => Some('«'),
        "guillemotright" => Some('»'),
        "guilsinglleft" => Some('‹'),
        "guilsinglright" => Some('›'),
        "h" => Some('h'),
        "hungarumlaut" => Some('˝'),
        "hyphen" => Some('-'),
        "i" => Some('i'),
        "iacute" => Some('í'),
        "icircumflex" => Some('î'),
        "idieresis" => Some('ï'),
        "igrave" => Some('ì'),
        "j" => Some('j'),
        "k" => Some('k'),
        "l" => Some('l'),
        "less" => Some('<'),
        "logicalnot" => Some('¬'),
        "lslash" => Some('ł'),
        "m" => Some('m'),
        "macron" => Some('¯'),
        "minus" => Some('−'),
        "mu" => Some('µ'),
        "multiply" => Some('×'),
        "n" => Some('n'),
        "nine" => Some('9'),
        "ntilde" => Some('ñ'),
        "numbersign" => Some('#'),
        "o" => Some('o'),
        "oacute" => Some('ó'),
        "ocircumflex" => Some('ô'),
        "odieresis" => Some('ö'),
        "oe" => Some('œ'),
        "ogonek" => Some('˛'),
        "ograve" => Some('ò'),
        "one" => Some('1'),
        "onehalf" => Some('½'),
        "onequarter" => Some('¼'),
        "onesuperior" => Some('¹'),
        "ordfeminine" => Some('ª'),
        "ordmasculine" => Some('º'),
        "oslash" => Some('ø'),
        "otilde" => Some('õ'),
        "p" => Some('p'),
        "paragraph" => Some('¶'),
        "parenleft" => Some('('),
        "parenright" => Some(')'),
        "percent" => Some('%'),
        "period" => Some('.'),
        "periodcentered" => Some('·'),
        "perthousand" => Some('‰'),
        "plus" => Some('+'),
        "plusminus" => Some('±'),
        "q" => Some('q'),
        "question" => Some('?'),
        "questiondown" => Some('¿'),
        "quotedbl" => Some('"'),
        "quotedblbase" => Some('„'),
        "quotedblleft" => Some('“'),
        "quotedblright" => Some('”'),
        "quoteleft" => Some('‘'),
        "quoteright" | "quotesingle" => Some('\''),
        "r" => Some('r'),
        "registered" => Some('®'),
        "ring" => Some('˚'),
        "s" => Some('s'),
        "scaron" => Some('š'),
        "section" => Some('§'),
        "semicolon" => Some(';'),
        "seven" => Some('7'),
        "six" => Some('6'),
        "slash" => Some('/'),
        "space" | "nbspace" => Some(' '),
        "sterling" => Some('£'),
        "t" => Some('t'),
        "thorn" => Some('þ'),
        "three" => Some('3'),
        "threequarters" => Some('¾'),
        "threesuperior" => Some('³'),
        "tilde" => Some('˜'),
        "trademark" => Some('™'),
        "two" => Some('2'),
        "twosuperior" => Some('²'),
        "u" => Some('u'),
        "uacute" => Some('ú'),
        "ucircumflex" => Some('û'),
        "udieresis" => Some('ü'),
        "ugrave" => Some('ù'),
        "underscore" => Some('_'),
        "v" => Some('v'),
        "w" => Some('w'),
        "x" => Some('x'),
        "y" => Some('y'),
        "yacute" => Some('ý'),
        "ydieresis" => Some('ÿ'),
        "yen" => Some('¥'),
        "z" => Some('z'),
        "zcaron" => Some('ž'),
        "zero" => Some('0'),
        _ => None,
    }
}
