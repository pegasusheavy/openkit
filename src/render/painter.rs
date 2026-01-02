//! Painter API for drawing primitives.
//!
//! # Performance
//!
//! The Painter is optimized for speed:
//! - Uses SmallVec for clip/transform stacks to avoid heap allocation
//! - Pre-allocates command buffer with reasonable capacity
//! - Inline hints on hot paths

use smallvec::SmallVec;
use crate::geometry::{BorderRadius, Color, Point, Rect};

/// Default capacity for command buffer
const DEFAULT_COMMAND_CAPACITY: usize = 256;

/// A painter for drawing primitives.
#[derive(Debug)]
pub struct Painter {
    commands: Vec<DrawCommand>,
    clip_stack: SmallVec<[Rect; 8]>,
    transform_stack: SmallVec<[Transform; 8]>,
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

impl Painter {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_COMMAND_CAPACITY)
    }

    /// Create a painter with a specific command buffer capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut transform_stack = SmallVec::new();
        transform_stack.push(Transform::identity());
        
        Self {
            commands: Vec::with_capacity(capacity),
            clip_stack: SmallVec::new(),
            transform_stack,
        }
    }

    /// Get the accumulated draw commands.
    #[inline]
    pub fn finish(self) -> Vec<DrawCommand> {
        self.commands
    }

    /// Clear all commands.
    #[inline]
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Number of commands in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the command buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Draw a filled rectangle.
    #[inline]
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.fill_rounded_rect(rect, color, BorderRadius::ZERO);
    }

    /// Draw a filled rounded rectangle.
    #[inline]
    pub fn fill_rounded_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius) {
        let rect = self.transform_rect(rect);
        self.commands.push(DrawCommand::Rect { rect, color, radius });
    }

    /// Draw a stroked rectangle.
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32) {
        let rect = self.transform_rect(rect);
        // Draw four lines for the stroke
        self.draw_line(
            Point::new(rect.x(), rect.y()),
            Point::new(rect.max_x(), rect.y()),
            color,
            width,
        );
        self.draw_line(
            Point::new(rect.max_x(), rect.y()),
            Point::new(rect.max_x(), rect.max_y()),
            color,
            width,
        );
        self.draw_line(
            Point::new(rect.max_x(), rect.max_y()),
            Point::new(rect.x(), rect.max_y()),
            color,
            width,
        );
        self.draw_line(
            Point::new(rect.x(), rect.max_y()),
            Point::new(rect.x(), rect.y()),
            color,
            width,
        );
    }

    /// Draw a stroked rounded rectangle.
    pub fn stroke_rounded_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius, width: f32) {
        let rect = self.transform_rect(rect);
        self.commands.push(DrawCommand::StrokeRoundedRect { rect, color, radius, width });
    }

    /// Draw a line.
    #[inline]
    pub fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let from = self.transform_point(from);
        let to = self.transform_point(to);
        self.commands.push(DrawCommand::Line { from, to, color, width });
    }

    /// Draw text.
    #[inline]
    pub fn draw_text(&mut self, text: &str, position: Point, color: Color, size: f32) {
        let position = self.transform_point(position);
        self.commands.push(DrawCommand::Text {
            text: text.to_string(),
            position,
            color,
            size,
        });
    }

    /// Draw an image.
    #[inline]
    pub fn draw_image(&mut self, rect: Rect, image_id: u64) {
        let rect = self.transform_rect(rect);
        self.commands.push(DrawCommand::Image { rect, image_id });
    }

    /// Draw an SVG path (Font Awesome icon style).
    /// The path string uses SVG path syntax (M, L, C, Z, etc.)
    /// The viewbox defines the source coordinate system.
    /// The rect defines where to draw the icon.
    #[inline]
    pub fn draw_path(&mut self, path: &str, rect: Rect, color: Color, viewbox: (f32, f32, f32, f32)) {
        let rect = self.transform_rect(rect);
        self.commands.push(DrawCommand::Path {
            path: path.to_string(),
            rect,
            color,
            viewbox,
        });
    }

    /// Draw an icon from path data at the specified position and size.
    #[inline]
    pub fn draw_icon(&mut self, path: &str, position: Point, size: f32, color: Color, viewbox: (f32, f32, f32, f32)) {
        let rect = Rect::new(position.x, position.y, size, size);
        self.draw_path(path, rect, color, viewbox);
    }

    /// Push a clip rectangle.
    #[inline]
    pub fn push_clip(&mut self, rect: Rect) {
        let rect = self.transform_rect(rect);
        self.clip_stack.push(rect);
    }

    /// Pop the clip rectangle.
    #[inline]
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Save the current transform.
    #[inline]
    pub fn save(&mut self) {
        if let Some(current) = self.transform_stack.last() {
            self.transform_stack.push(*current);
        }
    }

    /// Restore the previous transform.
    #[inline]
    pub fn restore(&mut self) {
        if self.transform_stack.len() > 1 {
            self.transform_stack.pop();
        }
    }

    /// Translate the coordinate system.
    #[inline]
    pub fn translate(&mut self, dx: f32, dy: f32) {
        if let Some(transform) = self.transform_stack.last_mut() {
            transform.tx += dx;
            transform.ty += dy;
        }
    }

    /// Scale the coordinate system.
    #[inline]
    pub fn scale(&mut self, sx: f32, sy: f32) {
        if let Some(transform) = self.transform_stack.last_mut() {
            transform.sx *= sx;
            transform.sy *= sy;
        }
    }

    fn transform_point(&self, point: Point) -> Point {
        if let Some(t) = self.transform_stack.last() {
            Point::new(point.x * t.sx + t.tx, point.y * t.sy + t.ty)
        } else {
            point
        }
    }

    fn transform_rect(&self, rect: Rect) -> Rect {
        let origin = self.transform_point(rect.origin);
        if let Some(t) = self.transform_stack.last() {
            Rect::new(origin.x, origin.y, rect.width() * t.sx, rect.height() * t.sy)
        } else {
            Rect::from_origin_size(origin, rect.size)
        }
    }
}

/// Draw commands.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    Rect {
        rect: Rect,
        color: Color,
        radius: BorderRadius,
    },
    StrokeRoundedRect {
        rect: Rect,
        color: Color,
        radius: BorderRadius,
        width: f32,
    },
    Text {
        text: String,
        position: Point,
        color: Color,
        size: f32,
    },
    Line {
        from: Point,
        to: Point,
        color: Color,
        width: f32,
    },
    Image {
        rect: Rect,
        image_id: u64,
    },
    /// SVG path command for icons
    Path {
        path: String,
        rect: Rect,
        color: Color,
        /// (min_x, min_y, width, height) of the SVG viewbox
        viewbox: (f32, f32, f32, f32),
    },
}

/// A 2D transform (translation + scale).
#[derive(Debug, Clone, Copy)]
struct Transform {
    tx: f32,
    ty: f32,
    sx: f32,
    sy: f32,
}

impl Transform {
    fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}
