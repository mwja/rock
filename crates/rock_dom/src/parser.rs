use std::io;

use html5ever::{parse_document, tendril::TendrilSink};
use thiserror::Error;

use crate::dom::Dom;

mod sink;

#[derive(Debug, Error)]
pub enum DomParserError {
    #[error(transparent)]
    IoError(#[from] io::Error),
}

pub fn create_dom(body: &str) -> Result<Dom, DomParserError> {
    let dom = parse_document(sink::DomSink::new(Dom::new()), Default::default())
        .from_utf8()
        .read_from(&mut body.as_bytes())?;

    #[cfg(debug_assertions)]
    check_invariants(&dom);

    Ok(dom)
}

#[cfg(debug_assertions)]
fn check_invariants(dom: &Dom) {
    for id in dom.ids() {
        let l = dom.links(id).expect("link in invariant check");
        if let Some(p) = l.parent {
            // I appear in my parent's child list exactly once
            assert_eq!(dom.children(p).filter(|&c| c == id).count(), 1);
        }
        if let Some(n) = l.next_sibling {
            assert_eq!(
                dom.links(n).expect("link in invariant check").prev_sibling,
                Some(id)
            );
            assert_eq!(
                dom.links(n).expect("link in invariant check").parent,
                l.parent
            );
        }
        if let Some(p) = l.prev_sibling {
            assert_eq!(
                dom.links(p).expect("link in invariant check").next_sibling,
                Some(id)
            );
        }
        if l.first_child.is_none() {
            assert!(l.last_child.is_none());
        }
        // no two adjacent Text children (the spec requires merging)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::NodeId;

    /// Find the first element called `name` (depth first).
    fn find(dom: &Dom, from: NodeId, name: &str) -> Option<NodeId> {
        if dom
            .try_element(from)
            .is_some_and(|e| &*e.name.local == name)
        {
            return Some(from);
        }
        dom.children(from).find_map(|c| find(dom, c, name))
    }

    #[test]
    fn creates_implied_html_head_body() {
        let dom = create_dom("hello").unwrap();
        let html = find(&dom, dom.root(), "html").unwrap();
        let names: Vec<_> = dom
            .children(html)
            .map(|c| dom.element(c).name.local.to_string())
            .collect();
        assert_eq!(names, ["head", "body"]);
    }

    #[test]
    fn adjacent_text_becomes_one_node() {
        // the tokenizer emits "a", "&", "b" separately; they should merge
        let dom = create_dom("<p>a&amp;b</p>").unwrap();
        let p = find(&dom, dom.root(), "p").unwrap();
        let kids: Vec<_> = dom.children(p).collect();
        assert_eq!(kids.len(), 1);
        assert_eq!(dom.text(kids[0]), "a&b");
    }

    #[test]
    fn template_children_go_into_its_fragment() {
        let dom = create_dom("<template><p>x</p></template>").unwrap();
        let template = find(&dom, dom.root(), "template").unwrap();
        assert_eq!(dom.children(template).count(), 0);
        let frag = dom.element(template).template_contents.unwrap();
        assert!(find(&dom, frag, "p").is_some());
    }
}
