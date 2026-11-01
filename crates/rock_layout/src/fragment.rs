use rock_render::geo::*;
use rock_render::text::TextSystem;

use crate::box_tree::{BoxKind, BoxNode};

#[derive(Clone, Copy)]
pub struct Constraints {
    pub available_width: f32,
}

/// What paint needs to draw a text fragment.
pub struct TextFragment {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
}

pub struct Fragment {
    pub offset: Point,
    pub size: Size,
    pub content_offset: Point,
    pub background: Option<Color>,
    pub border: Edges,
    pub border_color: Color,
    pub text: Option<TextFragment>, // Some for text boxes, None for blocks
    pub children: Vec<Fragment>,
}

pub fn layout(node: &BoxNode, c: Constraints, ts: &mut TextSystem) -> Fragment {
    match &node.kind {
        BoxKind::Block => layout_block(node, c, ts),
        BoxKind::Text(text) => layout_text(node, text, c, ts),
    }
}

fn layout_block(node: &BoxNode, c: Constraints, ts: &mut TextSystem) -> Fragment {
    let s = &node.style;
    let h_extra = s.border.left + s.border.right + s.padding.left + s.padding.right;
    let v_extra = s.border.top + s.border.bottom + s.padding.top + s.padding.bottom;

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
            ts,
        );
        let m = child.style.margin;
        f.offset = Point {
            x: m.left,
            y: cursor_y + m.top,
        };
        cursor_y = f.offset.y + f.size.h + m.bottom;
        children.push(f);
    }

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
        text: None,
        children,
    }
}

/// A text box fills the available width and is as tall as its wrapped lines.
fn layout_text(node: &BoxNode, text: &str, c: Constraints, ts: &mut TextSystem) -> Fragment {
    let s = &node.style;
    let buf = ts.shape(text, s.font_size, s.line_height, c.available_width);
    let lines = buf.layout_runs().count() as f32;

    Fragment {
        offset: Point::default(),
        size: Size {
            w: c.available_width,
            h: lines * s.line_height,
        },
        content_offset: Point::default(),
        background: None,
        border: Edges::default(),
        border_color: s.border_color,
        text: Some(TextFragment {
            text: text.to_owned(),
            font_size: s.font_size,
            line_height: s.line_height,
            color: s.color,
        }),
        children: Vec::new(),
    }
}
