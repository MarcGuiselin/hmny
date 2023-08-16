use bincode::{Decode, Encode};

pub const INTERFACE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct InterfaceVersion(String);

impl InterfaceVersion {
    pub fn new(version: &str) -> Self {
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

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub enum ElementType {
    None,
    Test,
    // These can be renamed in the future
    Unknown,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub struct SignalPacket {
    pub version: InterfaceVersion,
    pub element_type: ElementType,
    pub payload: Result<Vec<u8>, ElementError>,
}

impl SignalPacket {
    pub fn new(element_type: ElementType, payload: Result<Vec<u8>, ElementError>) -> Self {
        Self {
            version: InterfaceVersion(INTERFACE_VERSION.into()),
            element_type,
            payload,
        }
    }
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub enum Signal {
    None,
    // AskMetadata,
    // Metadata {
    //     name: String,
    //     version: InterfaceVersion,
    //     element_type: ElementType,
    //     description: String,
    //     publisher: Publisher,
    // },
    Ping { message: String },
    Pong { response: String },
}

#[derive(Clone, Decode, Encode, PartialEq, Debug, Eq)]
pub enum ElementError {
    UnsupportedInterfaceVersion(InterfaceVersion),
    UnsupportedSignal,
    DecodeFailed(String),
    EncodeFailed(String),
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
