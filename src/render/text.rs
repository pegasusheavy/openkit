//! Text rendering using cosmic-text.

use crate::geometry::Size;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use std::sync::{Arc, Mutex};

/// Text renderer using cosmic-text.
pub struct TextRenderer {
    font_system: Arc<Mutex<FontSystem>>,
    swash_cache: SwashCache,
}

impl TextRenderer {
    pub fn new() -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        Self {
            font_system: Arc::new(Mutex::new(font_system)),
            swash_cache,
        }
    }

    /// Measure text dimensions.
    pub fn measure(&self, text: &str, font_size: f32) -> Size {
        let mut font_system = self.font_system.lock().unwrap();

        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        buffer.set_size(&mut font_system, Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(&mut font_system, text, Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system, false);

        let width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, |a, b| a.max(b));

        let height = buffer.layout_runs().count() as f32 * font_size * 1.2;

        Size::new(width, height.max(font_size * 1.2))
    }

    /// Get a reference to the font system.
    pub fn font_system(&self) -> Arc<Mutex<FontSystem>> {
        self.font_system.clone()
    }

    /// Rasterize text to pixels.
    /// Returns (width, height, pixels) where pixels is RGBA.
    pub fn rasterize(
        &mut self,
        text: &str,
        font_size: f32,
        color: [u8; 4],
    ) -> (u32, u32, Vec<u8>) {
        let mut font_system = self.font_system.lock().unwrap();

        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        buffer.set_size(&mut font_system, Some(1000.0), Some(font_size * 2.0));
        buffer.set_text(&mut font_system, text, Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system, false);

        // Measure dimensions
        let width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, |a, b| a.max(b))
            .ceil() as u32;
        let height = (buffer.layout_runs().count() as f32 * font_size * 1.2).ceil() as u32;

        if width == 0 || height == 0 {
            return (0, 0, Vec::new());
        }

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        // Rasterize glyphs
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0.0, 0.0), 1.0);

                if let Some(image) = self.swash_cache.get_image(&mut font_system, physical_glyph.cache_key) {
                    let glyph_x = physical_glyph.x;
                    let glyph_y = physical_glyph.y + run.line_y as i32;

                    for (i, alpha) in image.data.iter().enumerate() {
                        let px = glyph_x + (i as i32 % image.placement.width as i32);
                        let py = glyph_y + (i as i32 / image.placement.width as i32);

                        if px >= 0 && py >= 0 && (px as u32) < width && (py as u32) < height {
                            let idx = ((py as u32 * width + px as u32) * 4) as usize;
                            if idx + 3 < pixels.len() {
                                let a = *alpha as f32 / 255.0;
                                pixels[idx] = ((color[0] as f32 * a) as u8).saturating_add(pixels[idx]);
                                pixels[idx + 1] = ((color[1] as f32 * a) as u8).saturating_add(pixels[idx + 1]);
                                pixels[idx + 2] = ((color[2] as f32 * a) as u8).saturating_add(pixels[idx + 2]);
                                pixels[idx + 3] = (*alpha).saturating_add(pixels[idx + 3]);
                            }
                        }
                    }
                }
            }
        }

        (width, height, pixels)
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_text() {
        let renderer = TextRenderer::new();
        let size = renderer.measure("Hello, World!", 16.0);

        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
    }

    #[test]
    fn test_empty_text() {
        let renderer = TextRenderer::new();
        let size = renderer.measure("", 16.0);

        assert_eq!(size.width, 0.0);
    }
}
