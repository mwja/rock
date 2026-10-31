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
