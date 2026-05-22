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
