#![allow(clippy::indexing_slicing, reason = "readability, and some certainty")]
use html5ever::interface::QuirksMode;
use slotmap::{SlotMap, SparseSecondaryMap};

mod data;
mod merge;
mod node;

pub use data::{ElementData, NodeData};
pub use node::NodeId;
use tracing::trace;

use crate::dom::node::Links;

/// The DOM, the central part of the managament of the page.
///
/// The vast majority of this code (rn) is managing the relationships between
/// nodes and the links they have
pub struct Dom {
    /// #root
    root: NodeId,
    // The DOM structure is the most important part.
    links: SparseSecondaryMap<NodeId, node::Links>,
    // Making this a sparse map gives us the side effect of cleaning up old
    // data when we don't need it from orphaned DOM elements.
    data: SlotMap<NodeId, data::NodeData>,
}

impl Dom {
    /// Create a new DOM with a singular document node.
    pub fn new() -> Self {
        // Create the root document node.
        let mut data = SlotMap::<NodeId, data::NodeData>::with_key();
        let mut links = SparseSecondaryMap::<NodeId, node::Links>::new();
        let root = data.insert(data::NodeData::Document {
            quirks_mode: QuirksMode::NoQuirks,
        });
        links.insert(root, Default::default());

        Self { root, data, links }
    }
}

impl Dom {
    pub fn ids(&self) -> Box<[NodeId]> {
        self.data.keys().collect()
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn set_quirks_mode(&mut self, mode: QuirksMode) {
        match self.data.get_mut(self.root) {
            Some(NodeData::Document { quirks_mode }) => *quirks_mode = mode,
            _ => unreachable!("root node will never be non-document"),
        }
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.links[id].parent
    }

    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.links[id].next_sibling
    }

    pub fn create_node(&mut self, data: NodeData) -> NodeId {
        let id = self.data.insert(data);
        self.links.insert(id, Default::default());
        id
    }

    pub fn node(&self, id: NodeId) -> Option<&NodeData> {
        self.data.get(id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.data.get_mut(id)
    }

    /// Only a read-only view is provided as manually manipulating links
    /// will likely break stuff.
    pub fn links(&self, id: NodeId) -> Option<&Links> {
        self.links.get(id)
    }

    /// Gets the first_child then traverses next_siblings (linked list style)
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let mut cur = self.links[id].first_child;
        std::iter::from_fn(move || {
            let this = cur?;
            cur = self.links[this].next_sibling;
            Some(this)
        })
    }

    /// Append a node to the end of the sibling chain of the given parent.
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        trace!("append child {child:?} to {parent:?}");
        self.detach(child); // a node can only have one parent

        let old_last = self.links[parent].last_child;
        {
            let c = &mut self.links[child];
            c.parent = Some(parent);
            c.prev_sibling = old_last;
            c.next_sibling = None;
        }
        match old_last {
            Some(last) => self.links[last].next_sibling = Some(child),
            None => self.links[parent].first_child = Some(child),
        }
        self.links[parent].last_child = Some(child);
    }

    pub fn append_before(&mut self, sibling: NodeId, child: NodeId) {
        self.detach(child); // a node can only have one parent

        // move right sibling to refer to this as prev_sibling and prev_sibling to this
        // as next_sibling
        let parent = self.links[sibling].parent;
        let old_prev_sibling = self.links[sibling].prev_sibling;
        {
            let c = &mut self.links[child];
            // Put this node in the place of the other one, and make the sibling
            // the next node
            c.parent = parent;
            c.prev_sibling = old_prev_sibling;
            c.next_sibling = Some(sibling);
        }
        // Make the next node (sibling) point to the new node as the previous node.
        self.links[sibling].prev_sibling = Some(child);
        match (old_prev_sibling, parent) {
            (Some(prev_sibling), _) => {
                self.links[prev_sibling].next_sibling = Some(child);
            }
            (None, Some(parent)) => self.links[parent].first_child = Some(child),
            (None, None) => {}
        }
    }

    /// Detach a node from its parent and update its siblings to no longer see
    /// it.
    pub fn detach(&mut self, id: NodeId) {
        let node::Links {
            parent,
            prev_sibling,
            next_sibling,
            ..
        } = self.links[id];

        if let Some(prev) = prev_sibling {
            self.links[prev].next_sibling = next_sibling;
        } else if let Some(p) = parent {
            self.links[p].first_child = next_sibling;
        }

        if let Some(next) = next_sibling {
            self.links[next].prev_sibling = prev_sibling;
        } else if let Some(p) = parent {
            self.links[p].last_child = prev_sibling;
        }

        self.links[id] = node::Links::default();
    }

    /// Detach a node from the DOM and destroy its data.
    pub fn destroy(&mut self, id: NodeId) {
        self.detach(id);
        self.links.remove(id);
        self.data.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A root with three text children `a`, `b`, `c`, in that order.
    fn abc() -> (Dom, NodeId, [NodeId; 3]) {
        let mut dom = Dom::new();
        let root = dom.root();
        let kids = ["a", "b", "c"].map(|t| dom.create_text(t.into()));
        for k in kids {
            dom.append_child(root, k);
        }
        (dom, root, kids)
    }

    #[test]
    fn append_child_keeps_document_order() {
        let (dom, root, [a, b, c]) = abc();
        assert_eq!(dom.children(root).collect::<Vec<_>>(), [a, b, c]);
        assert_eq!(dom.parent(b), Some(root));
        assert_eq!(dom.next_sibling(a), Some(b));
    }

    #[test]
    fn append_before_first_child_becomes_first() {
        let (mut dom, root, [a, b, c]) = abc();
        let z = dom.create_text("z".into());
        dom.append_before(a, z);
        assert_eq!(dom.children(root).collect::<Vec<_>>(), [z, a, b, c]);
        assert_eq!(dom.links(root).unwrap().first_child, Some(z));
    }

    #[test]
    fn detach_relinks_siblings() {
        let (mut dom, root, [a, b, c]) = abc();
        dom.detach(b);
        assert_eq!(dom.children(root).collect::<Vec<_>>(), [a, c]);
        assert_eq!(dom.parent(b), None);
        assert_eq!(dom.links(c).unwrap().prev_sibling, Some(a));
    }

    #[test]
    fn append_child_moves_node_from_old_parent() {
        let (mut dom, root, [a, b, c]) = abc();
        // move `c` under `a`
        dom.append_child(a, c);
        assert_eq!(dom.children(root).collect::<Vec<_>>(), [a, b]);
        assert_eq!(dom.children(a).collect::<Vec<_>>(), [c]);
        assert_eq!(dom.links(root).unwrap().last_child, Some(b));
    }
}
