use rock_render::geo::*;

use crate::box_tree::BoxNode;

/// Rendering constraints
#[derive(Clone, Copy)]
pub struct Constraints {
    pub available_width: f32,
}

/// A 'render fragment'. The result of the box node in its layout + style.
pub struct Fragment {
    pub offset: Point, // border-box origin, relative to the parent's content box
    pub size: Size,    // border-box size
    pub content_offset: Point, // border + padding: where this box's children start
    pub background: Option<Color>,
    pub border: Edges,
    pub border_color: Color,
    pub children: Vec<Fragment>,
}

/// Given a box node and constraints, calculates the render/layout fragment.
pub fn layout(node: &BoxNode, c: Constraints) -> Fragment {
    let s = &node.style;
    let h_extra = s.border.left + s.border.right + s.padding.left + s.padding.right;
    let v_extra = s.border.top + s.border.bottom + s.padding.top + s.padding.bottom;

    // Either: take width provided or use the available width and extend until margins
    // and padding are accounted for.
    //
    // This will in the future be in a style engine crate, or separated. For now
    // it's just here.
    let content_w = s
        .width
        .unwrap_or((c.available_width - s.margin.left - s.margin.right - h_extra).max(0.0));

    let mut cursor_y = 0.0;
    let mut children = Vec::with_capacity(node.children.len());
    for child in &node.children {
        let mut f = layout(
            child,
            Constraints {
                available_width: content_w,
            },
        );
        let m = child.style.margin;
        f.offset = Point {
            x: m.left,
            y: cursor_y + m.top,
        };
        cursor_y = f.offset.y + f.size.h + m.bottom;
        children.push(f);
    }

    // Height is either provided height or from children.
    let content_h = s.height.unwrap_or(cursor_y);
    Fragment {
        offset: Point::default(),
        size: Size {
            w: content_w + h_extra,
            h: content_h + v_extra,
        },
        // Margin is included in style calculations above but NOT here otherwise
        // margins wouldn't get background colours. Aka the main reason we
        // have margin AND padding as distinct attributes.
        content_offset: Point {
            x: s.border.left + s.padding.left,
            y: s.border.top + s.padding.top,
        },
        background: s.background,
        border: s.border,
        border_color: s.border_color,
        children,
    }
}
