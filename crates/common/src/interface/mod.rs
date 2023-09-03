use bincode::{Decode, Encode};

mod dom;
pub use dom::*;

mod signal;
pub use signal::*;

pub const INTERFACE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct InterfaceVersion(String);

impl InterfaceVersion {
    pub fn new() -> Self {
        Self(INTERFACE_VERSION.into())
    }

    pub fn from(version: &str) -> Self {
        Self(version.into())
    }

    /// Whether or not the interface version of an wrap matches the current version
    pub fn matches_own(&self) -> bool {
        self.0 == INTERFACE_VERSION
    }
}

impl Into<String> for InterfaceVersion {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq, Hash)]
pub enum WrapType {
    None,
    Test,
    HomeScreen,
    Mimetype(String),
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct RawVectorPtr {
    pub ptr: u64,
    pub len: u64,
}

impl RawVectorPtr {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr as *const u8, self.len as usize) }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct WrapMetdata {
    pub name: String,
    pub version: String,
    pub wrap_type: WrapType,
    pub description: String,
    pub publisher: Publisher,
    pub interface_version: InterfaceVersion,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub enum WrapError {
    UnsupportedSignal,
    DecodeFailed(String),
    EncodeFailed(String),
    UnsupportedInterface(u64),
    Other(String),
}

impl Into<WrapError> for String {
    fn into(self) -> WrapError {
        WrapError::Other(self)
    }
}

impl Into<WrapError> for &str {
    fn into(self) -> WrapError {
        WrapError::Other(self.into())
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct Publisher {
    name: String,
    signed_by: Vec<Publisher>,
}

impl Publisher {
    pub fn new(name: &str, signed_by: Vec<Publisher>) -> Self {
        Self {
            name: name.into(),
            signed_by,
        }
    }
}

impl Into<Publisher> for String {
    fn into(self) -> Publisher {
        Publisher {
            name: self.into(),
            signed_by: vec![],
        }
    }
}

impl Into<Publisher> for &str {
    fn into(self) -> Publisher {
        Publisher {
            name: self.into(),
            signed_by: vec![],
        }
    }
}
