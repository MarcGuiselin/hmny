use super::*;

mod common;
pub use common::*;

pub trait HarmonySignal: Sized + Decode + Encode {
    type ResponseType: Decode + Encode;
    const QUERY_ID: u64;
}

#[cfg(feature = "homescreen")]
mod homescreen;
#[cfg(feature = "homescreen")]
pub use homescreen::*;

#[cfg(feature = "mimetype")]
mod mimetype;
#[cfg(feature = "mimetype")]
pub use mimetype::*;
