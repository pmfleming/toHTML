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
