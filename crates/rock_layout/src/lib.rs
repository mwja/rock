//! Layouting for the DOM, calculates where what goes (including styling etc).
pub mod box_tree;
pub mod fragment;
mod paint;
mod style;

pub mod prelude {
    pub use super::box_tree::*;
    pub use super::fragment::*;
    pub use super::paint::build_display_list;
    pub use super::style::{Display, Style};
}
