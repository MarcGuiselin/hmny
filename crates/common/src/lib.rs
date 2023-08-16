pub mod interface;

pub mod prelude {
    pub use super::interface::*;

    pub extern crate bincode;
    pub use bincode::{Decode, Encode};
}
