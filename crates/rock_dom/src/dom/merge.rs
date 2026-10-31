//! Provides functions that instead of directly appending children, it attempts
//! to optimise by checking the would-be siblings and merging them into a
//! singular node.

use crate::dom::{Dom, NodeId, data::NodeData};

impl Dom {
    pub fn append_text(&mut self, parent: NodeId, src: String) -> NodeId {
        let last = self.links[parent].last_child;

        if let Some(last) = last
            && let NodeData::Text(dst) = &mut self.data[last]
        {
            dst.push_str(&src);
            return last;
        }

        // Create the text node on behalf of the callee
        let text_node = self.create_text(src);
        self.append_child(parent, text_node);
        text_node
    }

    pub fn append_text_before(&mut self, sibling: NodeId, src: String) -> NodeId {
        // Left merge, so:
        // X Y **Z** (Z is the new text node)
        // We are told: append before Y. So we need to check X. **NOT** Y.
        let prev = self.links[sibling].prev_sibling;

        if let Some(prev) = prev
            && let NodeData::Text(dst) = &mut self.data[prev]
        {
            dst.push_str(&src);
            return prev;
        }

        // Create the text node on behalf of the callee
        let text_node = self.create_text(src);
        self.append_before(sibling, text_node);
        text_node
    }
}
