//! Glass pane / blur container widget
//!
//! A container with background blur and transparency effects.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Blur intensity
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BlurIntensity {
    None,
    Light,
    #[default]
    Medium,
    Heavy,
    Custom(f32),
}

impl BlurIntensity {
    pub fn radius(&self) -> f32 {
        match self {
            BlurIntensity::None => 0.0,
            BlurIntensity::Light => 5.0,
            BlurIntensity::Medium => 10.0,
            BlurIntensity::Heavy => 20.0,
            BlurIntensity::Custom(r) => *r,
        }
    }
}

/// Glass pane / blur container
pub struct GlassPane {
    base: WidgetBase,
    background: Color,
    blur: BlurIntensity,
    border_color: Option<Color>,
    border_width: f32,
    border_radius: f32,
    padding: f32,
    vibrancy: bool,
    noise: f32,
}

impl GlassPane {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("glass-pane"),
            background: Color::rgba(0.1, 0.1, 0.1, 0.7),
            blur: BlurIntensity::Medium,
            border_color: Some(Color::rgba(1.0, 1.0, 1.0, 0.1)),
            border_width: 1.0,
            border_radius: 12.0,
            padding: 16.0,
            vibrancy: true,
            noise: 0.02,
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn blur(mut self, blur: BlurIntensity) -> Self {
        self.blur = blur;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn no_border(mut self) -> Self {
        self.border_color = None;
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn vibrancy(mut self, enabled: bool) -> Self {
        self.vibrancy = enabled;
        self
    }

    pub fn noise(mut self, intensity: f32) -> Self {
        self.noise = intensity.clamp(0.0, 1.0);
        self
    }

    pub fn dark() -> Self {
        Self::new().background(Color::rgba(0.0, 0.0, 0.0, 0.6))
    }

    pub fn light() -> Self {
        Self::new()
            .background(Color::rgba(1.0, 1.0, 1.0, 0.6))
            .border_color(Color::rgba(0.0, 0.0, 0.0, 0.1))
    }

    pub fn colored(color: Color, opacity: f32) -> Self {
        Self::new().background(Color::rgba(color.r, color.g, color.b, opacity))
    }

    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }
}

impl Default for GlassPane {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for GlassPane {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "glass-pane"
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
        Size::new(self.padding * 2.0, self.padding * 2.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        // Background
        painter.fill_rounded_rect(rect, self.background, BorderRadius::all(self.border_radius));

        // Border
        if let Some(border) = self.border_color {
            painter.stroke_rect(rect, border, self.border_width);
        }

        // Inner highlight (top edge)
        let highlight_rect = Rect::new(
            rect.x() + self.border_radius,
            rect.y() + 1.0,
            rect.width() - self.border_radius * 2.0,
            1.0,
        );
        painter.fill_rect(highlight_rect, Color::rgba(1.0, 1.0, 1.0, 0.1));
    }

    fn hit_test(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;
    }
}
