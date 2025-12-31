//! Painter API for drawing primitives.

use crate::geometry::{BorderRadius, Color, Point, Rect};

/// A painter for drawing primitives.
#[derive(Debug, Default)]
pub struct Painter {
    commands: Vec<DrawCommand>,
    clip_stack: Vec<Rect>,
    transform_stack: Vec<Transform>,
}

impl Painter {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            clip_stack: Vec::new(),
            transform_stack: vec![Transform::identity()],
        }
    }

    /// Get the accumulated draw commands.
    pub fn finish(self) -> Vec<DrawCommand> {
        self.commands
    }

    /// Clear all commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Draw a filled rectangle.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.fill_rounded_rect(rect, color, BorderRadius::ZERO);
    }

    /// Draw a filled rounded rectangle.
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

    /// Draw a line.
    pub fn draw_line(&mut self, from: Point, to: Point, color: Color, width: f32) {
        let from = self.transform_point(from);
        let to = self.transform_point(to);
        self.commands.push(DrawCommand::Line { from, to, color, width });
    }

    /// Draw text.
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
    pub fn draw_image(&mut self, rect: Rect, image_id: u64) {
        let rect = self.transform_rect(rect);
        self.commands.push(DrawCommand::Image { rect, image_id });
    }

    /// Push a clip rectangle.
    pub fn push_clip(&mut self, rect: Rect) {
        let rect = self.transform_rect(rect);
        self.clip_stack.push(rect);
    }

    /// Pop the clip rectangle.
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Save the current transform.
    pub fn save(&mut self) {
        if let Some(current) = self.transform_stack.last() {
            self.transform_stack.push(*current);
        }
    }

    /// Restore the previous transform.
    pub fn restore(&mut self) {
        if self.transform_stack.len() > 1 {
            self.transform_stack.pop();
        }
    }

    /// Translate the coordinate system.
    pub fn translate(&mut self, dx: f32, dy: f32) {
        if let Some(transform) = self.transform_stack.last_mut() {
            transform.tx += dx;
            transform.ty += dy;
        }
    }

    /// Scale the coordinate system.
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
