use crate::geo::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayItem {
    /// Solid fill of a rectangle.
    FillRect { rect: Rect, color: Color },

    /// Box border: four sides with their own widths and colors, drawn inside `rect`.
    Border {
        rect: Rect,
        widths: Edges,
        colors: [Color; 4], // top, right, bottom, left
    },

    Text {
        origin: Point,
        text: String,
        font_size: f32,
        line_height: f32,
        max_width: f32,
        color: Color,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayList {
    pub viewport: Size,
    pub items: Vec<DisplayItem>,
}

impl DisplayList {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
