use hmny_common::prelude::*;

#[define_element{
    publisher: Publisher::new("Harmony", vec![]),
    element_type: ElementType::HomeScreen,
}]
struct HomescreenElement(CommonQuery, HomescreenQuery);

impl HomescreenInterface for HomescreenElement {
    fn homescreen_query(query: HomescreenQuery) -> HomescreenResult {
        match query {
            HomescreenQuery::AskHomeScreen => Ok(HomescreenResponse::HomeScreen {
                mime_type: "markdown".into(),
                data: DataType::String(include_str!("../homescreen.md").into()),
            }),
        }
    }
}
