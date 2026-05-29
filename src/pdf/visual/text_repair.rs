mod common;
mod joined;
mod license;
mod urls;

pub(super) use urls::annotation_aligned_url_segments;

pub(super) fn repair_visual_text(text: &str) -> String {
    if text.trim() == "ISO 20022" {
        return text.to_string();
    }
    common::repair_common_visual_text(&super::super::text::repair_shifted_subset_text(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_embedded_license_punctuation_from_visual_text() {
        let text =
            "which can be produced by --`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---equipment tested";

        assert_eq!(
            repair_visual_text(text),
            "which can be produced by equipment tested"
        );
    }

    #[test]
    fn repairs_joined_common_words() {
        assert_eq!(
            repair_visual_text("document andcan be subject"),
            "document and can be subject"
        );
    }

    #[test]
    fn repairs_downshifted_table_labels_at_visual_boundary() {
        assert_eq!(repair_visual_text("2025 eN"), "2025 H1");
        assert_eq!(repair_visual_text("2025 e2(Q3+Q4)"), "2025 H2(Q3+Q4)");
        assert_eq!(repair_visual_text("2025 eOEnPHn QF"), "2025 H2(Q3+Q4)");
        assert_eq!(repair_visual_text("OMOQe O actual"), "2024H2 actual");
        assert_eq!(repair_visual_text("Q3"), "Q3");
        assert_eq!(repair_visual_text("NIQRUKMN"), "1,458.01");
    }

    #[test]
    fn repairs_shifted_symbol_markers_at_visual_boundary() {
        assert_eq!(repair_visual_text("Ł"), "•");
        assert_eq!(repair_visual_text(">&"), "(");
        assert_eq!(repair_visual_text(">'"), ")");
        assert_eq!(repair_visual_text("recognized by––"), "recognized by......");
        assert_eq!(
            repair_visual_text("Top- down, Global -wise"),
            "Top-down, Global-wise"
        );
    }

    #[test]
    fn repairs_iec_definition_rms_fragments() {
        assert_eq!(
            repair_visual_text(
                "ratio of the value of the sum of the harmonic components (in this context RMS harmonic"
            ),
            "ratio of the RMS value of the sum of the harmonic components (in this context, harmonic"
        );
        assert_eq!(
            repair_visual_text(
                "current components Ih of orders 2 to RMS40) to thevalue of the fundamental component"
            ),
            "current components Ih of orders 2 to 40) to the RMS value of the fundamental component"
        );
    }
}
