//! HTML in, display list out: the whole layout half of the pipeline.
use rock_dom::prelude::*;
use rock_layout::prelude::*;
use rock_render::{prelude::*, text::TextSystem};

const VIEWPORT: Size = Size { w: 800.0, h: 600.0 };

fn display_list(html: &str) -> DisplayList {
    let dom = create_dom(html).expect("valid html");
    let boxes = build_box_tree(&dom).expect("document has a root element");
    let root = layout(
        &boxes,
        Constraints {
            available_width: VIEWPORT.w,
        },
        &mut TextSystem::new(),
    );
    build_display_list(&root, VIEWPORT)
}

/// (text, origin) of every text item in the list.
fn texts(list: &DisplayList) -> Vec<(&str, Point)> {
    list.items
        .iter()
        .filter_map(|item| match item {
            DisplayItem::Text { text, origin, .. } => Some((text.as_str(), *origin)),
            _ => None,
        })
        .collect()
}

#[test]
fn paragraph_text_is_placed_inside_body_and_p_margins() {
    let list = display_list("<p>Hello</p>");
    // body margin 8 on both axes, plus p's 16 top margin
    assert_eq!(texts(&list), [("Hello", Point { x: 8.0, y: 24.0 })]);

    let DisplayItem::Text { max_width, .. } = list.items[0] else {
        panic!("expected text");
    };
    assert!((max_width - (VIEWPORT.w - 16.0)).abs() < 0.01);
}

#[test]
fn simple_squares_case_has_both_text_runs() {
    let list = display_list(include_str!("../../../cases/simple_squares.html"));
    let found: Vec<_> = texts(&list).into_iter().map(|(t, _)| t).collect();
    assert_eq!(found, ["Hello,", "World!"]);
}
