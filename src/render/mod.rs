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
    /// Window reference for deferred surface creation
    window: std::sync::Arc<winit::window::Window>,
    /// Frame count for initial frames (to work around X11 timing)
    frame_count: u32,
}

/// Software surface for CPU rendering presentation using softbuffer.
struct SoftwareSurface {
    #[allow(dead_code)]
    context: softbuffer::Context<std::sync::Arc<winit::window::Window>>,
    surface: softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
}

impl SoftwareSurface {
    #[allow(dead_code)]
    fn new(window: &Window) -> Option<Self> {
        Self::new_from_arc(window.inner_arc())
    }
    
    fn new_from_arc(arc: std::sync::Arc<winit::window::Window>) -> Option<Self> {
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
        let pixel_count = (width * height) as usize;
        for (i, pixel) in pixels.chunks_exact(4).take(pixel_count).enumerate() {
            if i >= buffer.len() {
                break;
            }
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
        // GPU 2D rendering is not yet implemented, so we always use CPU rendering
        // TODO: Enable GPU rendering once draw_rect, draw_line, draw_text are implemented for wgpu
        #[cfg(feature = "gpu")]
        let gpu: Option<GpuRenderer> = {
            log::info!("GPU 2D rendering not yet implemented, using CPU rendering (skia-rs)");
            None
        };

        #[cfg(not(feature = "gpu"))]
        log::info!("Using CPU rendering (skia-rs)");

        // Defer software surface creation until first resize to ensure window is mapped
        Self {
            #[cfg(feature = "gpu")]
            gpu,
            cpu: CpuRenderer::new(),
            text: TextRenderer::new(),
            software_surface: None,
            window: window.inner_arc(),
            frame_count: 0,
        }
    }

    /// Resize the renderer.
    pub fn resize(&mut self, size: Size) {
        #[cfg(feature = "gpu")]
        if let Some(gpu) = &mut self.gpu {
            gpu.resize(size);
        }
        self.cpu.resize(size);
        
        // Create software surface on first resize (when window is mapped)
        if self.software_surface.is_none() {
            #[cfg(feature = "gpu")]
            let should_create = self.gpu.is_none();
            #[cfg(not(feature = "gpu"))]
            let should_create = true;
            
            if should_create {
                self.software_surface = SoftwareSurface::new_from_arc(self.window.clone());
                if self.software_surface.is_some() {
                    log::info!("Software surface created for CPU rendering presentation (deferred)");
                }
            }
        }
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
        
        self.frame_count += 1;
    }
    
    /// Returns true if we need to force another paint for X11 timing workaround.
    /// Call this after end_frame() and force a repaint if it returns true.
    pub fn needs_initial_frames(&self) -> bool {
        self.frame_count <= 5
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
                DrawCommand::Path { rect, color, .. } => {
                    // TODO: SVG path rendering
                    // For now, draw a placeholder rectangle
                    self.draw_rect(*rect, *color, BorderRadius::ZERO);
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
        self.cpu.draw_text(text, position, color, size, &mut self.text);
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
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: Size,
    current_frame: Option<wgpu::SurfaceTexture>,
    background: Color,
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
            current_frame: None,
            background: Color::BLACK,
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

    pub fn begin_frame(&mut self, background: Color) {
        self.background = background;
        
        // Get the next frame
        match self.surface.get_current_texture() {
            Ok(frame) => {
                self.current_frame = Some(frame);
            }
            Err(e) => {
                log::warn!("Failed to get current texture: {}", e);
                self.current_frame = None;
            }
        }
    }

    pub fn end_frame(&mut self) {
        let Some(frame) = self.current_frame.take() else {
            return;
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let [r, g, b, a] = self.background.to_rgba8();
        let (r, g, b, a) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0, a as f64 / 255.0);
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("OpenKit Render Encoder"),
        });

        // Create render pass that clears to background color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("OpenKit Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // Render pass drops here, ending it
        }

        // Submit and present
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
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
        // Fill all pixels with background color (RGBA format)
        let pixels = &mut self.pixel_buffer.pixels;
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = a;
        }
    }

    pub fn end_frame(&mut self) {
        // Pixels are already in the buffer, nothing to do
    }

    pub fn pixels(&self) -> &[u8] {
        // PixelBuffer.pixels is already Vec<u8> in RGBA format
        &self.pixel_buffer.pixels
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius) {
        let [r, g, b, a] = color.to_rgba8();
        let _ = radius; // TODO: implement rounded corners
        
        // Manually fill pixels
        let x_start = rect.x().max(0.0) as i32;
        let y_start = rect.y().max(0.0) as i32;
        let x_end = (rect.x() + rect.width()).min(self.size.width) as i32;
        let y_end = (rect.y() + rect.height()).min(self.size.height) as i32;
        let stride = self.pixel_buffer.width;
        
        // PixelBuffer.pixels is Vec<u8> in RGBA format (4 bytes per pixel)
        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = ((y * stride + x) * 4) as usize;
                if idx + 3 < self.pixel_buffer.pixels.len() {
                    // Alpha blend if the color has transparency
                    if a < 255 {
                        let alpha = a as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;
                        self.pixel_buffer.pixels[idx] = (r as f32 * alpha + self.pixel_buffer.pixels[idx] as f32 * inv_alpha) as u8;
                        self.pixel_buffer.pixels[idx + 1] = (g as f32 * alpha + self.pixel_buffer.pixels[idx + 1] as f32 * inv_alpha) as u8;
                        self.pixel_buffer.pixels[idx + 2] = (b as f32 * alpha + self.pixel_buffer.pixels[idx + 2] as f32 * inv_alpha) as u8;
                        self.pixel_buffer.pixels[idx + 3] = 255; // Result is opaque
                    } else {
                        self.pixel_buffer.pixels[idx] = r;
                        self.pixel_buffer.pixels[idx + 1] = g;
                        self.pixel_buffer.pixels[idx + 2] = b;
                        self.pixel_buffer.pixels[idx + 3] = a;
                    }
                }
            }
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

    pub fn draw_text(&mut self, text: &str, position: Point, color: Color, size: f32, text_renderer: &mut TextRenderer) {
        if text.is_empty() {
            return;
        }
        
        let [r, g, b, a] = color.to_rgba8();
        
        // Rasterize the text using cosmic-text
        let (text_width, text_height, text_pixels) = text_renderer.rasterize(text, size, [r, g, b, a]);
        
        if text_width == 0 || text_height == 0 {
            return;
        }
        
        // Blit text pixels to the pixel buffer
        // position.y is the baseline, so offset up by most of the text height
        let dest_x = position.x as i32;
        let dest_y = (position.y - size * 0.8) as i32;
        let stride = self.pixel_buffer.width;
        let dest_pixels = &mut self.pixel_buffer.pixels;
        
        for ty in 0..text_height as i32 {
            for tx in 0..text_width as i32 {
                let src_idx = ((ty as u32 * text_width + tx as u32) * 4) as usize;
                let dx = dest_x + tx;
                let dy = dest_y + ty;
                
                if dx >= 0 && dy >= 0 && dx < stride && dy < (self.size.height as i32) {
                    let dest_idx = ((dy * stride + dx) * 4) as usize;
                    
                    if src_idx + 3 < text_pixels.len() && dest_idx + 3 < dest_pixels.len() {
                        let src_a = text_pixels[src_idx + 3] as f32 / 255.0;
                        
                        if src_a > 0.0 {
                            // Alpha blend
                            let inv_a = 1.0 - src_a;
                            dest_pixels[dest_idx] = (text_pixels[src_idx] as f32 + dest_pixels[dest_idx] as f32 * inv_a) as u8;
                            dest_pixels[dest_idx + 1] = (text_pixels[src_idx + 1] as f32 + dest_pixels[dest_idx + 1] as f32 * inv_a) as u8;
                            dest_pixels[dest_idx + 2] = (text_pixels[src_idx + 2] as f32 + dest_pixels[dest_idx + 2] as f32 * inv_a) as u8;
                            dest_pixels[dest_idx + 3] = (text_pixels[src_idx + 3] as f32 + dest_pixels[dest_idx + 3] as f32 * inv_a) as u8;
                        }
                    }
                }
            }
        }
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
