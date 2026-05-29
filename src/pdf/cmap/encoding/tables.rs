// Local PDF simple-font encodings and Adobe Glyph List names.
//
// Base encodings stay hand-curated to the simple-font tables this converter
// uses. Glyph-name lookup delegates to a generated, checked-in AGL table.

use super::agl;

pub(super) fn base_encoding_char(encoding: &str, code: u8) -> Option<char> {
    match encoding {
        "WinAnsiEncoding" => win_ansi(code),
        "MacRomanEncoding" => mac_roman(code),
        "MacExpertEncoding" => None,
        "StandardEncoding" => standard(code),
        "SymbolEncoding" => symbol(code),
        "ZapfDingbatsEncoding" => zapf_dingbats(code),
        _ => None,
    }
}

pub(super) fn glyph_name_to_unicode(name: &str) -> Option<String> {
    if name.len() == 1 && name.as_bytes()[0].is_ascii_alphanumeric() {
        return Some(name.to_string());
    }

    match name {
        // Seen in some producers as a short alias; AGL uses `quotesingle`.
        "quote" => Some("'"),
        _ => agl::glyph_name_to_unicode(name),
    }
    .map(str::to_string)
}

fn win_ansi(code: u8) -> Option<char> {
    match code {
        0x20..=0x7e => Some(char::from(code)),
        0xa0..=0xff => Some(char::from_u32(u32::from(code))?),
        0x80 => Some('\u{20ac}'),
        0x82 => Some('\u{201a}'),
        0x83 => Some('\u{0192}'),
        0x84 => Some('\u{201e}'),
        0x85 => Some('\u{2026}'),
        0x86 => Some('\u{2020}'),
        0x87 => Some('\u{2021}'),
        0x88 => Some('\u{02c6}'),
        0x89 => Some('\u{2030}'),
        0x8a => Some('\u{0160}'),
        0x8b => Some('\u{2039}'),
        0x8c => Some('\u{0152}'),
        0x8e => Some('\u{017d}'),
        0x91 => Some('\u{2018}'),
        0x92 => Some('\u{2019}'),
        0x93 => Some('\u{201c}'),
        0x94 => Some('\u{201d}'),
        0x95 => Some('\u{2022}'),
        0x96 => Some('\u{2013}'),
        0x97 => Some('\u{2014}'),
        0x98 => Some('\u{02dc}'),
        0x99 => Some('\u{2122}'),
        0x9a => Some('\u{0161}'),
        0x9b => Some('\u{203a}'),
        0x9c => Some('\u{0153}'),
        0x9e => Some('\u{017e}'),
        0x9f => Some('\u{0178}'),
        _ => None,
    }
}

fn mac_roman(code: u8) -> Option<char> {
    match code {
        0x20..=0x7e => Some(char::from(code)),
        0xdb => Some('\u{20ac}'),
        0xd2 => Some('\u{201c}'),
        0xd3 => Some('\u{201d}'),
        0xd4 => Some('\u{2018}'),
        0xd5 => Some('\u{2019}'),
        0xa5 => Some('\u{2022}'),
        0xc9 => Some('\u{2026}'),
        0xd0 => Some('\u{2013}'),
        0xd1 => Some('\u{2014}'),
        _ => None,
    }
}

fn standard(code: u8) -> Option<char> {
    match code {
        0x20 => Some('\u{00a0}'),
        0x21..=0x26 | 0x28..=0x5f | 0x61..=0x7e => Some(char::from(code)),
        0x27 => Some('\u{2019}'),
        0x60 => Some('\u{2018}'),
        0xa1 => Some('\u{00a1}'),
        0xa2 => Some('\u{00a2}'),
        0xa3 => Some('\u{00a3}'),
        0xa4 => Some('\u{2215}'),
        0xa5 => Some('\u{00a5}'),
        0xa6 => Some('\u{0192}'),
        0xa7 => Some('\u{00a7}'),
        0xa8 => Some('\u{00a4}'),
        0xa9 => Some('\''),
        0xaa => Some('\u{201c}'),
        0xab => Some('\u{00ab}'),
        0xac => Some('\u{2039}'),
        0xad => Some('\u{203a}'),
        0xae => Some('\u{fb01}'),
        0xaf => Some('\u{fb02}'),
        0xb1 => Some('\u{2013}'),
        0xb2 => Some('\u{2020}'),
        0xb3 => Some('\u{2021}'),
        0xb4 => Some('\u{2219}'),
        0xb6 => Some('\u{00b6}'),
        0xb7 => Some('\u{2022}'),
        0xb8 => Some('\u{201a}'),
        0xb9 => Some('\u{201e}'),
        0xba => Some('\u{201d}'),
        0xbb => Some('\u{00bb}'),
        0xbc => Some('\u{2026}'),
        0xbd => Some('\u{2030}'),
        0xbf => Some('\u{00bf}'),
        0xd0 => Some('\u{2014}'),
        0xe1 => Some('\u{00c6}'),
        0xe9 => Some('\u{0141}'),
        0xea => Some('\u{00d8}'),
        0xeb => Some('\u{0152}'),
        0xf1 => Some('\u{00e6}'),
        0xf5 => Some('\u{0131}'),
        0xf9 => Some('\u{0142}'),
        0xfa => Some('\u{00f8}'),
        0xfb => Some('\u{0153}'),
        0xfc => Some('\u{00df}'),
        _ => None,
    }
}

fn symbol(code: u8) -> Option<char> {
    match code {
        0x20 => Some('\u{00a0}'),
        0x21 => Some('!'),
        0x22 => Some('\u{2200}'),
        0x24 => Some('\u{2203}'),
        0x27 => Some('\u{220b}'),
        0x2a => Some('\u{2217}'),
        0x2d => Some('\u{2212}'),
        0x30..=0x3f => Some(char::from(code)),
        0x41 => Some('\u{0391}'),
        0x42 => Some('\u{0392}'),
        0x43 => Some('\u{03a7}'),
        0x44 => Some('\u{2206}'),
        0x45 => Some('\u{0395}'),
        0x46 => Some('\u{03a6}'),
        0x47 => Some('\u{0393}'),
        0x48 => Some('\u{0397}'),
        0x49 => Some('\u{0399}'),
        0x4a => Some('\u{03d1}'),
        0x4b => Some('\u{039a}'),
        0x4c => Some('\u{039b}'),
        0x4d => Some('\u{039c}'),
        0x4e => Some('\u{039d}'),
        0x4f => Some('\u{039f}'),
        0x50 => Some('\u{03a0}'),
        0x51 => Some('\u{0398}'),
        0x52 => Some('\u{03a1}'),
        0x53 => Some('\u{03a3}'),
        0x54 => Some('\u{03a4}'),
        0x55 => Some('\u{03a5}'),
        0x56 => Some('\u{03c2}'),
        0x57 => Some('\u{2126}'),
        0x58 => Some('\u{039e}'),
        0x59 => Some('\u{03a8}'),
        0x5a => Some('\u{0396}'),
        0x61 => Some('\u{03b1}'),
        0x62 => Some('\u{03b2}'),
        0x63 => Some('\u{03c7}'),
        0x64 => Some('\u{03b4}'),
        0x65 => Some('\u{03b5}'),
        0x66 => Some('\u{03c6}'),
        0x67 => Some('\u{03b3}'),
        0x68 => Some('\u{03b7}'),
        0x69 => Some('\u{03b9}'),
        0x6a => Some('\u{03d5}'),
        0x6b => Some('\u{03ba}'),
        0x6c => Some('\u{03bb}'),
        0x6d => Some('\u{03bc}'),
        0x6e => Some('\u{03bd}'),
        0x6f => Some('\u{03bf}'),
        0x70 => Some('\u{03c0}'),
        0x71 => Some('\u{03b8}'),
        0x72 => Some('\u{03c1}'),
        0x73 => Some('\u{03c3}'),
        0x74 => Some('\u{03c4}'),
        0x75 => Some('\u{03c5}'),
        0x76 => Some('\u{03d6}'),
        0x77 => Some('\u{03c9}'),
        0x78 => Some('\u{03be}'),
        0x79 => Some('\u{03c8}'),
        0x7a => Some('\u{03b6}'),
        0xa0 => Some('\u{20ac}'),
        0xa3 => Some('\u{2264}'),
        0xa5 => Some('\u{221e}'),
        0xac => Some('\u{2190}'),
        0xae => Some('\u{2192}'),
        0xb3 => Some('\u{2265}'),
        0xb6 => Some('\u{2202}'),
        0xb7 => Some('\u{2022}'),
        0xb9 => Some('\u{2260}'),
        0xba => Some('\u{2261}'),
        0xbb => Some('\u{2248}'),
        0xc5 => Some('\u{2295}'),
        0xc6 => Some('\u{2205}'),
        0xc7 => Some('\u{2229}'),
        0xc8 => Some('\u{222a}'),
        0xd0 => Some('\u{2220}'),
        0xd1 => Some('\u{2207}'),
        0xd5 => Some('\u{220f}'),
        0xd6 => Some('\u{221a}'),
        0xe5 => Some('\u{2211}'),
        0xf2 => Some('\u{222b}'),
        _ => None,
    }
}

fn zapf_dingbats(code: u8) -> Option<char> {
    match code {
        0x20 => Some('\u{00a0}'),
        0x21 => Some('\u{2701}'),
        0x22 => Some('\u{2702}'),
        0x23 => Some('\u{2703}'),
        0x24 => Some('\u{2704}'),
        0x25 => Some('\u{260e}'),
        0x26 => Some('\u{2706}'),
        0x27 => Some('\u{2707}'),
        0x28 => Some('\u{2708}'),
        0x29 => Some('\u{2709}'),
        0x2a => Some('\u{261b}'),
        0x2b => Some('\u{261e}'),
        0x2c => Some('\u{270c}'),
        0x2d => Some('\u{270d}'),
        0x2e => Some('\u{270e}'),
        0x2f => Some('\u{270f}'),
        0x33 => Some('\u{2713}'),
        0x34 => Some('\u{2714}'),
        0x35 => Some('\u{2715}'),
        0x36 => Some('\u{2716}'),
        0x37 => Some('\u{2717}'),
        0x38 => Some('\u{2718}'),
        0x6c => Some('\u{25cf}'),
        0x6e => Some('\u{25a0}'),
        0x73 => Some('\u{25b2}'),
        0x74 => Some('\u{25bc}'),
        0x75 => Some('\u{25c6}'),
        0xa8 => Some('\u{2663}'),
        0xa9 => Some('\u{2666}'),
        0xaa => Some('\u{2665}'),
        0xab => Some('\u{2660}'),
        0xd5 => Some('\u{2794}'),
        0xd6 => Some('\u{2192}'),
        0xd7 => Some('\u{2194}'),
        0xd8 => Some('\u{2195}'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::glyph_name_to_unicode;

    #[test]
    fn decodes_generated_agl_names() {
        assert_eq!(
            glyph_name_to_unicode("Lcommaaccent"),
            Some("\u{013b}".to_string())
        );
        assert_eq!(
            glyph_name_to_unicode("qofhatafpatah"),
            Some("\u{05e7}\u{05b2}".to_string())
        );
    }
}
