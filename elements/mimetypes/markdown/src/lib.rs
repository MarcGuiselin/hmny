use hmny_common::prelude::*;
use markdown::{mdast::Node, to_mdast};

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

fn children_to_string(children: &[Node]) -> String {
    children.iter().map(ToString::to_string).collect()
}

fn parse(data: DataType) -> MimetypeResult {
    // Markdown must be string
    let data = match data {
        DataType::String(data) => data,
        // _ => return Err("Invalid data type".into()),
    };

    // Parse markdown
    let mdast = to_mdast(&data, &markdown::ParseOptions::default()).map_err(|e| e.into())?;

    // Obtain title from markdown
    let mut title = String::from("None");
    match mdast {
        Node::Root(root) => {
            for child in root.children {
                match child {
                    Node::Heading(heading) if heading.depth == 1 => {
                        title = children_to_string(&heading.children[..]);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    // Convert mdast to dimension
    Ok(MimetypeResponse::Dimension(Dimension {
        title,
        children: vec![Entity {
            label: None,
            components: vec![
                Component::Text(Text {
                    text: "Test Text!".into(),
                    ..Default::default()
                }),
                Component::Location2D(Location2D::default()),
            ],
        }],
    }))
}
