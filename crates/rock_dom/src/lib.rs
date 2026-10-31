//! Manages the DOM, string parsing into a DOM, etc.
pub(crate) mod dom;
pub(crate) mod parser;

pub mod prelude {
    pub use super::dom::{Dom, ElementData, NodeData, NodeId};
    pub use super::parser::create_dom;
}
