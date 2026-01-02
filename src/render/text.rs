//! Text rendering using cosmic-text.
//!
//! # Performance
//!
//! The TextRenderer includes optimizations:
//! - LRU cache for text measurements (avoids repeated shaping)
//! - Pre-allocated buffers where possible
//! - Inline hints on hot paths

use crate::geometry::Size;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::{Hash, Hasher};

/// Maximum number of cached text measurements.
const MEASURE_CACHE_SIZE: usize = 256;

/// Key for text measurement cache.
#[derive(Clone, Eq, PartialEq)]
struct MeasureCacheKey {
    text: String,
    font_size_bits: u32,
}

impl Hash for MeasureCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font_size_bits.hash(state);
    }
}

/// Text renderer using cosmic-text.
pub struct TextRenderer {
    font_system: Arc<Mutex<FontSystem>>,
    swash_cache: SwashCache,
    /// Cache for text measurements (text+size -> dimensions)
    measure_cache: Mutex<MeasureCache>,
}

/// LRU-like cache for text measurements.
struct MeasureCache {
    entries: HashMap<MeasureCacheKey, Size>,
    order: Vec<MeasureCacheKey>,
}

impl MeasureCache {
    fn new() -> Self {
        Self {
            entries: HashMap::with_capacity(MEASURE_CACHE_SIZE),
            order: Vec::with_capacity(MEASURE_CACHE_SIZE),
        }
    }

    fn get(&self, key: &MeasureCacheKey) -> Option<Size> {
        self.entries.get(key).copied()
    }

    fn insert(&mut self, key: MeasureCacheKey, size: Size) {
        // Simple eviction: remove oldest when full
        if self.entries.len() >= MEASURE_CACHE_SIZE && !self.order.is_empty() {
            let old_key = self.order.remove(0);
            self.entries.remove(&old_key);
        }
        self.entries.insert(key.clone(), size);
        self.order.push(key);
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
    }
}

impl TextRenderer {
    pub fn new() -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        Self {
            font_system: Arc::new(Mutex::new(font_system)),
            swash_cache,
            measure_cache: Mutex::new(MeasureCache::new()),
        }
    }

    /// Measure text dimensions with caching.
    #[inline]
    pub fn measure(&self, text: &str, font_size: f32) -> Size {
        // Quick path for empty text
        if text.is_empty() {
            return Size::new(0.0, font_size * 1.2);
        }

        // Check cache first
        let key = MeasureCacheKey {
            text: text.to_string(),
            font_size_bits: font_size.to_bits(),
        };

        {
            let cache = self.measure_cache.lock().unwrap();
            if let Some(size) = cache.get(&key) {
                return size;
            }
        }

        // Cache miss - compute size
        let size = self.measure_uncached(text, font_size);

        // Update cache
        {
            let mut cache = self.measure_cache.lock().unwrap();
            cache.insert(key, size);
        }

        size
    }

    /// Measure text without caching (for internal use).
    fn measure_uncached(&self, text: &str, font_size: f32) -> Size {
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

    /// Clear the measurement cache.
    pub fn clear_cache(&self) {
        let mut cache = self.measure_cache.lock().unwrap();
        cache.clear();
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.measure_cache.lock().unwrap();
        (cache.entries.len(), MEASURE_CACHE_SIZE)
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
