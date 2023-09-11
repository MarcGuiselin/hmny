use super::*;
pub use std::sync::Arc;

mod location;
pub use location::*;

mod text;
pub use text::*;

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Dimension {
    pub title: String,
    pub children: Vec<Element>,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum Element {
    Canvas(Canvas),
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Canvas {
    pub texts: Vec<Text>,
}
