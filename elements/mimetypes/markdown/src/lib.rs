use hmny_common::prelude::*;
use markdown::mdast::Node;
mod mdast;

#[define_element{
    publisher: Publisher::new("Harmony", vec![]),
    element_type: ElementType::Mimetype("markdown".into()),
}]
struct HomescreenElement(CommonQuery, MimetypeQuery);

impl MimetypeInterface for HomescreenElement {
    fn mimetype_query(query: MimetypeQuery) -> MimetypeResult {
        match query {
            MimetypeQuery::AskParse { data } => parse(data),
        }
    }
}

fn parse(data: DataType) -> MimetypeResult {
    // Markdown must be string
    let data = match data {
        DataType::String(data) => data,
        // _ => return Err("Invalid data type".into()),
    };

    // Parse markdown and produce dimension
    let dimension = markdown::to_mdast(&data, &markdown::ParseOptions::default())
        .and_then(|mdast| match mdast {
            Node::Root(root) => mdast::root_to_dimension(root),
            _ => Err("Expected root of markdown to be root".into()),
        })
        .map_err(|e| e.into())?;

    // Convert mdast to dimension
    Ok(MimetypeResponse::Dimension(dimension))
}
