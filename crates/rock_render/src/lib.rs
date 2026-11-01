mod display;
pub mod geo;
pub mod text;

pub mod prelude {
    pub use super::display::*;
    pub use super::geo::*;
    pub use super::render;
}

use crate::{
    geo::Point,
    prelude::{Color, DisplayItem, DisplayList, Rect},
    text::TextSystem,
};
use cosmic_text::Buffer;
use tiny_skia::{Paint, Pixmap, Transform};
use tracing::{trace, warn};

/// Rasterize a display list into a `width` x `height` device-pixel pixmap.
/// `scale` is the device-pixel ratio (window.scale_factor()).
pub fn render(
    list: &DisplayList,
    width: u32,
    height: u32,
    scale: f32,
    ts: &mut TextSystem,
) -> Pixmap {
    trace!("Rendering list of size {} at scale {scale}", list.len());
    trace!("Pixmap in device pixels: {width},{height}");

    let mut pixmap = Pixmap::new(width.max(1), height.max(1)).expect("non-zero size");
    pixmap.fill(tiny_skia::Color::WHITE);

    // display list coordinates are CSS pixels; this maps them to device pixels
    let transform = Transform::from_scale(scale, scale);

    for item in &list.items {
        match item {
            DisplayItem::FillRect { rect, color } => {
                trace!("Rect fill at {:?}", rect.origin);
                fill(&mut pixmap, *rect, *color, transform);
            }
            DisplayItem::Border {
                rect,
                widths,
                colors,
            } => {
                trace!("Border draw at {:?}, width={:?}", rect.origin, widths);
                let [top, right, bottom, left] = *colors;
                let Rect { origin: o, size: s } = *rect;
                // top and bottom span the full width; left and right fit between them
                fill(
                    &mut pixmap,
                    Rect::new(o.x, o.y, s.w, widths.top),
                    top,
                    transform,
                );
                fill(
                    &mut pixmap,
                    Rect::new(o.x, o.y + s.h - widths.bottom, s.w, widths.bottom),
                    bottom,
                    transform,
                );
                let inner_y = o.y + widths.top;
                let inner_h = s.h - widths.top - widths.bottom;
                fill(
                    &mut pixmap,
                    Rect::new(o.x, inner_y, widths.left, inner_h),
                    left,
                    transform,
                );
                fill(
                    &mut pixmap,
                    Rect::new(o.x + s.w - widths.right, inner_y, widths.right, inner_h),
                    right,
                    transform,
                );
            }
            DisplayItem::Text {
                origin,
                text,
                font_size,
                line_height,
                max_width,
                color,
            } => {
                let mut buf = ts.shape(
                    text,
                    font_size * scale,
                    line_height * scale,
                    max_width * scale,
                );
                let device_origin = Point {
                    x: origin.x * scale,
                    y: origin.y * scale,
                };
                draw_text(&mut pixmap, ts, &mut buf, device_origin, *color);
            }
        }
    }
    pixmap
}

fn fill(pixmap: &mut Pixmap, rect: Rect, color: Color, transform: Transform) {
    let Some(r) =
        tiny_skia::Rect::from_xywh(rect.origin.x, rect.origin.y, rect.size.w, rect.size.h)
    else {
        warn!("rect {:?} failed to be converted into a skia rect", rect);
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    pixmap.fill_rect(r, &paint, transform, None);
}

pub fn draw_text(
    pixmap: &mut Pixmap,
    ts: &mut TextSystem,
    buf: &mut Buffer,
    origin: Point,
    color: Color,
) {
    let c = cosmic_text::Color::rgba(color.r, color.g, color.b, color.a);
    buf.draw(&mut ts.fonts, &mut ts.cache, c, |x, y, w, h, px| {
        let Some(r) = tiny_skia::Rect::from_xywh(
            origin.x + x as f32,
            origin.y + y as f32,
            w as f32,
            h as f32,
        ) else {
            return;
        };
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(px.r(), px.g(), px.b(), px.a());
        pixmap.fill_rect(r, &paint, tiny_skia::Transform::identity(), None);
    });
}
