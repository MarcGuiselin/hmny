use super::*;

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum HomescreenQuery {
    AskHomeScreen,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
pub enum HomescreenResponse {
    HomeScreen { mime_type: String, data: DataType },
}

pub type HomescreenResult = Result<HomescreenResponse, WrapError>;

pub trait HomescreenInterface {
    fn homescreen_query(query: HomescreenQuery) -> HomescreenResult;
}

impl HarmonySignal for HomescreenQuery {
    type ResponseType = HomescreenResponse;
    const QUERY_ID: u64 = 1;
}
