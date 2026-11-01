use rock_dom::prelude::*;

use crate::style::{Display, Style, style_for};

pub enum BoxKind {
    Block,
    Text(String),
}

pub struct BoxNode {
    pub node: Option<NodeId>,
    pub kind: BoxKind,
    pub style: Style,
    pub children: Vec<BoxNode>,
}

pub fn build_box_tree(dom: &Dom) -> Option<BoxNode> {
    let html = dom
        .children(dom.root())
        .find(|&id| dom.try_element(id).is_some())?;
    build_element_box(dom, html)
}

fn build_box(dom: &Dom, id: NodeId, parent_style: &Style) -> Option<BoxNode> {
    match dom.node(id) {
        Some(NodeData::Element(_)) => build_element_box(dom, id),
        Some(NodeData::Text(text)) => build_text_box(id, text, parent_style),
        _ => None, // comments, doctypes, etc. generate no box
    }
}

fn build_element_box(dom: &Dom, id: NodeId) -> Option<BoxNode> {
    let style = style_for(dom, id);
    if matches!(style.display, Display::None) {
        return None;
    }

    // Inline elements are treated as blocks for now
    let children = dom
        .children(id)
        .filter_map(|child| build_box(dom, child, &style))
        .collect();
    Some(BoxNode {
        node: Some(id),
        kind: BoxKind::Block,
        style,
        children,
    })
}

fn build_text_box(id: NodeId, text: &str, parent_style: &Style) -> Option<BoxNode> {
    let collapsed = collapse_whitespace(text);
    if collapsed.is_empty() {
        return None; // whitespace-only text between tags: no box
    }
    Some(BoxNode {
        node: Some(id),
        kind: BoxKind::Text(collapsed),
        style: text_style(parent_style),
        children: Vec::new(),
    })
}

fn collapse_whitespace(s: &str) -> String {
    s.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}

/// Text inherits only inherited properties (font, color). Box properties reset.
/// Again, will replace with a better styling system later.
fn text_style(parent: &Style) -> Style {
    Style {
        display: Display::Inline,
        font_size: parent.font_size,
        line_height: parent.line_height,
        color: parent.color,
        ..Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rock_render::geo::Edges;

    fn boxes(html: &str) -> BoxNode {
        build_box_tree(&create_dom(html).unwrap()).unwrap()
    }

    /// All text in the box tree, depth first.
    fn texts(b: &BoxNode) -> Vec<String> {
        let mut out = Vec::new();
        if let BoxKind::Text(t) = &b.kind {
            out.push(t.clone());
        }
        for c in &b.children {
            out.extend(texts(c));
        }
        out
    }

    #[test]
    fn root_box_is_html_and_head_is_skipped() {
        // html's only child box should be body, head is display: none
        let root = boxes("<title>t</title><p>x</p>");
        assert!(matches!(root.kind, BoxKind::Block));
        assert_eq!(root.children.len(), 1);
        assert_eq!(texts(&root), ["x"]);
    }

    #[test]
    fn whitespace_only_text_makes_no_box() {
        let root = boxes("<div>\n   <p>a</p>\n   </div>");
        let body = &root.children[0];
        let div = &body.children[0];
        assert_eq!(div.children.len(), 1); // just the <p>
    }

    #[test]
    fn text_whitespace_is_collapsed() {
        assert_eq!(texts(&boxes("<p>  a \n\t b  </p>")), ["a b"]);
    }

    #[test]
    fn text_inherits_font_from_parent() {
        let root = boxes("<p>x</p>");
        let p = &root.children[0].children[0];
        let text = &p.children[0];
        assert_eq!(text.style.font_size, p.style.font_size);
        assert_eq!(text.style.color, p.style.color);
        assert_eq!(text.style.margin, Edges::default()); // box props don't inherit
    }
}
