use super::super::*;

#[test]
fn keeps_plain_codes_and_parenthesized_words_after_cmap_decoding() {
    assert_eq!(
        strings::repair_shifted_subset_words("PRG-MUL2 2T21151D000412 TLB01"),
        "PRG-MUL2 2T21151D000412 TLB01"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("(MAKE ONE BOLD)"),
        "(MAKE ONE BOLD)"
    );
}

#[test]
fn keeps_plain_numbers_after_shifted_subset_repair() {
    assert_eq!(
        strings::repair_shifted_subset_words("15 125 55 76 214 -20 4.597 0.63 0,04 123. 2021)"),
        "15 125 55 76 214 -20 4.597 0.63 0,04 123. 2021)"
    );
}

#[test]
fn decodes_shifted_numeric_decimal_tokens() {
    assert_eq!(
        strings::repair_shifted_subset_words("OIPM NINQ MITT MIQM MIPP MION NIMU MIQP MIPM IOP"),
        "2,30 1,14 0,77 0,40 0,33 0,21 1,08 0,43 0,30 0,23"
    );
    assert_eq!(strings::repair_shifted_subset_words("(MION)"), "(0,21)");
}

#[test]
fn decodes_downshifted_table_numbers_and_short_labels() {
    assert_eq!(
        strings::repair_shifted_subset_words("OMOR eN Weight NIQRUKMN NIPUTKNQ ORB"),
        "2025 H1 Weight 1,458.01 1,387.14 ORB"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("22 iM C iN orgs"),
        "22 L0 & L1 orgs"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("Million RMB"),
        "Million RMB"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("total RMS value"),
        "total RMS value"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("21 to 39 , expressed as:"),
        "21 to 39 , expressed as:"
    );
    assert_eq!(strings::repair_shifted_subset_words("RMS40)"), "RMS40)");
    assert_eq!(
        strings::repair_shifted_subset_words("PRB1,458.01"),
        "PRB1,458.01"
    );
}

#[test]
fn decodes_mixed_fiscal_period_markers() {
    assert_eq!(
        strings::repair_shifted_subset_words("2025 e2(Q3+Q4)"),
        "2025 H2(Q3+Q4)"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("2025 eOEnPHn QF"),
        "2025 H2(Q3+Q4)"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("E OMOQe O actual*70%, OMORe O budget OMORe2 actual"),
        "E 2024H2 actual*70%, 2025H2 budget 2025H2 actual"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("ﬂOMOQe O actual*70%"),
        "ﬂ2024H2 actual*70%"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("ZOMORe O actual EOMORe O budget"),
        "=2025H2 actual (2025H2 budget"
    );
}

#[test]
fn repairs_split_fiscal_quarter_labels_after_year_segments() {
    let mut segments = vec![
        TextSegment::new("OMOR".to_string(), 100.0, 240.0, 12.0, 24.0),
        TextSegment::new("QP".to_string(), 124.0, 240.0, 12.0, 12.0),
        TextSegment::new("Budget".to_string(), 102.0, 224.0, 12.0, 36.0),
        TextSegment::new("OMOQ".to_string(), 180.0, 240.0, 12.0, 24.0),
        TextSegment::new("QQ".to_string(), 204.0, 240.0, 12.0, 12.0),
        TextSegment::new("43".to_string(), 260.0, 240.0, 12.0, 12.0),
    ];

    repair_segment_text(&mut segments);

    assert_eq!(segments[0].text, "2025");
    assert_eq!(segments[1].text, "Q3");
    assert_eq!(segments[3].text, "2024");
    assert_eq!(segments[4].text, "Q4");
    assert_eq!(segments[5].text, "43");
}

#[test]
fn decodes_shifted_dash_wrapped_page_number_markers() {
    assert_eq!(strings::repair_shifted_subset_words("ŒOPŒ"), "– 23 –");
    assert_eq!(strings::repair_shifted_subset_words("Œ23Œ"), "– 23 –");
}

#[test]
fn repairs_shifted_symbol_markers_in_prose() {
    assert!(strings::is_readable_text("("));
    assert!(strings::is_readable_text(")"));
    assert_eq!(
        strings::repair_shifted_subset_words("Every contributor should be recognized by––"),
        "Every contributor should be recognized by......"
    );
    assert_eq!(
        strings::repair_shifted_subset_words(
            "Strategic Project Rewarding >&Company Level, Top- down, Global -wise >' Ł Annual"
        ),
        "Strategic Project Rewarding (Company Level, Top-down, Global-wise) • Annual"
    );
    assert_eq!(
        strings::repair_shifted_subset_words(
            "Strategic Project Rewarding E Company Level, Top- down, Global -wise)"
        ),
        "Strategic Project Rewarding (Company Level, Top-down, Global-wise)"
    );
}

#[test]
fn repairs_parenthetical_reporting_period_spacing() {
    assert_eq!(
        strings::repair_shifted_subset_words("Handling Time E Last2 months )"),
        "Handling Time (Last 2 months)"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("By Workstream (Last3 months)"),
        "By Workstream (Last 3 months)"
    );
}

#[test]
fn repairs_split_initial_capital_word_fragments() {
    assert_eq!(
        strings::repair_shifted_subset_words(
            "o Read igital D imming brightness level D I returns value between 0 - 200"
        ),
        "o Read Digital Dimming brightness level, returns value between 0-200"
    );
    assert_eq!(
        strings::repair_shifted_subset_words(
            "o Allows for dimming of driver overD igital D imming bus"
        ),
        "o Allows for dimming of driver over Digital Dimming bus"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("C ommunication P rotocol"),
        "Communication Protocol"
    );
}

#[test]
fn repairs_recital_markers_and_joined_legal_prose_boundaries() {
    assert_eq!(strings::repair_shifted_subset_words("ENNF"), "(11)");
    assert_eq!(strings::repair_shifted_subset_words("EOOF"), "(22)");
    assert_eq!(strings::repair_shifted_subset_words("EOPF"), "(23)");
    assert_eq!(
        strings::repair_shifted_subset_words(
            "ENNF The purpose ofthis Regulationis to ensurea high level"
        ),
        "(11) The purpose of this Regulation is to ensure a high level"
    );
    assert_eq!(
        strings::repair_shifted_subset_words(
            "with digital elementsand their integrated remote data processing solutionsshould be definedas"
        ),
        "with digital elements and their integrated remote data processing solutions should be defined as"
    );
    assert_eq!(
        strings::repair_shifted_subset_words(
            "the productwith digital elementsconcerned I the absence of which"
        ),
        "the product with digital elements concerned, the absence of which"
    );
    assert_eq!(
        strings::repair_shifted_subset_words("as regards the Union™s dependency"),
        "as regards the Union's dependency"
    );
}

#[test]
fn decodes_concatenated_shifted_subset_text() {
    let stream = b"BT (7KLV$JUHHPHQWVKDOOEHFRQILGHQWLDO) Tj ET";

    assert_eq!(
        extract_text(stream).as_deref(),
        Some("ThisAgreementshallbeconfidential")
    );
}
