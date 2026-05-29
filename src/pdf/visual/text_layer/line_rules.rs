use super::super::super::text::TextLine;
use super::super::text_inference::has_domain_like_text;
use super::needs_inserted_word_gap;

pub(in crate::pdf::visual) fn should_render_line_cells(line: &TextLine) -> bool {
    if has_email_address(&line.text) {
        return true;
    }
    if is_inline_expression_line(line) {
        return false;
    }
    if has_shifted_parenthetical_prose(line) {
        return false;
    }
    if has_broken_hyphenated_prose(line) {
        return false;
    }
    if has_dot_leader_page_number(&line.text) {
        return true;
    }
    if is_bulleted_prose_line(line) {
        return false;
    }
    if is_definition_prose_line(line) && !is_style_fragmented_prose_line(line) {
        return false;
    }
    if has_wide_cell_gap(line) {
        return true;
    }
    if is_style_fragmented_prose_line(line) {
        return true;
    }
    if line.text.chars().count() > 70 {
        return false;
    }
    if line.text.contains('©') {
        return false;
    }
    false
}

pub(in crate::pdf::visual) fn is_definition_prose_line(line: &TextLine) -> bool {
    if !(7.0..=12.5).contains(&line.font_size)
        || line.text.contains('=')
        || line.text.contains('+')
        || has_domain_like_text(&line.text)
        || has_dot_leader_page_number(&line.text)
    {
        return false;
    }
    if !line.text.chars().any(char::is_lowercase) {
        return false;
    }

    let alphabetic_words = line
        .text
        .split_whitespace()
        .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 2)
        .count();
    line.text.chars().count() >= 42 && alphabetic_words >= 6
}

fn has_broken_hyphenated_prose(line: &TextLine) -> bool {
    if line.text.contains('=')
        || !(line.text.contains(" -wise")
            || line.text.contains("- wise")
            || line.text.contains("-wise")
            || line.text.contains(" -down")
            || line.text.contains("- down")
            || line.text.contains("-down"))
    {
        return false;
    }
    line.text
        .split_whitespace()
        .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 3)
        .count()
        >= 3
}

fn is_bulleted_prose_line(line: &TextLine) -> bool {
    let text = line.text.trim_start();
    if !(text.starts_with("• ") || text.starts_with("o ")) {
        return false;
    }
    text.split_whitespace()
        .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 2)
        .count()
        >= 4
        && text.chars().any(char::is_lowercase)
}

fn is_style_fragmented_prose_line(line: &TextLine) -> bool {
    line.cells.len() >= 2
        && line.font_size >= 11.0
        && line.text.chars().any(char::is_lowercase)
        && line
            .text
            .split_whitespace()
            .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 2)
            .count()
            >= 6
        && line
            .cells
            .windows(2)
            .any(|cells| needs_inserted_word_gap(&cells[0], &cells[1]))
}

fn has_shifted_parenthetical_prose(line: &TextLine) -> bool {
    if !line.text.contains(">&") || !line.text.contains(">'") || line.text.contains('=') {
        return false;
    }
    line.text
        .split_whitespace()
        .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 3)
        .count()
        >= 3
}

fn has_dot_leader_page_number(text: &str) -> bool {
    let trimmed = text.trim_end();
    trimmed.contains("...")
        && trimmed
            .split_whitespace()
            .last()
            .is_some_and(|token| token.chars().all(|ch| ch.is_ascii_digit()))
}

fn has_wide_cell_gap(line: &TextLine) -> bool {
    let column_gap = line.font_size.max(8.0) * 2.25;
    line.cells.windows(2).any(|cells| {
        let left_end = cells[0].x + cells[0].width.max(0.0);
        cells[1].x - left_end >= column_gap
    })
}

fn has_email_address(text: &str) -> bool {
    text.split_whitespace().any(|word| {
        let trimmed = word.trim_matches(|ch: char| {
            matches!(ch, ',' | ';' | ':' | '<' | '>' | '(' | ')' | '[' | ']')
        });
        trimmed.contains('@') && trimmed.rsplit_once('.').is_some()
    })
}

fn is_inline_expression_line(line: &TextLine) -> bool {
    line.cells.len() <= 3 && line.text.chars().any(|ch| matches!(ch, '=' | '+'))
}
