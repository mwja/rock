#![allow(clippy::unwrap_used, reason = "test suite")]

#[cfg(test)]
use html5ever::{QualName, ns};
use rock_dom::prelude::*;

#[cfg(test)]
fn display_name(n: &QualName) -> String {
    match n.ns {
        ns!(svg) => format!("svg {}", n.local),
        ns!(mathml) => format!("math {}", n.local),
        _ => n.local.to_string(), // HTML, and anything unexpected
    }
}

/// Dumps into a snapshottable string.
#[cfg(test)]
pub fn dump(dom: &Dom) -> String {
    use std::fmt::Write as _;

    fn go(dom: &Dom, id: NodeId, depth: usize, out: &mut String) {
        let pad = "  ".repeat(depth);
        match dom.node(id).unwrap() {
            NodeData::Element(e) => {
                // namespaced elements (svg, math) print as `<svg path>`, plain HTML as `<p>`
                writeln!(out, "| {pad}<{}>", display_name(&e.name)).unwrap();
                let mut attrs: Vec<_> = e.attrs.iter().collect();
                attrs.sort_by_key(|a| display_name(&a.name)); // the format sorts attributes
                for a in attrs {
                    writeln!(out, "| {pad}  {}=\"{}\"", display_name(&a.name), a.value).unwrap();
                }
                if let Some(frag) = e.template_contents {
                    writeln!(out, "| {pad}  content").unwrap();
                    for c in dom.children(frag) {
                        go(dom, c, depth + 2, out);
                    }
                } else {
                    for c in dom.children(id) {
                        go(dom, c, depth + 1, out);
                    }
                }
            }
            NodeData::Text(t) => writeln!(out, "| {pad}\"{t}\"").unwrap(),
            NodeData::Comment(c) => writeln!(out, "| {pad}<!-- {c} -->").unwrap(),
            NodeData::Doctype { name, .. } => writeln!(out, "| {pad}<!DOCTYPE {name}>").unwrap(),
            _ => {}
        }
    }
    let mut out = String::new();
    for c in dom.children(dom.root()) {
        go(dom, c, 0, &mut out);
    }
    out
}
