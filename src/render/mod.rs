//! Rendering engine for OpenKit.
//!
//! Provides GPU-accelerated rendering using wgpu with a CPU fallback using skia-rs.

mod painter;
mod text;

pub use painter::{Painter, DrawCommand};
pub use text::TextRenderer;

use crate::geometry::{Color, Point, Rect, Size, BorderRadius};
use crate::platform::Window;

use std::num::NonZeroU32;

use skia_rs_safe::canvas::raster::{PixelBuffer, Rasterizer};
use skia_rs_safe::core::Color as SkiaColor;
use skia_rs_safe::paint::{Paint, Style};

#[cfg(feature = "gpu")]
use wgpu;

/// The main renderer.
pub struct Renderer {
    #[cfg(feature = "gpu")]
    gpu: Option<GpuRenderer>,
    cpu: CpuRenderer,
    text: TextRenderer,
    /// Softbuffer surface for CPU rendering presentation
    software_surface: Option<SoftwareSurface>,
}

/// Software surface for CPU rendering presentation using softbuffer.
struct SoftwareSurface {
    #[allow(dead_code)]
    context: softbuffer::Context<std::sync::Arc<winit::window::Window>>,
    surface: softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
}

impl SoftwareSurface {
    fn new(window: &Window) -> Option<Self> {
        let arc = window.inner_arc();

        let context = match softbuffer::Context::new(arc.clone()) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to create softbuffer context: {}", e);
                return None;
            }
        };

        let surface = match softbuffer::Surface::new(&context, arc) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to create softbuffer surface: {}", e);
                return None;
            }
        };

        Some(Self { context, surface })
    }

    fn present(&mut self, pixels: &[u8], width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        // Resize surface if needed
        if let Err(e) = self.surface.resize(
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        ) {
            log::warn!("Failed to resize software surface: {}", e);
            return;
        }

        // Get buffer and copy pixels
        let mut buffer = match self.surface.buffer_mut() {
            Ok(b) => b,
            Err(e) => {
                log::warn!("Failed to get software surface buffer: {}", e);
                return;
            }
        };

        // Convert RGBA8 to the format softbuffer expects (0x00RRGGBB)
        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;
            buffer[i] = (r << 16) | (g << 8) | b;
        }

        // Present
        if let Err(e) = buffer.present() {
            log::warn!("Failed to present software surface: {}", e);
        }
    }
}

impl Renderer {
    /// Create a new renderer for a window.
    pub fn new(window: &Window) -> Self {
        #[cfg(feature = "gpu")]
        let gpu = match GpuRenderer::new(window) {
            Ok(g) => {
                log::info!("Using GPU rendering (wgpu)");
                Some(g)
            }
            Err(e) => {
                log::warn!("GPU rendering unavailable: {}. Falling back to CPU rendering.", e);
                None
            }
        };

        #[cfg(not(feature = "gpu"))]
        log::info!("Using CPU rendering (skia-rs)");

        // Create software surface for CPU rendering presentation
        let software_surface = {
            #[cfg(feature = "gpu")]
            {
                if gpu.is_none() {
                    SoftwareSurface::new(window)
                } else {
                    None
                }
            }
            #[cfg(not(feature = "gpu"))]
            {
                SoftwareSurface::new(window)
            }
        };

        if software_surface.is_some() {
            log::info!("Software surface created for CPU rendering presentation");
        }

        Self {
            #[cfg(feature = "gpu")]
            gpu,
            cpu: CpuRenderer::new(),
            text: TextRenderer::new(),
            software_surface,
        }
    }

    /// Resize the renderer.
    pub fn resize(&mut self, size: Size) {
        #[cfg(feature = "gpu")]
        if let Some(gpu) = &mut self.gpu {
            gpu.resize(size);
        }
        self.cpu.resize(size);
    }

    /// Begin a new frame.
    pub fn begin_frame(&mut self, background: Color) {
        #[cfg(feature = "gpu")]
        if let Some(gpu) = &mut self.gpu {
            gpu.begin_frame(background);
            return;
        }
        self.cpu.begin_frame(background);
    }

    /// End the frame and present.
    pub fn end_frame(&mut self) {
        #[cfg(feature = "gpu")]
        if let Some(gpu) = &mut self.gpu {
            gpu.end_frame();
            return;
        }
        self.cpu.end_frame();

        // Present CPU-rendered pixels to window using softbuffer
        if let Some(surface) = &mut self.software_surface {
            let size = self.cpu.size;
            surface.present(self.cpu.pixels(), size.width as u32, size.height as u32);
        }
    }

    /// Get a painter for drawing.
    pub fn painter(&mut self) -> Painter {
        Painter::new()
    }

    /// Execute draw commands.
    pub fn draw(&mut self, commands: &[DrawCommand]) {
        for cmd in commands {
            match cmd {
                DrawCommand::Rect { rect, color, radius } => {
                    self.draw_rect(*rect, *color, *radius);
                }
                DrawCommand::Text { text, position, color, size } => {
                    self.draw_text(text, *position, *color, *size);
                }
                DrawCommand::Line { from, to, color, width } => {
                    self.draw_line(*from, *to, *color, *width);
                }
                DrawCommand::Image { rect, .. } => {
                    // TODO: Image rendering
                    self.draw_rect(*rect, Color::from_rgb8(200, 200, 200), BorderRadius::ZERO);
                }
            }
        }
    }

    /// Draw a filled rectangle.
    pub fn draw_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius) {
        #[cfg(feature = "gpu")]
        if self.gpu.is_some() {
            // GPU path - would use wgpu
            return;
        }
        self.cpu.draw_rect(rect, color, radius);
    }

    /// Draw text.
    pub fn draw_text(&mut self, text: &str, position: Point, color: Color, size: f32) {
        #[cfg(feature = "gpu")]
        if self.gpu.is_some() {
            // GPU path
            return;
        }
        self.cpu.draw_text(text, position, color, size, &self.text);
    }

    /// Draw a line.
    pub fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        #[cfg(feature = "gpu")]
        if self.gpu.is_some() {
            return;
        }
        self.cpu.draw_line(from, to, color, width);
    }

    /// Measure text size.
    pub fn measure_text(&self, text: &str, size: f32) -> Size {
        self.text.measure(text, size)
    }

    /// Get the pixel buffer for software rendering (for presenting to window).
    pub fn pixels(&self) -> Option<&[u8]> {
        Some(self.cpu.pixels())
    }
}

/// GPU renderer using wgpu.
#[cfg(feature = "gpu")]
pub struct GpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    #[allow(dead_code)]
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: Size,
}

#[cfg(feature = "gpu")]
impl GpuRenderer {
    pub fn new(window: &Window) -> Result<Self, RenderError> {
        let size = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window.inner_arc())
            .map_err(|e| RenderError::SurfaceCreation(e.to_string()))?;

        // Get adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or(RenderError::NoAdapter)?;

        // Get device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("OpenKit Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .map_err(|e| RenderError::DeviceCreation(e.to_string()))?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
        })
    }

    pub fn resize(&mut self, size: Size) {
        if size.width > 0.0 && size.height > 0.0 {
            self.size = size;
            self.config.width = size.width as u32;
            self.config.height = size.height as u32;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn begin_frame(&mut self, _background: Color) {
        // TODO: Start render pass with background clear
    }

    pub fn end_frame(&mut self) {
        // TODO: Submit and present
    }
}

/// CPU renderer using skia-rs raster backend.
pub struct CpuRenderer {
    pixel_buffer: PixelBuffer,
    size: Size,
}

impl CpuRenderer {
    pub fn new() -> Self {
        Self {
            pixel_buffer: PixelBuffer::new(1, 1),
            size: Size::new(1.0, 1.0),
        }
    }

    pub fn resize(&mut self, size: Size) {
        if size.width > 0.0 && size.height > 0.0 {
            self.size = size;
            let w = size.width as i32;
            let h = size.height as i32;
            self.pixel_buffer = PixelBuffer::new(w, h);
        }
    }

    pub fn begin_frame(&mut self, background: Color) {
        let [r, g, b, a] = background.to_rgba8();
        self.pixel_buffer.clear(SkiaColor::from_argb(a, r, g, b));
    }

    pub fn end_frame(&mut self) {
        // Pixels are already in the buffer, nothing to do
    }

    pub fn pixels(&self) -> &[u8] {
        // Convert the pixel data to bytes
        // PixelBuffer stores pixels as u32 (ARGB), we need to return as &[u8]
        let pixels = &self.pixel_buffer.pixels;
        // Safety: u32 slice can be viewed as u8 slice with 4x the length
        unsafe {
            std::slice::from_raw_parts(
                pixels.as_ptr() as *const u8,
                pixels.len() * 4,
            )
        }
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius) {
        let [r, g, b, a] = color.to_rgba8();
        let skia_color = SkiaColor::from_argb(a, r, g, b);

        let mut paint = Paint::new();
        paint.set_color(skia_color.into());
        paint.set_style(Style::Fill);
        paint.set_anti_alias(true);

        let mut rasterizer = Rasterizer::new(&mut self.pixel_buffer);

        let skia_rect = skia_rs_safe::core::Rect::from_xywh(
            rect.x(),
            rect.y(),
            rect.width(),
            rect.height(),
        );

        if radius.is_zero() {
            rasterizer.fill_rect(&skia_rect, &paint);
        } else {
            // For rounded rects, we use fill_rect for now
            // TODO: Implement rounded rect path when available
            rasterizer.fill_rect(&skia_rect, &paint);
        }
    }

    pub fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let [r, g, b, a] = color.to_rgba8();
        let skia_color = SkiaColor::from_argb(a, r, g, b);

        let mut paint = Paint::new();
        paint.set_color(skia_color.into());
        paint.set_style(Style::Stroke);
        paint.set_stroke_width(width);
        paint.set_anti_alias(true);

        let mut rasterizer = Rasterizer::new(&mut self.pixel_buffer);

        let p0 = skia_rs_safe::core::Point::new(from.x, from.y);
        let p1 = skia_rs_safe::core::Point::new(to.x, to.y);
        rasterizer.draw_line(p0, p1, &paint);
    }

    pub fn draw_text(&mut self, _text: &str, _position: Point, _color: Color, _size: f32, _text_renderer: &TextRenderer) {
        // Text rendering requires the full canvas API with fonts
        // For now, use the TextRenderer from cosmic-text for text
        // TODO: Integrate skia-rs text rendering when available in raster backend
    }
}

impl Default for CpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Render error types.
#[derive(Debug, Clone)]
pub enum RenderError {
    SurfaceCreation(String),
    NoAdapter,
    DeviceCreation(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::SurfaceCreation(e) => write!(f, "Failed to create surface: {}", e),
            RenderError::NoAdapter => write!(f, "No suitable GPU adapter found"),
            RenderError::DeviceCreation(e) => write!(f, "Failed to create device: {}", e),
        }
    }
}

impl std::error::Error for RenderError {}
