//! Label widget for displaying text.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext};
use crate::css::{ClassList, ComputedStyle, StyleContext, WidgetState};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A text label widget.
pub struct Label {
    base: WidgetBase,
    text: String,
    computed_style: Option<ComputedStyle>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new().with_class("label"),
            text: text.into(),
            computed_style: None,
        }
    }

    /// Set the text content.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Set the element ID.
    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }

    /// Get the text content.
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Set the text content (mutably).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
    
    /// Get font size based on CSS class (built-in typography support).
    fn font_size_for_class(&self) -> Option<f32> {
        let classes = &self.base.classes;
        
        // Tailwind-inspired typography classes (check largest to smallest)
        if classes.contains("hero-title") || classes.contains("text-4xl") {
            Some(36.0)
        } else if classes.contains("title") || classes.contains("text-3xl") || classes.contains("h1") {
            Some(30.0)
        } else if classes.contains("heading") || classes.contains("text-2xl") || classes.contains("h2") {
            Some(24.0)
        } else if classes.contains("subheading") || classes.contains("text-xl") || classes.contains("h3") {
            Some(20.0)
        } else if classes.contains("text-lg") {
            Some(18.0)
        } else if classes.contains("text-sm") || classes.contains("subtitle") {
            Some(14.0)
        } else if classes.contains("text-xs") || classes.contains("section-title") || classes.contains("caption") {
            Some(12.0)
        } else {
            None
        }
    }
    
    /// Get opacity based on CSS class.
    fn opacity_for_class(&self) -> f32 {
        let classes = &self.base.classes;
        
        if classes.contains("subtitle") || classes.contains("muted") {
            0.7
        } else if classes.contains("section-title") || classes.contains("caption") {
            0.5
        } else {
            1.0
        }
    }
}

impl Widget for Label {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "label"
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
        // Use class-based font size or default
        let font_size = self.font_size_for_class()
            .or_else(|| self.computed_style.as_ref().map(|s| s.font_size))
            .unwrap_or(16.0);
        let char_width = font_size * 0.6; // Approximate average character width
        let width = self.text.len() as f32 * char_width;
        let height = font_size * 1.5; // Line height
        Size::new(width, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let intrinsic = self.intrinsic_size(ctx);
        let size = constraints.constrain(intrinsic);
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        // Use class-based font size or computed/default
        let font_size = self.font_size_for_class()
            .or_else(|| self.computed_style.as_ref().map(|s| s.font_size))
            .unwrap_or(16.0);
        
        // Base color from theme
        let base_color = self.computed_style.as_ref()
            .map(|s| s.color)
            .unwrap_or(ctx.style_ctx.theme.colors.foreground);
        
        // Apply opacity for muted text
        let opacity = self.opacity_for_class();
        let color = if opacity < 1.0 {
            base_color.with_alpha(opacity)
        } else {
            base_color
        };

        // Draw text at the baseline
        let baseline_y = rect.y() + font_size;
        painter.draw_text(&self.text, Point::new(rect.x(), baseline_y), color, font_size);
    }

    fn style(&self, ctx: &StyleContext) -> ComputedStyle {
        ComputedStyle {
            color: ctx.theme.colors.foreground,
            font_size: ctx.theme.typography.base_size,
            line_height: ctx.theme.typography.line_height,
            ..Default::default()
        }
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;
    }
}
