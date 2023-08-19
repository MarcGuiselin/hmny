use bincode::{Decode, Encode};

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

    /// Whether or not the interface version of an element matches the current version
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
pub enum ElementType {
    None,
    Test,
    HomeScreen,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct RawVectorPtr {
    pub ptr: u64,
    pub len: u64,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct SignalPacket {
    pub version: InterfaceVersion,
    pub payload: Result<RawVectorPtr, ElementError>,
}

impl SignalPacket {
    pub fn new(payload: Result<RawVectorPtr, ElementError>) -> Self {
        Self {
            version: InterfaceVersion(INTERFACE_VERSION.into()),
            payload,
        }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct ElementMetdata {
    pub name: String,
    pub version: String,
    pub element_type: ElementType,
    pub description: String,
    pub publisher: Publisher,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub enum DataType {
    String(String),
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub enum Signal {
    // Generic Signals
    None,
    AskMetadata,
    Metadata(ElementMetdata),
    Ping { message: String },
    Pong { response: String },
    // Home Screen Signals
    AskHomeScreen,
    HomeScreen { mime_type: String, data: DataType },
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub enum ElementError {
    UnsupportedInterfaceVersion(InterfaceVersion),
    UnsupportedSignal,
    DecodeFailed(String),
    EncodeFailed(String),
}

pub type SignalResult = Result<Signal, ElementError>;

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
