//! Loading spinner widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Spinner size presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpinnerSize {
    /// Small (16x16)
    Small,
    /// Medium (24x24) - default
    #[default]
    Medium,
    /// Large (32x32)
    Large,
    /// Extra large (48x48)
    XLarge,
}

impl SpinnerSize {
    /// Get the pixel size.
    pub fn pixels(&self) -> f32 {
        match self {
            SpinnerSize::Small => 16.0,
            SpinnerSize::Medium => 24.0,
            SpinnerSize::Large => 32.0,
            SpinnerSize::XLarge => 48.0,
        }
    }

    /// Get the stroke width.
    pub fn stroke_width(&self) -> f32 {
        match self {
            SpinnerSize::Small => 2.0,
            SpinnerSize::Medium => 3.0,
            SpinnerSize::Large => 4.0,
            SpinnerSize::XLarge => 5.0,
        }
    }
}

/// A loading spinner widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Basic spinner
/// let spinner = Spinner::new();
///
/// // Large spinner with custom color
/// let spinner = Spinner::new()
///     .size(SpinnerSize::Large)
///     .color(Color::from_hex("#3b82f6").unwrap());
/// ```
pub struct Spinner {
    base: WidgetBase,
    size: SpinnerSize,
    color: Option<Color>,
    #[allow(dead_code)]
    rotation: f32,
}

impl Spinner {
    /// Create a new spinner.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("spinner"),
            size: SpinnerSize::default(),
            color: None,
            rotation: 0.0,
        }
    }

    /// Set the spinner size.
    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }

    /// Set a custom color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "spinner"
    }

    fn element_id(&self) -> Option<&str> {
        self.base.element_id.as_deref()
    }

    fn classes(&self) -> &ClassList {
        &self.base.classes
    }

    fn state(&self) -> WidgetState {
        self.base.state
    }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        let size = self.size.pixels();
        Size::new(size, size)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let color = self.color.unwrap_or(theme.colors.primary);

        let size = self.size.pixels();
        let stroke = self.size.stroke_width();
        let center_x = rect.x() + rect.width() / 2.0;
        let center_y = rect.y() + rect.height() / 2.0;
        let radius = (size - stroke) / 2.0;

        // Draw background circle (track)
        let _track_color = color.with_alpha(0.2);

        // Draw a simple circular representation
        // In a real implementation, we'd draw proper arcs
        // For now, we'll draw a simplified version with multiple segments

        let segments = 12;
        for i in 0..segments {
            let angle = (i as f32) * (std::f32::consts::PI * 2.0 / segments as f32);
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;

            // Fade segments to create spinner effect
            let alpha = ((i as f32 / segments as f32) * 0.8 + 0.2).min(1.0);
            let segment_color = color.with_alpha(alpha);

            painter.fill_rect(
                Rect::new(x - stroke / 2.0, y - stroke / 2.0, stroke, stroke),
                segment_color,
            );
        }

        // Draw a simple loading indicator using text (temporary)
        // This will be replaced with proper arc rendering
        painter.draw_text(
            "⟳",
            Point::new(
                center_x - size * 0.3,
                center_y + size * 0.25,
            ),
            color,
            size * 0.8,
        );
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        // Spinners don't handle events
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;
    }
}
