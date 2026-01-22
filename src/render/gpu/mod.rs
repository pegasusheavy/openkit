//! GPU-accelerated rendering engine for OpenKit.
//!
//! This module provides high-performance GPU rendering using wgpu with:
//! - Batched draw calls for minimal GPU overhead
//! - Custom shaders for primitives (rects, rounded rects, gradients)
//! - Texture atlas for efficient text and image rendering
//! - GPU-accelerated effects (blur, shadow, glow)
//! - Multi-sample anti-aliasing (MSAA)

mod pipeline;
mod batch;
mod shaders;
mod texture_atlas;
mod effects;

pub use pipeline::GpuPipeline;
pub use batch::{DrawBatch, BatchVertex};
pub use texture_atlas::TextureAtlas;
pub use effects::{EffectsPipeline, BlurEffect, ShadowEffect, GlowEffect};

use crate::geometry::{Color, Point, Rect, Size, BorderRadius};
use wgpu;

/// Configuration for GPU rendering.
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// Enable multi-sample anti-aliasing.
    pub msaa_samples: u32,
    /// Maximum texture atlas size.
    pub max_atlas_size: u32,
    /// Enable V-Sync.
    pub vsync: bool,
    /// Preferred power mode.
    pub power_preference: PowerPreference,
    /// Enable HDR if available.
    pub hdr: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            msaa_samples: 4,
            max_atlas_size: 4096,
            vsync: true,
            power_preference: PowerPreference::HighPerformance,
            hdr: false,
        }
    }
}

/// Power preference for GPU selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerPreference {
    /// Prefer low power GPU (integrated).
    LowPower,
    /// Prefer high performance GPU (discrete).
    HighPerformance,
}

impl From<PowerPreference> for wgpu::PowerPreference {
    fn from(pref: PowerPreference) -> Self {
        match pref {
            PowerPreference::LowPower => wgpu::PowerPreference::LowPower,
            PowerPreference::HighPerformance => wgpu::PowerPreference::HighPerformance,
        }
    }
}

/// GPU-accelerated renderer.
pub struct GpuRenderer {
    // Core wgpu objects
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,

    // Rendering pipelines
    pub(crate) rect_pipeline: GpuPipeline,
    pub(crate) text_pipeline: GpuPipeline,
    pub(crate) effects_pipeline: EffectsPipeline,

    // Resource management
    pub(crate) texture_atlas: TextureAtlas,
    pub(crate) draw_batch: DrawBatch,

    // MSAA
    pub(crate) msaa_texture: Option<wgpu::Texture>,
    pub(crate) msaa_view: Option<wgpu::TextureView>,

    // State
    pub(crate) size: Size,
    pub(crate) scale_factor: f32,
    pub(crate) frame_texture: Option<wgpu::SurfaceTexture>,
    pub(crate) frame_view: Option<wgpu::TextureView>,

    // Stats
    pub(crate) stats: RenderStats,
}

/// Rendering statistics for profiling.
#[derive(Debug, Default, Clone)]
pub struct RenderStats {
    /// Number of draw calls this frame.
    pub draw_calls: u32,
    /// Number of vertices this frame.
    pub vertices: u32,
    /// Number of triangles this frame.
    pub triangles: u32,
    /// Time to render frame in microseconds.
    pub frame_time_us: u64,
    /// GPU memory used in bytes.
    pub gpu_memory_bytes: u64,
}

impl GpuRenderer {
    /// Create a new GPU renderer for a window.
    pub fn new(
        window: &crate::platform::Window,
        config: GpuConfig,
    ) -> Result<Self, GpuError> {
        let size = window.size();

        // Create wgpu instance with all backends
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window.inner_arc())
            .map_err(|e| GpuError::SurfaceCreation(e.to_string()))?;

        // Request high-performance adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: config.power_preference.into(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or(GpuError::NoAdapter)?;

        log::info!("GPU Adapter: {:?}", adapter.get_info());

        // Request device with needed features
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("OpenKit GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .map_err(|e| GpuError::DeviceCreation(e.to_string()))?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = if config.hdr {
            surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| matches!(f, wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgb10a2Unorm))
                .unwrap_or(surface_caps.formats[0])
        } else {
            surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0])
        };

        let present_mode = if config.vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Immediate
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1.0) as u32,
            height: size.height.max(1.0) as u32,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create pipelines
        let rect_pipeline = GpuPipeline::new_rect_pipeline(&device, surface_format, config.msaa_samples);
        let text_pipeline = GpuPipeline::new_text_pipeline(&device, surface_format, config.msaa_samples);
        let effects_pipeline = EffectsPipeline::new(&device, surface_format);

        // Create texture atlas
        let texture_atlas = TextureAtlas::new(&device, config.max_atlas_size);

        // Create draw batch
        let draw_batch = DrawBatch::new(&device);

        // Create MSAA texture if enabled
        let (msaa_texture, msaa_view) = if config.msaa_samples > 1 {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Texture"),
                size: wgpu::Extent3d {
                    width: size.width.max(1.0) as u32,
                    height: size.height.max(1.0) as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: config.msaa_samples,
                dimension: wgpu::TextureDimension::D2,
                format: surface_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(texture), Some(view))
        } else {
            (None, None)
        };

        Ok(Self {
            surface,
            device,
            queue,
            config: surface_config,
            rect_pipeline,
            text_pipeline,
            effects_pipeline,
            texture_atlas,
            draw_batch,
            msaa_texture,
            msaa_view,
            size,
            scale_factor: 1.0,
            frame_texture: None,
            frame_view: None,
            stats: RenderStats::default(),
        })
    }

    /// Resize the renderer.
    pub fn resize(&mut self, size: Size) {
        if size.width > 0.0 && size.height > 0.0 {
            self.size = size;
            self.config.width = size.width as u32;
            self.config.height = size.height as u32;
            self.surface.configure(&self.device, &self.config);

            // Recreate MSAA texture if needed
            if self.msaa_texture.is_some() {
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("MSAA Texture"),
                    size: wgpu::Extent3d {
                        width: size.width as u32,
                        height: size.height as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 4,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.config.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.msaa_texture = Some(texture);
                self.msaa_view = Some(view);
            }
        }
    }

    /// Set the scale factor (for HiDPI).
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale;
    }

    /// Begin a new frame.
    pub fn begin_frame(&mut self, background: Color) -> Result<(), GpuError> {
        // Reset stats
        self.stats = RenderStats::default();

        // Get the next frame texture
        let frame = self.surface
            .get_current_texture()
            .map_err(|e| GpuError::FrameAcquisition(e.to_string()))?;

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Clear the frame
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear Encoder"),
        });

        let [r, g, b, a] = background.to_rgba_f32();

        {
            let render_target = self.msaa_view.as_ref().unwrap_or(&view);
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: if self.msaa_view.is_some() { Some(&view) } else { None },
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.frame_texture = Some(frame);
        self.frame_view = Some(view);

        // Clear the batch
        self.draw_batch.clear();

        Ok(())
    }

    /// Draw a filled rectangle.
    pub fn draw_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius) {
        self.draw_batch.add_rect(rect, color, radius, self.size);
    }

    /// Draw a gradient rectangle.
    pub fn draw_gradient_rect(
        &mut self,
        rect: Rect,
        colors: &[Color],
        angle: f32,
        radius: BorderRadius,
    ) {
        self.draw_batch.add_gradient_rect(rect, colors, angle, radius, self.size);
    }

    /// Draw text.
    pub fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        color: Color,
        size: f32,
        font_id: u32,
    ) {
        self.draw_batch.add_text(text, position, color, size, font_id, self.size);
    }

    /// Draw an image.
    pub fn draw_image(&mut self, rect: Rect, texture_id: u32) {
        self.draw_batch.add_image(rect, texture_id, self.size);
    }

    /// Apply a blur effect to a region.
    pub fn apply_blur(&mut self, rect: Rect, radius: f32) {
        self.effects_pipeline.queue_blur(rect, radius);
    }

    /// Apply a shadow effect.
    pub fn apply_shadow(&mut self, rect: Rect, color: Color, blur: f32, offset: Point) {
        self.effects_pipeline.queue_shadow(rect, color, blur, offset);
    }

    /// End the frame and present.
    pub fn end_frame(&mut self) -> Result<RenderStats, GpuError> {
        let start = std::time::Instant::now();

        if let Some(frame_view) = &self.frame_view {
            // Render all batched geometry
            let commands = self.draw_batch.build(&self.device, &self.queue);

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            // Main render pass
            {
                let render_target = self.msaa_view.as_ref().unwrap_or(frame_view);
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: render_target,
                        resolve_target: if self.msaa_view.is_some() { Some(frame_view) } else { None },
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Execute rect pipeline
                if !commands.rect_vertices.is_empty() {
                    pass.set_pipeline(&self.rect_pipeline.pipeline);
                    pass.set_vertex_buffer(0, commands.rect_buffer.slice(..));
                    pass.set_index_buffer(commands.rect_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..commands.rect_index_count, 0, 0..1);

                    self.stats.draw_calls += 1;
                    self.stats.vertices += commands.rect_vertices.len() as u32;
                    self.stats.triangles += commands.rect_index_count / 3;
                }
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        // Present
        if let Some(frame) = self.frame_texture.take() {
            frame.present();
        }

        self.frame_view = None;

        self.stats.frame_time_us = start.elapsed().as_micros() as u64;

        Ok(self.stats.clone())
    }

    /// Get rendering statistics.
    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }

    /// Upload a texture to the GPU.
    pub fn upload_texture(&mut self, width: u32, height: u32, data: &[u8]) -> u32 {
        self.texture_atlas.upload(&self.device, &self.queue, width, height, data)
    }
}

/// GPU rendering errors.
#[derive(Debug, Clone)]
pub enum GpuError {
    SurfaceCreation(String),
    NoAdapter,
    DeviceCreation(String),
    FrameAcquisition(String),
    ShaderCompilation(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::SurfaceCreation(e) => write!(f, "Surface creation failed: {}", e),
            GpuError::NoAdapter => write!(f, "No suitable GPU adapter found"),
            GpuError::DeviceCreation(e) => write!(f, "Device creation failed: {}", e),
            GpuError::FrameAcquisition(e) => write!(f, "Frame acquisition failed: {}", e),
            GpuError::ShaderCompilation(e) => write!(f, "Shader compilation failed: {}", e),
        }
    }
}

impl std::error::Error for GpuError {}
