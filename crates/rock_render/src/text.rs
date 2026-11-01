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
