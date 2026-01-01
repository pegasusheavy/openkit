//! Texture atlas for efficient glyph and image rendering.

use std::collections::HashMap;
use wgpu;

/// A texture atlas for packing multiple images/glyphs into a single texture.
pub struct TextureAtlas {
    /// The atlas texture.
    pub texture: wgpu::Texture,
    /// The atlas texture view.
    pub view: wgpu::TextureView,
    /// Sampler for the atlas.
    pub sampler: wgpu::Sampler,
    /// Maximum atlas size.
    max_size: u32,
    /// Current allocation state.
    allocator: AtlasAllocator,
    /// Uploaded texture regions.
    regions: HashMap<u32, AtlasRegion>,
    /// Next texture ID.
    next_id: u32,
}

/// A region within the atlas.
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub layer: u32,
}

impl AtlasRegion {
    /// Get UV coordinates for this region (normalized 0-1).
    pub fn uv(&self, atlas_size: u32) -> (f32, f32, f32, f32) {
        let x1 = self.x as f32 / atlas_size as f32;
        let y1 = self.y as f32 / atlas_size as f32;
        let x2 = (self.x + self.width) as f32 / atlas_size as f32;
        let y2 = (self.y + self.height) as f32 / atlas_size as f32;
        (x1, y1, x2, y2)
    }
}

/// Simple row-based atlas allocator.
struct AtlasAllocator {
    /// Current row Y position.
    row_y: u32,
    /// Current X position in row.
    row_x: u32,
    /// Height of current row.
    row_height: u32,
    /// Atlas size.
    size: u32,
}

impl AtlasAllocator {
    fn new(size: u32) -> Self {
        Self {
            row_y: 0,
            row_x: 0,
            row_height: 0,
            size,
        }
    }

    /// Allocate space for a texture.
    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        // Check if we can fit in current row
        if self.row_x + width <= self.size {
            let pos = (self.row_x, self.row_y);
            self.row_x += width;
            self.row_height = self.row_height.max(height);
            return Some(pos);
        }

        // Start new row
        self.row_y += self.row_height;
        self.row_x = 0;
        self.row_height = 0;

        // Check if we can fit
        if self.row_y + height > self.size {
            return None; // Atlas is full
        }

        let pos = (self.row_x, self.row_y);
        self.row_x = width;
        self.row_height = height;
        Some(pos)
    }

    /// Reset the allocator.
    fn reset(&mut self) {
        self.row_y = 0;
        self.row_x = 0;
        self.row_height = 0;
    }
}

impl TextureAtlas {
    /// Create a new texture atlas.
    pub fn new(device: &wgpu::Device, max_size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width: max_size,
                height: max_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            max_size,
            allocator: AtlasAllocator::new(max_size),
            regions: HashMap::new(),
            next_id: 1,
        }
    }

    /// Upload a texture to the atlas.
    /// Returns a texture ID that can be used for rendering.
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> u32 {
        // Allocate space
        let (x, y) = match self.allocator.allocate(width, height) {
            Some(pos) => pos,
            None => {
                // Atlas is full, need to clear and start over
                log::warn!("Texture atlas full, clearing...");
                self.clear(device);
                self.allocator.allocate(width, height).expect("Texture too large for atlas")
            }
        };

        // Upload to GPU
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Store region
        let id = self.next_id;
        self.next_id += 1;
        self.regions.insert(id, AtlasRegion {
            x,
            y,
            width,
            height,
            layer: 0,
        });

        id
    }

    /// Get a region by ID.
    pub fn get(&self, id: u32) -> Option<&AtlasRegion> {
        self.regions.get(&id)
    }

    /// Get UV coordinates for a texture ID.
    pub fn get_uv(&self, id: u32) -> Option<(f32, f32, f32, f32)> {
        self.regions.get(&id).map(|r| r.uv(self.max_size))
    }

    /// Clear the atlas.
    pub fn clear(&mut self, _device: &wgpu::Device) {
        self.allocator.reset();
        self.regions.clear();
    }

    /// Get the atlas size.
    pub fn size(&self) -> u32 {
        self.max_size
    }

    /// Get the number of textures in the atlas.
    pub fn texture_count(&self) -> usize {
        self.regions.len()
    }
}

/// A glyph cache using the texture atlas.
pub struct GlyphCache {
    /// Cached glyph info.
    glyphs: HashMap<GlyphKey, GlyphInfo>,
}

/// Key for identifying a glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: u32,
    pub glyph_id: u32,
    pub size_px: u32,
}

/// Information about a cached glyph.
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub atlas_id: u32,
    pub width: f32,
    pub height: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
}

impl GlyphCache {
    /// Create a new glyph cache.
    pub fn new() -> Self {
        Self {
            glyphs: HashMap::new(),
        }
    }

    /// Get cached glyph info.
    pub fn get(&self, key: GlyphKey) -> Option<&GlyphInfo> {
        self.glyphs.get(&key)
    }

    /// Insert glyph info.
    pub fn insert(&mut self, key: GlyphKey, info: GlyphInfo) {
        self.glyphs.insert(key, info);
    }

    /// Clear the cache.
    pub fn clear(&mut self) {
        self.glyphs.clear();
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}
