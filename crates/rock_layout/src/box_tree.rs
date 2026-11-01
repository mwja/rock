use rock_dom::prelude::*;

use crate::style::{Display, Style, style_for};

/// A styled node with children
pub struct BoxNode {
    pub node: Option<NodeId>, // What dom element this box node refers too.
    pub style: Style,
    pub children: Vec<BoxNode>, // block children only, for now
}

pub fn build_box_tree(dom: &Dom) -> Option<BoxNode> {
    let html = dom
        .children(dom.root())
        .find(|&id| dom.try_element(id).is_some())?;
    build_box(dom, html)
}

fn build_box(dom: &Dom, id: NodeId) -> Option<BoxNode> {
    let style = style_for(dom, id);
    match style.display {
        Display::None => None,
        Display::Block | Display::Inline => {
            // Inline treated as Block for now
            let children = dom
                .children(id)
                .filter_map(|child| build_box(dom, child))
                .collect();
            Some(BoxNode {
                node: Some(id),
                style,
                children,
            })
        }
    }
}
