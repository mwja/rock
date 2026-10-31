use slotmap::new_key_type;

new_key_type! {
  /// An opaque reference to a node on the DOM.
  pub struct NodeId;
}

/// The relationships a node has
#[derive(Clone, Copy, Default)]
pub struct Links {
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
}
