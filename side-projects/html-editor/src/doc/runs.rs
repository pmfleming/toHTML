use super::{InlineStyle, StyledChar};

/// Group consecutive styled chars with identical style into (text, style) runs.
pub fn group_runs(runs: &[StyledChar]) -> Vec<(String, InlineStyle)> {
    let mut out: Vec<(String, InlineStyle)> = Vec::new();
    for sc in runs {
        if let Some(last) = out.last_mut() {
            if last.1 == sc.style {
                last.0.push(sc.ch);
                continue;
            }
        }
        out.push((sc.ch.to_string(), sc.style.clone()));
    }
    if out.is_empty() {
        out.push((String::new(), InlineStyle::default()));
    }
    out
}
