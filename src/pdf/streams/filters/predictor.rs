use crate::ConvertError;

#[derive(Debug, Clone, Default)]
pub(in crate::pdf::streams::filters) struct DecodeParams {
    pub(in crate::pdf::streams::filters) predictor: i64,
    pub(in crate::pdf::streams::filters) colors: usize,
    pub(in crate::pdf::streams::filters) bits_per_component: usize,
    pub(in crate::pdf::streams::filters) columns: usize,
}

pub(in crate::pdf::streams::filters) fn apply_predictor(
    data: &[u8],
    params: &Option<DecodeParams>,
) -> Result<Vec<u8>, ConvertError> {
    let Some(params) = params else {
        return Ok(data.to_vec());
    };
    match params.predictor {
        0 | 1 => Ok(data.to_vec()),
        2 => tiff_predictor(data, params),
        10..=15 => png_predictor(data, params),
        predictor => Err(ConvertError::Pdf(format!(
            "unsupported PDF stream predictor {predictor}"
        ))),
    }
}

fn row_len(params: &DecodeParams) -> Result<usize, ConvertError> {
    if params.bits_per_component != 8 {
        return Err(ConvertError::Pdf(format!(
            "unsupported predictor bits per component {}",
            params.bits_per_component
        )));
    }
    params
        .columns
        .checked_mul(params.colors)
        .filter(|value| *value > 0)
        .ok_or_else(|| ConvertError::Pdf("invalid predictor columns/colors".to_string()))
}

pub(in crate::pdf::streams::filters) fn tiff_predictor(
    data: &[u8],
    params: &DecodeParams,
) -> Result<Vec<u8>, ConvertError> {
    let row_len = row_len(params)?;
    let mut output = data.to_vec();
    for row in output.chunks_mut(row_len) {
        for index in params.colors..row.len() {
            row[index] = row[index].wrapping_add(row[index - params.colors]);
        }
    }
    Ok(output)
}

fn png_predictor(data: &[u8], params: &DecodeParams) -> Result<Vec<u8>, ConvertError> {
    let row_len = row_len(params)?;
    let stride = row_len + 1;
    if data.len() % stride != 0 {
        return Err(ConvertError::Pdf(
            "invalid PNG predictor row length".to_string(),
        ));
    }
    let mut output = Vec::with_capacity(data.len() / stride * row_len);
    let mut previous = vec![0u8; row_len];
    for row in data.chunks_exact(stride) {
        let filter = row[0];
        let mut current = row[1..].to_vec();
        for index in 0..row_len {
            let left = index
                .checked_sub(params.colors)
                .and_then(|left| current.get(left))
                .copied()
                .unwrap_or(0);
            let up = previous[index];
            let upper_left = index
                .checked_sub(params.colors)
                .and_then(|left| previous.get(left))
                .copied()
                .unwrap_or(0);
            current[index] = current[index].wrapping_add(match filter {
                0 => 0,
                1 => left,
                2 => up,
                3 => ((u16::from(left) + u16::from(up)) / 2) as u8,
                4 => paeth(left, up, upper_left),
                _ => {
                    return Err(ConvertError::Pdf(format!(
                        "unsupported PNG predictor row filter {filter}"
                    )))
                }
            });
        }
        output.extend_from_slice(&current);
        previous = current;
    }
    Ok(output)
}

fn paeth(left: u8, up: u8, upper_left: u8) -> u8 {
    let left = i32::from(left);
    let up = i32::from(up);
    let upper_left = i32::from(upper_left);
    let estimate = left + up - upper_left;
    let pa = (estimate - left).abs();
    let pb = (estimate - up).abs();
    let pc = (estimate - upper_left).abs();
    if pa <= pb && pa <= pc {
        left as u8
    } else if pb <= pc {
        up as u8
    } else {
        upper_left as u8
    }
}
