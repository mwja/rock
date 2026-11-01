use crate::fragment::Fragment;
use rock_render::prelude::*;

/// Takes a list of fragments and turns them into a render list for rock_render.
pub fn build_display_list(root: &Fragment, viewport: Size) -> DisplayList {
    let mut list = DisplayList {
        viewport,
        items: Vec::new(),
    };
    paint(root, Point::default(), &mut list);
    list
}

/// Takes a fragment, with a content origin and adds to the render list.
fn paint(f: &Fragment, parent_content_origin: Point, list: &mut DisplayList) {
    let origin = Point {
        // Fragment offset is likely due to margin.
        x: parent_content_origin.x + f.offset.x,
        y: parent_content_origin.y + f.offset.y,
    };
    let rect = Rect {
        origin,
        size: f.size,
    };

    if let Some(color) = f.background {
        list.items.push(DisplayItem::FillRect { rect, color });
    }

    if f.border != Edges::default() {
        list.items.push(DisplayItem::Border {
            rect,
            widths: f.border,
            colors: [f.border_color; 4],
        });
    }

    let content_origin = Point {
        x: origin.x + f.content_offset.x,
        y: origin.y + f.content_offset.y,
    };

    for child in &f.children {
        paint(child, content_origin, list);
    }
}
