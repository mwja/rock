use rock_dom::prelude::*;
use rock_render::geo::{Color, Edges};

/// Styling, eventually calculated from the CSS when that works
#[derive(Debug, Default)]
pub struct Style {
    pub display: Display,
    pub width: Option<f32>, // None = auto; content-box width
    pub height: Option<f32>,
    pub margin: Edges,
    pub padding: Edges,
    pub border: Edges,
    pub border_color: Color,
    pub background: Option<Color>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Display {
    None,
    #[default]
    Block,
    Inline,
}

/// Stand-in for the real cascade (M2). Replace this one function later.
pub fn style_for(dom: &Dom, id: NodeId) -> Style {
    let Some(el) = dom.try_element(id) else {
        return Style {
            display: Display::None,
            ..Style::default()
        };
    };
    let tag = &*el.name.local;
    let mut s = Style {
        display: Display::Block,
        ..Style::default()
    };

    match tag {
        "head" | "script" | "style" | "title" | "meta" | "link" | "template" => {
            s.display = Display::None;
        }
        // Body default margin is 8.0
        "body" => s.margin = Edges::all(8.0),
        "p" => {
            s.margin.top = 16.0;
            s.margin.bottom = 16.0;
        }
        "span" | "a" | "b" | "i" | "em" | "strong" | "code" => s.display = Display::Inline,
        _ => {}
    }

    // a debugging aid, not CSS: give some tags a background so you can see them
    match tag {
        "div" => {
            s.background = Some(Color {
                r: 230,
                g: 230,
                b: 250,
                a: 255,
            })
        }
        "p" => {
            s.background = Some(Color {
                r: 250,
                g: 240,
                b: 220,
                a: 255,
            })
        }
        _ => {}
    }
    s
}
