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
