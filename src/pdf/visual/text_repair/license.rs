pub(in crate::pdf::visual::text_repair) fn strip_license_artifact_runs(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < chars.len() {
        if is_license_artifact_char(chars[index]) {
            let run_start = index;
            while index < chars.len() && is_license_artifact_char(chars[index]) {
                index += 1;
            }

            let mut next_text = index;
            while next_text < chars.len() && chars[next_text].is_whitespace() {
                next_text += 1;
            }

            if index - run_start >= 12
                && (next_text == chars.len() || chars[next_text].is_alphanumeric())
            {
                if output.trim().is_empty() {
                    output.clear();
                } else if next_text == chars.len() {
                    while output.ends_with(char::is_whitespace) {
                        output.pop();
                    }
                } else if next_text < chars.len() && !output.ends_with(char::is_whitespace) {
                    output.push(' ');
                }
                index = next_text;
                continue;
            }

            for ch in &chars[run_start..index] {
                output.push(*ch);
            }
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

fn is_license_artifact_char(ch: char) -> bool {
    matches!(ch, '`' | ',' | '-' | '\'' | '’' | '“' | '”')
}
