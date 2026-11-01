// rock_render/src/text.rs
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};

pub struct TextSystem {
    pub fonts: FontSystem,
    pub cache: SwashCache,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    pub fn new() -> Self {
        Self {
            fonts: FontSystem::new(),
            cache: SwashCache::new(),
        }
    }

    /// Shape `text` into a buffer that wraps at `max_width`.
    pub fn shape(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        max_width: f32,
    ) -> Buffer {
        let mut buf = Buffer::new(&mut self.fonts, Metrics::new(font_size, line_height));
        buf.set_size(Some(max_width), None);
        buf.set_text(
            text,
            &Attrs::new().family(Family::Serif),
            Shaping::Advanced,
            None,
        );
        buf.shape_until_scroll(&mut self.fonts, false);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "the quick brown fox jumps over the lazy dog";

    fn lines(ts: &mut TextSystem, max_width: f32) -> usize {
        ts.shape(TEXT, 12.0, 16.0, max_width).layout_runs().count()
    }

    #[test]
    fn short_text_fits_on_one_line() {
        assert_eq!(lines(&mut TextSystem::new(), 10_000.0), 1);
    }

    #[test]
    fn narrow_width_wraps_onto_more_lines() {
        let mut ts = TextSystem::new();
        assert!(lines(&mut ts, 40.0) > lines(&mut ts, 10_000.0));
    }
}
