use hmny_common::prelude::*;

#[define_wrap{
    publisher: Publisher::new("Harmony", vec![]),
    wrap_type: WrapType::HomeScreen,
}]
struct HomescreenWrap(CommonQuery, HomescreenQuery);

impl HomescreenInterface for HomescreenWrap {
    fn homescreen_query(query: HomescreenQuery) -> HomescreenResult {
        match query {
            HomescreenQuery::AskHomeScreen => Ok(HomescreenResponse::HomeScreen {
                mime_type: "markdown".into(),
                data: DataType::String(include_str!("../homescreen.md").into()),
            }),
        }
    }
}
