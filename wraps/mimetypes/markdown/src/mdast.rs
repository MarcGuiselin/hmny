use hmny_common::prelude::*;
use markdown::mdast::*;

struct FontSize;
impl FontSize {
    pub const H1: f32 = 30.0;
    pub const H2: f32 = 27.0;
    pub const H3: f32 = 24.0;
    pub const H4: f32 = 21.0;
    pub const H5: f32 = 18.0;
    pub const H6: f32 = 15.0;
    pub const P: f32 = 16.0;

    fn from_header_depth(depth: u8) -> f32 {
        match depth {
            1 => Self::H1,
            2 => Self::H2,
            3 => Self::H3,
            4 => Self::H4,
            5 => Self::H5,
            _ => Self::H6,
        }
    }
}

const LINE_HEIGHT: f32 = 1.5;

struct Weight;
impl Weight {
    // pub const THIN: u16 = 100;
    // pub const EXTRA_LIGHT: u16 = 200;
    // pub const LIGHT: u16 = 300;
    pub const NORMAL: u16 = (400);
    // pub const MEDIUM: u16 = (500);
    pub const SEMIBOLD: u16 = (600);
    pub const BOLD: u16 = (700);
    // pub const EXTRA_BOLD: u16 = (800);
    // pub const BLACK: u16 = (900);
}

pub fn root_to_dimension(root: Root) -> Result<Dimension, String> {
    let title = root
        .children
        .iter()
        .find_map(|node| match node {
            Node::Heading(heading) if heading.depth == 1 => {
                Some(children_to_string(&heading.children))
            }
            _ => None,
        })
        .unwrap_or("None".into());

    let texts = root
        .children
        .into_iter()
        .map(|node| node_to_entities(node))
        .collect::<Result<Vec<_>, String>>()?;

    Ok(Dimension {
        title,
        children: vec![Element::Canvas(Canvas { texts })],
    })
}

fn node_to_entities(root: Node) -> Result<hmny_common::prelude::Text, String> {
    match root {
        // Parents.
        Node::Root(_) => Err("Root not implemented".into()),
        Node::BlockQuote(_) => Err("BlockQuote not implemented".into()),
        Node::FootnoteDefinition(_) => Err("FootnoteDefinition not implemented".into()),
        Node::MdxJsxFlowElement(_) => Err("MdxJsxFlowElement not implemented".into()),
        Node::List(_) => Err("List not implemented".into()),
        Node::Delete(_) => Err("Delete not implemented".into()),
        Node::Emphasis(_) => Err("Emphasis not implemented".into()),
        Node::MdxJsxTextElement(_) => Err("MdxJsxTextElement not implemented".into()),
        Node::Link(_) => Err("Link not implemented".into()),
        Node::LinkReference(_) => Err("LinkReference not implemented".into()),
        Node::Strong(_) => Err("Strong not implemented".into()),
        Node::Heading(Heading {
            children, depth, ..
        }) => Ok(hmny_common::prelude::Text {
            spans: children_to_text_spans(children, Style::Normal, Weight::SEMIBOLD),
            font_size: FontSize::from_header_depth(depth),
            line_height: LINE_HEIGHT,
            color: TextColor::BLACK,
        }),
        Node::Table(_) => Err("Table not implemented".into()),
        Node::TableRow(_) => Err("TableRow not implemented".into()),
        Node::TableCell(_) => Err("TableCell not implemented".into()),
        Node::ListItem(_) => Err("ListItem not implemented".into()),
        Node::Paragraph(Paragraph { children, .. }) => Ok(hmny_common::prelude::Text {
            spans: children_to_text_spans(children, Style::Normal, Weight::NORMAL),
            font_size: FontSize::P,
            line_height: LINE_HEIGHT,
            color: TextColor::BLACK,
        }),

        // Literals.
        Node::MdxjsEsm(_) => Err("MdxjsEsm not implemented".into()),
        Node::Toml(_) => Err("Toml not implemented".into()),
        Node::Yaml(_) => Err("Yaml not implemented".into()),
        Node::InlineCode(_) => Err("InlineCode not implemented".into()),
        Node::InlineMath(_) => Err("InlineMath not implemented".into()),
        Node::MdxTextExpression(_) => Err("MdxTextExpression not implemented".into()),
        Node::Html(_) => Err("Html not implemented".into()),
        Node::Text(_) => Err("Text not implemented".into()),
        Node::Code(_) => Err("Code not implemented".into()),
        Node::Math(_) => Err("Math not implemented".into()),
        Node::MdxFlowExpression(_) => Err("MdxFlowExpression not implemented".into()),

        // Voids.
        Node::Break(_) => Err("Break not implemented".into()),
        Node::FootnoteReference(_) => Err("FootnoteReference not implemented".into()),
        Node::Image(_) => Err("Image not implemented".into()),
        Node::ImageReference(_) => Err("ImageReference not implemented".into()),
        Node::ThematicBreak(_) => Err("ThematicBreak not implemented".into()),
        Node::Definition(_) => Err("Definition not implemented".into()),
    }
}

fn children_to_string(children: &Vec<Node>) -> String {
    let mut string: String = children
        .iter()
        .map(|node| {
            let mut string = node.to_string();
            if !string.is_empty() {
                string.push_str(" ");
            }
            string
        })
        .collect();
    // Trim last extra space
    string.pop();
    string
}

fn children_to_text_spans(children: Vec<Node>, style: Style, weight: u16) -> Vec<TextSpan> {
    children
        .into_iter()
        .filter_map(|node| match node {
            Node::Text(text) => Some(vec![TextSpan {
                text: text.value,
                color: None,
                style: style.clone(),
                weight,
            }]),
            Node::Strong(strong) => Some(children_to_text_spans(
                strong.children,
                style.clone(),
                Weight::BOLD,
            )),
            Node::Emphasis(emphasis) => Some(children_to_text_spans(
                emphasis.children,
                Style::Italic,
                weight,
            )),
            _ => None,
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_children_to_string() {
        let markdown = "# Hey, *you*!\n\n > this\n   \n is pretty **cool**!\n  1. First item   \n2. Second item \n\n - end";
        let mdast = markdown::to_mdast(markdown, &markdown::ParseOptions::default()).unwrap();
        let result = children_to_string(mdast.children().unwrap());
        assert_eq!(
            result,
            // Due to the way toString is implemented, bullet point items are coalesced without a space
            // between them. This is fine for now
            "Hey, you! this is pretty cool! First itemSecond item end"
        );
    }
}
