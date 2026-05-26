#[derive(Debug)]
pub(super) enum Operand {
    Name(String),
    Text(DecodedText),
    ActualText(DecodedText),
    TextArray(Vec<TextArrayItem>),
    Number(f32),
    Mcid(u32),
}

#[derive(Debug, Clone)]
pub(super) enum TextArrayItem {
    Text(DecodedText),
    Adjustment(f32),
}

#[derive(Debug, Clone)]
pub(super) struct DecodedText {
    pub(super) text: String,
    pub(super) raw: Vec<u8>,
}

#[derive(Debug, Default)]
pub(super) struct OperandStack {
    operands: Vec<Operand>,
}

impl OperandStack {
    pub(super) fn push(&mut self, operand: Operand) {
        self.operands.push(operand);
    }

    pub(super) fn clear(&mut self) {
        self.operands.clear();
    }

    pub(super) fn latest_text(&self) -> Option<DecodedText> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::Text(text) => Some(text.clone()),
                _ => None,
            })
    }

    pub(super) fn latest_array(&self) -> Option<Vec<TextArrayItem>> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::TextArray(items) => Some(items.clone()),
                _ => None,
            })
    }

    pub(super) fn latest_number(&self) -> Option<f32> {
        self.operands.iter().rev().find_map(number_operand)
    }

    pub(super) fn latest_two_numbers(&self) -> Option<(f32, f32)> {
        let values = self.numbers();
        let last = values.len().checked_sub(1)?;
        Some((values[last - 1], values[last]))
    }

    pub(super) fn latest_six_numbers(&self) -> Option<[f32; 6]> {
        let values = self.numbers();
        last_six_numbers(&values)
    }

    pub(super) fn latest_name(&self) -> Option<String> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::Name(name) => Some(name.clone()),
                _ => None,
            })
    }

    pub(super) fn latest_actual_text(&self) -> Option<DecodedText> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::ActualText(text) => Some(text.clone()),
                _ => None,
            })
    }

    pub(super) fn latest_mcid(&self) -> Option<u32> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::Mcid(mcid) => Some(*mcid),
                _ => None,
            })
    }

    pub(super) fn numbers(&self) -> Vec<f32> {
        self.operands.iter().filter_map(number_operand).collect()
    }
}

fn number_operand(operand: &Operand) -> Option<f32> {
    match operand {
        Operand::Number(number) => Some(*number),
        _ => None,
    }
}

fn last_six_numbers(values: &[f32]) -> Option<[f32; 6]> {
    let values = values.get(values.len().checked_sub(6)?..)?;
    Some([
        values[0], values[1], values[2], values[3], values[4], values[5],
    ])
}

pub(super) fn push_array_text(
    text: &mut DecodedText,
    value: &DecodedText,
    pending_space: &mut bool,
) {
    if *pending_space && needs_inserted_space(&text.text, &value.text) {
        text.text.push(' ');
        text.raw.push(b' ');
    }
    text.text.push_str(&value.text);
    text.raw.extend_from_slice(&value.raw);
    *pending_space = false;
}

fn needs_inserted_space(text: &str, value: &str) -> bool {
    !text.ends_with(' ') && !value.starts_with(' ') && !text.is_empty() && !value.is_empty()
}
