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

#[cfg(test)]
mod tests {
    use crate::dom::Dom;

    #[test]
    fn append_text_merges_into_previous_text() {
        let mut dom = Dom::new();
        let root = dom.root();
        let first = dom.append_text(root, "Hello, ".into());
        let second = dom.append_text(root, "World!".into());
        assert_eq!(first, second);
        assert_eq!(dom.text(first), "Hello, World!");
        assert_eq!(dom.children(root).count(), 1);
    }

    #[test]
    fn append_text_after_non_text_creates_new_node() {
        let mut dom = Dom::new();
        let root = dom.root();
        let comment = dom.create_comment("c".into());
        dom.append_child(root, comment);
        let text = dom.append_text(root, "t".into());
        assert_ne!(text, comment);
        assert_eq!(dom.children(root).collect::<Vec<_>>(), [comment, text]);
    }

    #[test]
    fn append_text_before_merges_left_not_right() {
        let mut dom = Dom::new();
        let root = dom.root();
        let left = dom.append_text(root, "X".into());
        let right = dom.create_comment("Y".into());
        dom.append_child(root, right);
        let merged = dom.append_text_before(right, "Z".into());
        assert_eq!(merged, left);
        assert_eq!(dom.text(left), "XZ");
    }
}
