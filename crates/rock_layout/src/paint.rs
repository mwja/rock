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

    if let Some(t) = &f.text {
        list.items.push(DisplayItem::Text {
            origin,
            text: t.text.clone(),
            font_size: t.font_size,
            line_height: t.line_height,
            max_width: f.size.w,
            color: t.color,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn frag(offset: Point, children: Vec<Fragment>) -> Fragment {
        Fragment {
            offset,
            size: Size { w: 10.0, h: 10.0 },
            content_offset: Point::default(),
            background: None,
            border: Edges::default(),
            border_color: Color::BLACK,
            text: None,
            children,
        }
    }

    fn paint_one(f: &Fragment) -> Vec<DisplayItem> {
        build_display_list(f, Size { w: 100.0, h: 100.0 }).items
    }

    #[test]
    fn empty_fragment_paints_nothing() {
        assert!(paint_one(&frag(Point::default(), Vec::new())).is_empty());
    }

    #[test]
    fn background_and_border_are_painted_in_order() {
        let mut f = frag(Point { x: 1.0, y: 2.0 }, Vec::new());
        f.background = Some(Color::WHITE);
        f.border = Edges::all(1.0);
        let items = paint_one(&f);
        let rect = Rect::new(1.0, 2.0, 10.0, 10.0);
        assert_eq!(
            items[0],
            DisplayItem::FillRect {
                rect,
                color: Color::WHITE
            }
        );
        assert!(matches!(items[1], DisplayItem::Border { rect: r, .. } if r == rect));
    }

    #[test]
    fn child_is_offset_by_parent_content_origin() {
        let mut child = frag(Point { x: 3.0, y: 4.0 }, Vec::new());
        child.background = Some(Color::WHITE);
        let mut parent = frag(Point { x: 10.0, y: 20.0 }, vec![child]);
        parent.content_offset = Point { x: 1.0, y: 1.0 };

        let items = paint_one(&parent);
        let DisplayItem::FillRect { rect, .. } = items[0] else {
            panic!("expected a fill, got {:?}", items[0]);
        };
        assert_eq!(rect.origin, Point { x: 14.0, y: 25.0 });
    }
}
