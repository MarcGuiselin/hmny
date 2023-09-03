use super::*;

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum MimetypeQuery {
    AskParse { data: DataType },
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum MimetypeResponse {
    Dimension(dom::Dimension),
}

pub type MimetypeResult = Result<MimetypeResponse, WrapError>;

pub trait MimetypeInterface {
    fn mimetype_query(query: MimetypeQuery) -> MimetypeResult;
}

impl HarmonySignal for MimetypeQuery {
    type ResponseType = MimetypeResponse;
    const QUERY_ID: u64 = 2;
}
