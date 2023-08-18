pub mod interface;

pub mod prelude {
    pub use super::interface::*;
    pub use hmny_macros::*;

    pub extern crate bincode;
    pub use bincode::{Decode, Encode};
}
