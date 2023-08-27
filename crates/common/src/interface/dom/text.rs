use super::*;

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Text {
    pub text: String,
    pub size: f32,
    pub color: String,
    pub location: Location3D,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            size: 16.0,
            color: "#000000".into(),
            location: Location3D::default(),
        }
    }
}
