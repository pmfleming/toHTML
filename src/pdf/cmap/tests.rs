use super::*;

#[test]
fn parses_bfchar_mappings() {
    let cmap = CMap::parse(
        br#"
        beginbfchar
        <01> <0041>
        <02> <0042>
        endbfchar
        "#,
    );

    assert_eq!(cmap.decode(&[1, 2]), "AB");
}

#[test]
fn maps_wingdings_private_use_ballot_box_to_unicode_square() {
    let cmap = CMap::parse(
        br#"
        beginbfchar
        <0087> <F070>
        endbfchar
        "#,
    );

    assert_eq!(cmap.decode(&[0, 0x87]), "□");
}

#[test]
fn parses_packed_bfchar_mappings_on_one_line() {
    let cmap = CMap::parse(
        br#"
        /CIDInit /ProcSet findresource begin 12 dict begin begincmap 3 beginbfchar <0003> <0020> <0037> <0054> <004B> <0068> endbfchar endcmap
        "#,
    );

    assert_eq!(cmap.decode(&[0, 3, 0, 0x37, 0, 0x4b]), " Th");
}

#[test]
fn parses_bfrange_mappings() {
    let cmap = CMap::parse(
        br#"
        beginbfrange
        <01> <03> <0041>
        endbfrange
        "#,
    );

    assert_eq!(cmap.decode(&[1, 2, 3]), "ABC");
}

#[test]
fn keeps_large_bfrange_mappings_lazy() {
    let cmap = CMap::parse(
        br#"
        beginbfrange
        <00000000> <FFFFFFFF> <0041>
        endbfrange
        "#,
    );

    assert!(cmap.entries.is_empty());
    assert_eq!(cmap.ranges.len(), 1);
    assert_eq!(cmap.decode(&[0, 0, 0, 1]), "B");
}

#[test]
fn preserves_leading_zero_bytes_in_bfrange_sources() {
    let cmap = CMap::parse(
        br#"
        beginbfrange
        <0003> <0004> [<0020> <0041>]
        endbfrange
        "#,
    );

    assert_eq!(cmap.decode(&[0, 3, 0, 4]), " A");
}

#[test]
fn parses_codespace_notdef_usecmap_and_wmode() {
    let cmap = CMap::parse(
        br#"
        /Base /UniJIS-UCS2-H usecmap
        /WMode 1 def
        1 begincodespacerange
        <0000> <FFFF>
        endcodespacerange
        1 beginnotdefrange
        <0000> <0001> 0
        endnotdefrange
        1 beginbfchar
        <0041> <005A>
        endbfchar
        "#,
    );

    let decoded = cmap.decode_with_stats(&[0, 0, 0, 0x41, 0x4e, 0x2d]);

    assert_eq!(decoded.text, "Z中");
    assert_eq!(decoded.stats.notdef, 1);
    assert_eq!(decoded.stats.mapped, 1);
    assert_eq!(decoded.stats.fallback_mapped, 1);
    assert_eq!(cmap.writing_mode(), super::WritingMode::Vertical);
}

#[test]
fn does_not_split_unmapped_multibyte_codes_into_raw_ascii() {
    let cmap = CMap::parse(
        br#"
        1 begincodespacerange
        <0000> <FFFF>
        endcodespacerange
        "#,
    );

    let decoded = cmap.decode_with_stats(&[0, 0x41]);

    assert_eq!(decoded.text, "");
    assert_eq!(decoded.stats.raw_fallback, 1);
}
