//! Base geometric and style units for all rendering that occurs.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {
            origin: Point { x, y },
            size: Size { w, h },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub fn all(val: f32) -> Edges {
        Edges {
            top: val,
            right: val,
            bottom: val,
            left: val,
        }
    }

    pub fn x(val: f32) -> Edges {
        Edges {
            top: 0.0,
            right: val,
            bottom: 0.0,
            left: val,
        }
    }

    pub fn y(val: f32) -> Edges {
        Edges {
            top: val,
            right: 0.0,
            bottom: val,
            left: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_new_sets_origin_and_size() {
        let r = Rect::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(r.origin, Point { x: 1.0, y: 2.0 });
        assert_eq!(r.size, Size { w: 3.0, h: 4.0 });
    }

    #[test]
    fn edges_axis_helpers() {
        assert_eq!(
            Edges::x(2.0),
            Edges {
                top: 0.0,
                right: 2.0,
                bottom: 0.0,
                left: 2.0
            }
        );
        assert_eq!(
            Edges::y(2.0),
            Edges {
                top: 2.0,
                right: 0.0,
                bottom: 2.0,
                left: 0.0
            }
        );
    }

    #[test]
    fn named_colors_are_opaque() {
        assert_eq!(Color::WHITE.a, 255);
        assert_eq!(Color::BLACK.a, 255);
    }
}
