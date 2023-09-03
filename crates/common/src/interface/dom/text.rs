use super::*;

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Text {
    pub spans: Vec<TextSpan>,
    pub font_size: f32,
    pub line_height: f32,
    pub color: TextColor,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct TextColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TextColor {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct TextSpan {
    pub text: String,
    pub color: Option<TextColor>,
    pub style: Style,
    pub weight: u16,
}

impl Default for TextSpan {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: None,
            style: Style::Normal,
            weight: 400,
        }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            spans: vec![],
            font_size: 16.0,
            line_height: 1.5,
            color: TextColor::BLACK,
        }
    }
}
