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
