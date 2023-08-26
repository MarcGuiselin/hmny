use super::*;

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum DataType {
    String(String),
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum CommonQuery {
    AskMetadata,
    Ping { message: String },
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum CommonResponse {
    Metadata(ElementMetdata),
    Pong { response: String },
}

pub type CommonResult = Result<CommonResponse, ElementError>;

pub trait CommonInterface {
    fn common_query(query: CommonQuery) -> CommonResult;
}

impl HarmonySignal for CommonQuery {
    type ResponseType = CommonResponse;
    const QUERY_ID: u64 = 0;
}
