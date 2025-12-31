//! Avatar widget for displaying user images or initials.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Avatar size presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarSize {
    /// Extra small (24x24)
    XSmall,
    /// Small (32x32)
    Small,
    /// Medium (40x40) - default
    #[default]
    Medium,
    /// Large (64x64)
    Large,
    /// Extra large (96x96)
    XLarge,
    /// Custom size
    Custom(u32),
}

impl AvatarSize {
    /// Get the pixel size.
    pub fn pixels(&self) -> f32 {
        match self {
            AvatarSize::XSmall => 24.0,
            AvatarSize::Small => 32.0,
            AvatarSize::Medium => 40.0,
            AvatarSize::Large => 64.0,
            AvatarSize::XLarge => 96.0,
            AvatarSize::Custom(size) => *size as f32,
        }
    }

    /// Get the appropriate font size for initials.
    pub fn font_size(&self) -> f32 {
        self.pixels() * 0.4
    }
}

/// Avatar shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarShape {
    /// Circular avatar (default)
    #[default]
    Circle,
    /// Rounded square
    Rounded,
    /// Square with no rounding
    Square,
}

/// A circular avatar widget for displaying user images or initials.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Avatar with initials
/// let avatar = Avatar::new()
///     .initials("JD")
///     .size(AvatarSize::Large);
///
/// // Avatar with image
/// let avatar = Avatar::new()
///     .image("/path/to/image.png")
///     .fallback_initials("JD");
/// ```
pub struct Avatar {
    base: WidgetBase,
    initials: Option<String>,
    image_path: Option<String>,
    fallback_initials: Option<String>,
    size: AvatarSize,
    shape: AvatarShape,
    background_color: Option<Color>,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Avatar {
    /// Create a new avatar.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("avatar"),
            initials: None,
            image_path: None,
            fallback_initials: None,
            size: AvatarSize::default(),
            shape: AvatarShape::default(),
            background_color: None,
            on_click: None,
        }
    }

    /// Set the initials to display.
    pub fn initials(mut self, initials: impl Into<String>) -> Self {
        let text = initials.into();
        // Take first two characters, uppercase
        self.initials = Some(
            text.chars()
                .take(2)
                .collect::<String>()
                .to_uppercase()
        );
        self
    }

    /// Set the image path.
    pub fn image(mut self, path: impl Into<String>) -> Self {
        self.image_path = Some(path.into());
        self
    }

    /// Set fallback initials if image fails to load.
    pub fn fallback_initials(mut self, initials: impl Into<String>) -> Self {
        let text = initials.into();
        self.fallback_initials = Some(
            text.chars()
                .take(2)
                .collect::<String>()
                .to_uppercase()
        );
        self
    }

    /// Set the avatar size.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Set the avatar shape.
    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set a custom background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set a click handler.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Generate a background color from a string (for consistent user colors).
    fn color_from_string(s: &str) -> Color {
        // Simple hash to generate consistent colors
        let hash: u32 = s.bytes().fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u32)
        });

        // Generate pastel colors
        let hue = (hash % 360) as f32;
        let saturation: f32 = 0.5;
        let lightness: f32 = 0.6;

        // HSL to RGB conversion
        let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = lightness - c / 2.0;

        let (r, g, b): (f32, f32, f32) = match (hue / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color::rgb(r + m, g + m, b + m)
    }

    fn border_radius(&self) -> BorderRadius {
        let size = self.size.pixels();
        match self.shape {
            AvatarShape::Circle => BorderRadius::all(size / 2.0),
            AvatarShape::Rounded => BorderRadius::all(size * 0.2),
            AvatarShape::Square => BorderRadius::all(0.0),
        }
    }
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Avatar {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "avatar"
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
        let radius = self.border_radius();

        // Determine background color
        let bg_color = self.background_color.unwrap_or_else(|| {
            if let Some(ref initials) = self.initials {
                Self::color_from_string(initials)
            } else if let Some(ref fallback) = self.fallback_initials {
                Self::color_from_string(fallback)
            } else {
                theme.colors.muted
            }
        });

        // Draw background
        painter.fill_rounded_rect(rect, bg_color, radius);

        // Draw initials (if no image or image failed)
        // TODO: Add image rendering when image loading is implemented
        let text = self.initials.as_ref()
            .or(self.fallback_initials.as_ref());

        if let Some(initials) = text {
            let font_size = self.size.font_size();
            let text_width = initials.len() as f32 * font_size * 0.6;
            let text_x = rect.x() + (rect.width() - text_width) / 2.0;
            let text_y = rect.y() + (rect.height() + font_size * 0.8) / 2.0;

            painter.draw_text(
                initials,
                Point::new(text_x, text_y),
                Color::WHITE,
                font_size,
            );
        }

        // Draw border for hover/focus
        if self.base.state.hovered && self.on_click.is_some() {
            painter.stroke_rect(rect, theme.colors.ring.with_alpha(0.5), 2.0);
        }

        // Focus ring
        if self.base.state.focused {
            let ring_rect = Rect::new(
                rect.x() - 2.0,
                rect.y() - 2.0,
                rect.width() + 4.0,
                rect.height() + 4.0,
            );
            painter.stroke_rect(ring_rect, theme.colors.ring, 2.0);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if self.on_click.is_none() {
            return EventResult::Ignored;
        }

        if let Event::Mouse(mouse) = event {
            let in_bounds = self.bounds().contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Move | MouseEventKind::Enter => {
                    if in_bounds && !self.base.state.hovered {
                        self.base.state.hovered = true;
                        ctx.request_redraw();
                    } else if !in_bounds && self.base.state.hovered {
                        self.base.state.hovered = false;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Leave => {
                    if self.base.state.hovered {
                        self.base.state.hovered = false;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Up if in_bounds && self.base.state.pressed => {
                    self.base.state.pressed = false;
                    if let Some(handler) = &self.on_click {
                        handler();
                    }
                    ctx.request_redraw();
                    return EventResult::Handled;
                }
                MouseEventKind::Down if in_bounds => {
                    self.base.state.pressed = true;
                    ctx.request_redraw();
                    return EventResult::Handled;
                }
                _ => {}
            }
        }
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;
    }
}
