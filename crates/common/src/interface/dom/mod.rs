use super::*;
pub use std::sync::Arc;

mod location;
pub use location::*;

mod text;
pub use text::*;

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Dimension {
    pub title: String,
    pub children: Vec<Entity>,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub struct Entity {
    pub label: Option<String>,
    pub components: Vec<Component>,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum Component {
    Location2D(Location2D),
    Location3D(Location3D),
    Text(Text),
}
