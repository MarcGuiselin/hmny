use hmny_common::prelude::*;
use markdown::mdast::*;

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

    let children = root
        .children
        .into_iter()
        .map(|node| node_to_entities(node))
        .collect::<Result<Vec<Vec<Entity>>, String>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(Dimension { title, children })
}

fn node_to_entities(root: Node) -> Result<Vec<Entity>, String> {
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
        Node::Heading(_) => Err("Heading not implemented".into()),
        Node::Table(_) => Err("Table not implemented".into()),
        Node::TableRow(_) => Err("TableRow not implemented".into()),
        Node::TableCell(_) => Err("TableCell not implemented".into()),
        Node::ListItem(_) => Err("ListItem not implemented".into()),
        Node::Paragraph(_) => Err("Paragraph not implemented".into()),

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
