//! Draw call batching for efficient GPU rendering.

use super::pipeline::RectVertex;
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use wgpu::util::DeviceExt;

/// A batch of draw calls to be rendered together.
pub struct DrawBatch {
    rect_vertices: Vec<RectVertex>,
    rect_indices: Vec<u32>,
    text_commands: Vec<TextCommand>,
    image_commands: Vec<ImageCommand>,

    // Cached buffers
    rect_buffer: Option<wgpu::Buffer>,
    rect_index_buffer: Option<wgpu::Buffer>,

    // Dirty flags
    needs_rebuild: bool,
}

/// Vertex type alias for external use.
pub type BatchVertex = RectVertex;

/// A text rendering command.
#[derive(Debug, Clone)]
pub struct TextCommand {
    pub text: String,
    pub position: Point,
    pub color: Color,
    pub size: f32,
    pub font_id: u32,
}

/// An image rendering command.
#[derive(Debug, Clone)]
pub struct ImageCommand {
    pub rect: Rect,
    pub texture_id: u32,
    pub tint: Color,
    pub corner_radii: BorderRadius,
}

/// Built command buffers ready for rendering.
pub struct BuiltCommands {
    pub rect_buffer: wgpu::Buffer,
    pub rect_index_buffer: wgpu::Buffer,
    pub rect_vertices: Vec<RectVertex>,
    pub rect_index_count: u32,
}

impl DrawBatch {
    /// Create a new draw batch.
    pub fn new(_device: &wgpu::Device) -> Self {
        Self {
            rect_vertices: Vec::with_capacity(4096),
            rect_indices: Vec::with_capacity(8192),
            text_commands: Vec::with_capacity(256),
            image_commands: Vec::with_capacity(128),
            rect_buffer: None,
            rect_index_buffer: None,
            needs_rebuild: true,
        }
    }

    /// Clear all commands.
    pub fn clear(&mut self) {
        self.rect_vertices.clear();
        self.rect_indices.clear();
        self.text_commands.clear();
        self.image_commands.clear();
        self.needs_rebuild = true;
    }

    /// Add a rectangle to the batch.
    pub fn add_rect(&mut self, rect: Rect, color: Color, radius: BorderRadius, viewport: Size) {
        let base_index = self.rect_vertices.len() as u32;

        // Convert to normalized device coordinates (-1 to 1)
        let x1 = (rect.x() / viewport.width) * 2.0 - 1.0;
        let y1 = 1.0 - (rect.y() / viewport.height) * 2.0;
        let x2 = ((rect.x() + rect.width()) / viewport.width) * 2.0 - 1.0;
        let y2 = 1.0 - ((rect.y() + rect.height()) / viewport.height) * 2.0;

        let [r, g, b, a] = color.to_rgba_f32();
        let color_arr = [r, g, b, a];
        let bounds = [rect.x(), rect.y(), rect.width(), rect.height()];
        let radii = [radius.top_left, radius.top_right, radius.bottom_right, radius.bottom_left];
        let params = [0.0, 0.0, 0.0, 0.0]; // gradient_angle, border_width, flags, unused

        // Add 4 vertices (quad)
        self.rect_vertices.push(RectVertex {
            position: [x1, y1],
            uv: [0.0, 0.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });
        self.rect_vertices.push(RectVertex {
            position: [x2, y1],
            uv: [1.0, 0.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });
        self.rect_vertices.push(RectVertex {
            position: [x2, y2],
            uv: [1.0, 1.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });
        self.rect_vertices.push(RectVertex {
            position: [x1, y2],
            uv: [0.0, 1.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });

        // Add 6 indices (2 triangles)
        self.rect_indices.push(base_index);
        self.rect_indices.push(base_index + 1);
        self.rect_indices.push(base_index + 2);
        self.rect_indices.push(base_index);
        self.rect_indices.push(base_index + 2);
        self.rect_indices.push(base_index + 3);

        self.needs_rebuild = true;
    }

    /// Add a gradient rectangle to the batch.
    pub fn add_gradient_rect(
        &mut self,
        rect: Rect,
        colors: &[Color],
        angle: f32,
        radius: BorderRadius,
        viewport: Size,
    ) {
        if colors.is_empty() {
            return;
        }

        let base_index = self.rect_vertices.len() as u32;

        // Convert to normalized device coordinates
        let x1 = (rect.x() / viewport.width) * 2.0 - 1.0;
        let y1 = 1.0 - (rect.y() / viewport.height) * 2.0;
        let x2 = ((rect.x() + rect.width()) / viewport.width) * 2.0 - 1.0;
        let y2 = 1.0 - ((rect.y() + rect.height()) / viewport.height) * 2.0;

        // Use first color for now (full gradient requires the gradient shader)
        let color = colors[0];
        let [r, g, b, a] = color.to_rgba_f32();
        let color_arr = [r, g, b, a];
        let bounds = [rect.x(), rect.y(), rect.width(), rect.height()];
        let radii = [radius.top_left, radius.top_right, radius.bottom_right, radius.bottom_left];
        let params = [angle, 0.0, 1.0, 0.0]; // gradient_angle, border_width, is_gradient, unused

        self.rect_vertices.push(RectVertex {
            position: [x1, y1],
            uv: [0.0, 0.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });
        self.rect_vertices.push(RectVertex {
            position: [x2, y1],
            uv: [1.0, 0.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });
        self.rect_vertices.push(RectVertex {
            position: [x2, y2],
            uv: [1.0, 1.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });
        self.rect_vertices.push(RectVertex {
            position: [x1, y2],
            uv: [0.0, 1.0],
            color: color_arr,
            rect_bounds: bounds,
            corner_radii: radii,
            params,
        });

        self.rect_indices.push(base_index);
        self.rect_indices.push(base_index + 1);
        self.rect_indices.push(base_index + 2);
        self.rect_indices.push(base_index);
        self.rect_indices.push(base_index + 2);
        self.rect_indices.push(base_index + 3);

        self.needs_rebuild = true;
    }

    /// Add a text command to the batch.
    pub fn add_text(
        &mut self,
        text: &str,
        position: Point,
        color: Color,
        size: f32,
        font_id: u32,
        _viewport: Size,
    ) {
        self.text_commands.push(TextCommand {
            text: text.to_string(),
            position,
            color,
            size,
            font_id,
        });
    }

    /// Add an image command to the batch.
    pub fn add_image(&mut self, rect: Rect, texture_id: u32, _viewport: Size) {
        self.image_commands.push(ImageCommand {
            rect,
            texture_id,
            tint: Color::WHITE,
            corner_radii: BorderRadius::ZERO,
        });
    }

    /// Build GPU buffers from the batched commands.
    pub fn build(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue) -> BuiltCommands {
        // Create vertex buffer
        let rect_buffer = if !self.rect_vertices.is_empty() {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Rect Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.rect_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        } else {
            // Empty buffer
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Rect Vertex Buffer"),
                size: 64,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        };

        // Create index buffer
        let rect_index_buffer = if !self.rect_indices.is_empty() {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Rect Index Buffer"),
                contents: bytemuck::cast_slice(&self.rect_indices),
                usage: wgpu::BufferUsages::INDEX,
            })
        } else {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty Rect Index Buffer"),
                size: 64,
                usage: wgpu::BufferUsages::INDEX,
                mapped_at_creation: false,
            })
        };

        BuiltCommands {
            rect_buffer,
            rect_index_buffer,
            rect_vertices: self.rect_vertices.clone(),
            rect_index_count: self.rect_indices.len() as u32,
        }
    }

    /// Get the number of queued rectangles.
    pub fn rect_count(&self) -> usize {
        self.rect_vertices.len() / 4
    }

    /// Get the number of queued text commands.
    pub fn text_count(&self) -> usize {
        self.text_commands.len()
    }

    /// Get the number of queued image commands.
    pub fn image_count(&self) -> usize {
        self.image_commands.len()
    }
}

impl Default for DrawBatch {
    fn default() -> Self {
        Self {
            rect_vertices: Vec::new(),
            rect_indices: Vec::new(),
            text_commands: Vec::new(),
            image_commands: Vec::new(),
            rect_buffer: None,
            rect_index_buffer: None,
            needs_rebuild: true,
        }
    }
}
