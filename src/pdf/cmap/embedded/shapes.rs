use std::collections::{HashMap, HashSet};

use ttf_parser::{Face, GlyphId, OutlineBuilder};

pub(super) fn shape_inferred_mappings(
    face: &Face<'_>,
    known: &HashMap<u16, String>,
) -> HashMap<u16, String> {
    let known_gids = known.keys().copied().collect::<HashSet<_>>();
    let mut signatures: HashMap<OutlineSignature, Option<String>> = HashMap::new();

    for (gid, text) in known {
        if text.chars().count() != 1 {
            continue;
        }
        let Some(signature) = outline_signature(face, *gid) else {
            continue;
        };
        signatures
            .entry(signature)
            .and_modify(|existing| {
                if existing.as_ref() != Some(text) {
                    *existing = None;
                }
            })
            .or_insert_with(|| Some(text.clone()));
    }

    (0..face.number_of_glyphs())
        .filter(|gid| !known_gids.contains(gid))
        .filter_map(|gid| {
            let signature = outline_signature(face, gid)?;
            let text = signatures.get(&signature)?.clone()?;
            Some((gid, text))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OutlineSignature(Vec<OutlineOp>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OutlineOp {
    Move(i16, i16),
    Line(i16, i16),
    Quad(i16, i16, i16, i16),
    Curve(i16, i16, i16, i16, i16, i16),
    Close,
}

#[derive(Default)]
struct SignatureBuilder {
    ops: Vec<OutlineOp>,
}

impl OutlineBuilder for SignatureBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.ops.push(OutlineOp::Move(q(x), q(y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.ops.push(OutlineOp::Line(q(x), q(y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.ops.push(OutlineOp::Quad(q(x1), q(y1), q(x), q(y)));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.ops
            .push(OutlineOp::Curve(q(x1), q(y1), q(x2), q(y2), q(x), q(y)));
    }

    fn close(&mut self) {
        self.ops.push(OutlineOp::Close);
    }
}

fn outline_signature(face: &Face<'_>, gid: u16) -> Option<OutlineSignature> {
    let mut builder = SignatureBuilder::default();
    face.outline_glyph(GlyphId(gid), &mut builder)?;
    (!builder.ops.is_empty()).then_some(OutlineSignature(builder.ops))
}

fn q(value: f32) -> i16 {
    (value / 8.0)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_unique_shape_matches_only() {
        let mut signatures = HashMap::new();
        let sig = OutlineSignature(vec![OutlineOp::Move(0, 0), OutlineOp::Close]);
        signatures.insert(sig.clone(), Some("A".to_string()));
        signatures.entry(sig).and_modify(|value| {
            if value.as_deref() != Some("B") {
                *value = None;
            }
        });

        assert_eq!(signatures.values().next().unwrap(), &None);
    }
}
