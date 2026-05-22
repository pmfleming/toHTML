#[derive(Debug, Clone, PartialEq)]
pub struct TextSegment {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub width: f32,
    pub rotation: f32,
    pub role: Option<String>,
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
}
