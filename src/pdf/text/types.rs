#[derive(Debug, Clone, PartialEq)]
pub struct TextSegment {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub width: f32,
    pub rotation: f32,
    pub role: Option<String>,
    pub color: Option<String>,
    pub font_family: Option<String>,
    pub font_weight: Option<u16>,
    pub font_style: Option<String>,
}

impl TextSegment {
    pub fn new(text: String, x: f32, y: f32, font_size: f32, width: f32) -> Self {
        Self {
            text,
            x,
            y,
            font_size,
            width,
            rotation: 0.0,
            role: None,
            color: None,
            font_family: None,
            font_weight: None,
            font_style: None,
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_role(mut self, role: Option<String>) -> Self {
        self.role = role;
        self
    }

    pub fn with_color(mut self, color: Option<String>) -> Self {
        self.color = color;
        self
    }

    pub fn with_font_style(
        mut self,
        family: Option<String>,
        weight: Option<u16>,
        style: Option<String>,
    ) -> Self {
        self.font_family = family;
        self.font_weight = weight;
        self.font_style = style;
        self
    }
}
