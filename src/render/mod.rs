//! Rendering engine for OpenKit.
//!
//! Provides GPU-accelerated rendering using wgpu with a CPU fallback using tiny-skia.
//!
//! # GPU Acceleration
//!
//! The GPU renderer provides:
//! - Batched draw calls for minimal GPU overhead
//! - Custom WGSL shaders for primitives (rectangles, rounded rectangles, gradients)
//! - Texture atlas for efficient text and image rendering
//! - GPU-accelerated effects (blur, shadow, glow)
//! - Multi-sample anti-aliasing (MSAA)
//! - HDR support (when available)
//!
//! # Performance Tips
//!
//! - Draw calls are automatically batched - draw similar elements together
//! - Use the texture atlas for frequently-used images
//! - Minimize effect usage (blur, shadow) as they require extra passes

mod painter;
mod text;

#[cfg(feature = "gpu")]
pub mod gpu;

pub use painter::{Painter, DrawCommand};
pub use text::TextRenderer;

#[cfg(feature = "gpu")]
pub use gpu::{GpuRenderer, GpuConfig, GpuError, RenderStats, PowerPreference};

use crate::geometry::{Color, Point, Rect, Size, BorderRadius};
use crate::platform::Window;

/// The main renderer.
pub struct Renderer {
    #[cfg(feature = "gpu")]
    gpu: Option<GpuRenderer>,
    cpu: CpuRenderer,
    text: TextRenderer,
}

impl Renderer {
    /// Create a new renderer for a window.
    pub fn new(window: &Window) -> Self {
        #[cfg(feature = "gpu")]
        let gpu = GpuRenderer::new(window, GpuConfig::default()).ok();

        Self {
            #[cfg(feature = "gpu")]
            gpu,
            cpu: CpuRenderer::new(),
            text: TextRenderer::new(),
        }
    }

    /// Create a new renderer with custom GPU configuration.
    #[cfg(feature = "gpu")]
    pub fn with_config(window: &Window, config: GpuConfig) -> Self {
        let gpu = GpuRenderer::new(window, config).ok();

        Self {
            gpu,
            cpu: CpuRenderer::new(),
            text: TextRenderer::new(),
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
            let _ = gpu.begin_frame(background);
            return;
        }
        self.cpu.begin_frame(background);
    }

    /// End the frame and present.
    pub fn end_frame(&mut self) {
        #[cfg(feature = "gpu")]
        if let Some(gpu) = &mut self.gpu {
            let _ = gpu.end_frame();
            return;
        }
        self.cpu.end_frame();
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
                DrawCommand::StrokeRoundedRect { rect, color, radius, width } => {
                    self.stroke_rounded_rect(*rect, *color, *radius, *width);
                }
                DrawCommand::Path { path, rect, color, viewbox } => {
                    self.draw_path(path, *rect, *color, *viewbox);
                }
            }
        }
    }

    /// Draw an SVG path scaled to fit within the given rect.
    pub fn draw_path(&mut self, path: &str, rect: Rect, color: Color, viewbox: (f32, f32, f32, f32)) {
        self.cpu.draw_path(path, rect, color, viewbox);
    }

    /// Draw a stroked rounded rectangle.
    pub fn stroke_rounded_rect(&mut self, rect: Rect, color: Color, _radius: BorderRadius, width: f32) {
        // For now, use stroke_rect as fallback
        // TODO: Implement proper rounded rect stroking in CPU/GPU backends
        self.stroke_rect(rect, color, width);
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

    /// Draw a stroked rectangle.
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32) {
        // Draw four lines for the stroke
        let x = rect.x();
        let y = rect.y();
        let max_x = rect.max_x();
        let max_y = rect.max_y();
        self.draw_line(Point::new(x, y), Point::new(max_x, y), color, width);
        self.draw_line(Point::new(max_x, y), Point::new(max_x, max_y), color, width);
        self.draw_line(Point::new(max_x, max_y), Point::new(x, max_y), color, width);
        self.draw_line(Point::new(x, max_y), Point::new(x, y), color, width);
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

/// CPU renderer using tiny-skia.
pub struct CpuRenderer {
    pixmap: tiny_skia::Pixmap,
    size: Size,
}

impl CpuRenderer {
    pub fn new() -> Self {
        Self {
            pixmap: tiny_skia::Pixmap::new(1, 1).unwrap(),
            size: Size::new(1.0, 1.0),
        }
    }

    pub fn resize(&mut self, size: Size) {
        if size.width > 0.0 && size.height > 0.0 {
            self.size = size;
            self.pixmap = tiny_skia::Pixmap::new(size.width as u32, size.height as u32)
                .unwrap_or_else(|| tiny_skia::Pixmap::new(1, 1).unwrap());
        }
    }

    pub fn begin_frame(&mut self, background: Color) {
        let [r, g, b, a] = background.to_rgba8();
        self.pixmap.fill(tiny_skia::Color::from_rgba8(r, g, b, a));
    }

    pub fn end_frame(&mut self) {
        // Nothing to do for CPU renderer
    }

    pub fn pixels(&self) -> &[u8] {
        self.pixmap.data()
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius) {
        let [r, g, b, a] = color.to_rgba8();
        let paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(r, g, b, a)),
            anti_alias: true,
            ..Default::default()
        };

        if radius.is_zero() {
            // Simple rectangle
            let rect = tiny_skia::Rect::from_xywh(
                rect.x(),
                rect.y(),
                rect.width(),
                rect.height(),
            );
            if let Some(rect) = rect {
                self.pixmap.fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
            }
        } else {
            // Rounded rectangle
            let path = create_rounded_rect_path(rect, radius);
            self.pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None);
        }
    }

    pub fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let [r, g, b, a] = color.to_rgba8();
        let paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(r, g, b, a)),
            anti_alias: true,
            ..Default::default()
        };

        let mut path = tiny_skia::PathBuilder::new();
        path.move_to(from.x, from.y);
        path.line_to(to.x, to.y);

        if let Some(path) = path.finish() {
            let stroke = tiny_skia::Stroke {
                width,
                ..Default::default()
            };
            self.pixmap.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
        }
    }

    pub fn draw_text(&mut self, text: &str, position: Point, color: Color, size: f32, text_renderer: &TextRenderer) {
        // Simple text rendering - in production would use cosmic-text for proper glyph rendering
        // For now, just draw a placeholder
        let [r, g, b, a] = color.to_rgba8();
        let _paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(r, g, b, a)),
            anti_alias: true,
            ..Default::default()
        };

        // Draw a simple placeholder rectangle for text
        let text_size = text_renderer.measure(text, size);
        let rect = tiny_skia::Rect::from_xywh(
            position.x,
            position.y - size * 0.8, // Approximate baseline offset
            text_size.width,
            text_size.height,
        );

        // For MVP, we'll render text using cosmic-text's rasterization
        // This is a simplified placeholder
        if let Some(_rect) = rect {
            // In a full implementation, we'd rasterize glyphs here
            // For now, just indicate text area
        }
    }

    /// Draw an SVG path scaled to fit within the given rect.
    /// Parses basic SVG path commands: M, L, C, Q, Z (and lowercase variants).
    pub fn draw_path(&mut self, path_str: &str, rect: Rect, color: Color, viewbox: (f32, f32, f32, f32)) {
        let [r, g, b, a] = color.to_rgba8();
        let paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(r, g, b, a)),
            anti_alias: true,
            ..Default::default()
        };

        // Calculate scale and offset to fit path in rect
        let (vb_x, vb_y, vb_w, vb_h) = viewbox;
        let scale_x = rect.width() / vb_w;
        let scale_y = rect.height() / vb_h;
        let scale = scale_x.min(scale_y);
        let offset_x = rect.x() - vb_x * scale + (rect.width() - vb_w * scale) / 2.0;
        let offset_y = rect.y() - vb_y * scale + (rect.height() - vb_h * scale) / 2.0;

        if let Some(path) = parse_svg_path(path_str) {
            let transform = tiny_skia::Transform::from_scale(scale, scale)
                .post_translate(offset_x, offset_y);
            self.pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, transform, None);
        }
    }
}

/// Parse a simplified SVG path string into a tiny_skia::Path.
/// Supports: M/m, L/l, H/h, V/v, C/c, Q/q, A/a, Z/z
fn parse_svg_path(path_str: &str) -> Option<tiny_skia::Path> {
    let mut pb = tiny_skia::PathBuilder::new();
    let mut current_x = 0.0f32;
    let mut current_y = 0.0f32;
    let mut start_x = 0.0f32;
    let mut start_y = 0.0f32;

    // Tokenize: split on commands while keeping the command letter
    let mut chars = path_str.chars().peekable();
    let mut current_cmd = ' ';

    while chars.peek().is_some() {
        // Skip whitespace and commas
        while chars.peek().map(|c| c.is_whitespace() || *c == ',').unwrap_or(false) {
            chars.next();
        }

        // Check for command letter
        if let Some(&c) = chars.peek() {
            if c.is_ascii_alphabetic() {
                current_cmd = c;
                chars.next();
                continue;
            }
        }

        // Parse numbers based on current command
        let mut nums = Vec::new();
        let count_needed = match current_cmd.to_ascii_uppercase() {
            'M' | 'L' => 2,
            'H' | 'V' => 1,
            'C' => 6,
            'S' => 4,
            'Q' => 4,
            'T' => 2,
            'A' => 7,
            'Z' => 0,
            _ => 0,
        };

        for _ in 0..count_needed {
            // Skip whitespace and commas
            while chars.peek().map(|c| c.is_whitespace() || *c == ',').unwrap_or(false) {
                chars.next();
            }

            // Parse number
            let mut num_str = String::new();
            if chars.peek() == Some(&'-') || chars.peek() == Some(&'+') {
                num_str.push(chars.next().unwrap());
            }
            while chars.peek().map(|c| c.is_ascii_digit() || *c == '.').unwrap_or(false) {
                num_str.push(chars.next().unwrap());
            }
            // Handle scientific notation
            if chars.peek() == Some(&'e') || chars.peek() == Some(&'E') {
                num_str.push(chars.next().unwrap());
                if chars.peek() == Some(&'-') || chars.peek() == Some(&'+') {
                    num_str.push(chars.next().unwrap());
                }
                while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    num_str.push(chars.next().unwrap());
                }
            }
            if let Ok(n) = num_str.parse::<f32>() {
                nums.push(n);
            }
        }

        let is_relative = current_cmd.is_ascii_lowercase();
        match current_cmd.to_ascii_uppercase() {
            'M' if nums.len() >= 2 => {
                let (x, y) = if is_relative {
                    (current_x + nums[0], current_y + nums[1])
                } else {
                    (nums[0], nums[1])
                };
                pb.move_to(x, y);
                current_x = x;
                current_y = y;
                start_x = x;
                start_y = y;
            }
            'L' if nums.len() >= 2 => {
                let (x, y) = if is_relative {
                    (current_x + nums[0], current_y + nums[1])
                } else {
                    (nums[0], nums[1])
                };
                pb.line_to(x, y);
                current_x = x;
                current_y = y;
            }
            'H' if nums.len() >= 1 => {
                let x = if is_relative { current_x + nums[0] } else { nums[0] };
                pb.line_to(x, current_y);
                current_x = x;
            }
            'V' if nums.len() >= 1 => {
                let y = if is_relative { current_y + nums[0] } else { nums[0] };
                pb.line_to(current_x, y);
                current_y = y;
            }
            'C' if nums.len() >= 6 => {
                let (x1, y1, x2, y2, x, y) = if is_relative {
                    (current_x + nums[0], current_y + nums[1],
                     current_x + nums[2], current_y + nums[3],
                     current_x + nums[4], current_y + nums[5])
                } else {
                    (nums[0], nums[1], nums[2], nums[3], nums[4], nums[5])
                };
                pb.cubic_to(x1, y1, x2, y2, x, y);
                current_x = x;
                current_y = y;
            }
            'Q' if nums.len() >= 4 => {
                let (x1, y1, x, y) = if is_relative {
                    (current_x + nums[0], current_y + nums[1],
                     current_x + nums[2], current_y + nums[3])
                } else {
                    (nums[0], nums[1], nums[2], nums[3])
                };
                pb.quad_to(x1, y1, x, y);
                current_x = x;
                current_y = y;
            }
            'A' if nums.len() >= 7 => {
                // Arc: rx, ry, x-axis-rotation, large-arc-flag, sweep-flag, x, y
                // tiny_skia doesn't have direct arc support, approximate with lines for now
                let (x, y) = if is_relative {
                    (current_x + nums[5], current_y + nums[6])
                } else {
                    (nums[5], nums[6])
                };
                // Fallback: just draw a line to the endpoint
                pb.line_to(x, y);
                current_x = x;
                current_y = y;
            }
            'Z' => {
                pb.close();
                current_x = start_x;
                current_y = start_y;
            }
            _ => {}
        }
    }

    pb.finish()
}

impl Default for CpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a rounded rectangle path.
fn create_rounded_rect_path(rect: Rect, radius: BorderRadius) -> tiny_skia::Path {
    let mut pb = tiny_skia::PathBuilder::new();

    let x = rect.x();
    let y = rect.y();
    let w = rect.width();
    let h = rect.height();

    let tl = radius.top_left.min(w / 2.0).min(h / 2.0);
    let tr = radius.top_right.min(w / 2.0).min(h / 2.0);
    let br = radius.bottom_right.min(w / 2.0).min(h / 2.0);
    let bl = radius.bottom_left.min(w / 2.0).min(h / 2.0);

    // Top edge
    pb.move_to(x + tl, y);
    pb.line_to(x + w - tr, y);

    // Top right corner
    if tr > 0.0 {
        pb.quad_to(x + w, y, x + w, y + tr);
    }

    // Right edge
    pb.line_to(x + w, y + h - br);

    // Bottom right corner
    if br > 0.0 {
        pb.quad_to(x + w, y + h, x + w - br, y + h);
    }

    // Bottom edge
    pb.line_to(x + bl, y + h);

    // Bottom left corner
    if bl > 0.0 {
        pb.quad_to(x, y + h, x, y + h - bl);
    }

    // Left edge
    pb.line_to(x, y + tl);

    // Top left corner
    if tl > 0.0 {
        pb.quad_to(x, y, x + tl, y);
    }

    pb.close();
    pb.finish().unwrap_or_else(|| tiny_skia::PathBuilder::new().finish().unwrap())
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
