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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    fn block(style: Style, children: Vec<BoxNode>) -> BoxNode {
        BoxNode {
            node: None,
            kind: BoxKind::Block,
            style,
            children,
        }
    }

    fn sized(h: f32, margin: Edges) -> BoxNode {
        block(
            Style {
                height: Some(h),
                margin,
                ..Style::default()
            },
            Vec::new(),
        )
    }

    fn run(node: &BoxNode, w: f32) -> Fragment {
        layout(
            node,
            Constraints { available_width: w },
            &mut TextSystem::new(),
        )
    }

    #[test]
    fn block_fills_width_minus_margin() {
        let node = block(
            Style {
                margin: Edges::x(10.0),
                padding: Edges::all(5.0),
                border: Edges::all(1.0),
                ..Style::default()
            },
            Vec::new(),
        );
        let f = run(&node, 100.0);
        // border box = available - margins; content = that - padding - border
        assert_eq!(f.size.w, 80.0);
        assert_eq!(f.content_offset, Point { x: 6.0, y: 6.0 });
    }

    #[test]
    fn children_stack_vertically_with_margins() {
        let node = block(
            Style::default(),
            vec![sized(10.0, Edges::y(5.0)), sized(20.0, Edges::default())],
        );
        let f = run(&node, 100.0);
        assert_eq!(f.children[0].offset, Point { x: 0.0, y: 5.0 });
        assert_eq!(f.children[1].offset, Point { x: 0.0, y: 20.0 });
        assert_eq!(f.size.h, 40.0);
    }

    #[test]
    fn explicit_height_overrides_content() {
        let node = block(
            Style {
                height: Some(3.0),
                ..Style::default()
            },
            vec![sized(50.0, Edges::default())],
        );
        assert_eq!(run(&node, 100.0).size.h, 3.0);
    }

    #[test]
    fn text_gets_taller_when_narrower() {
        let node = BoxNode {
            node: None,
            kind: BoxKind::Text("the quick brown fox jumps over the lazy dog".into()),
            style: Style {
                font_size: 12.0,
                line_height: 16.0,
                ..Style::default()
            },
            children: Vec::new(),
        };
        let wide = run(&node, 1000.0);
        let narrow = run(&node, 40.0);
        assert_eq!(wide.size.h, 16.0);
        assert!(narrow.size.h > wide.size.h);
    }
}
