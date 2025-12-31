//! Icon button widget for actions with icons.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Icon button size presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonSize {
    /// Small (32x32)
    Small,
    /// Medium (40x40) - default
    #[default]
    Medium,
    /// Large (48x48)
    Large,
}

impl IconButtonSize {
    /// Get the pixel size.
    pub fn pixels(&self) -> f32 {
        match self {
            IconButtonSize::Small => 32.0,
            IconButtonSize::Medium => 40.0,
            IconButtonSize::Large => 48.0,
        }
    }

    /// Get the icon size.
    pub fn icon_size(&self) -> f32 {
        match self {
            IconButtonSize::Small => 16.0,
            IconButtonSize::Medium => 20.0,
            IconButtonSize::Large => 24.0,
        }
    }
}

/// Icon button variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonVariant {
    /// Default/ghost style (transparent background)
    #[default]
    Ghost,
    /// Filled background
    Filled,
    /// Outlined with border
    Outline,
    /// Destructive action (red)
    Destructive,
}

/// A button that displays an icon.
///
/// Used for toolbar actions, close buttons, power buttons, etc.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let power_btn = IconButton::new("⏻")
///     .tooltip("Shutdown")
///     .variant(IconButtonVariant::Destructive)
///     .on_click(|| {
///         println!("Shutdown clicked");
///     });
/// ```
pub struct IconButton {
    base: WidgetBase,
    /// Icon character or emoji
    icon: String,
    tooltip: Option<String>,
    size: IconButtonSize,
    variant: IconButtonVariant,
    disabled: bool,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl IconButton {
    /// Create a new icon button.
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new().with_class("icon-button"),
            icon: icon.into(),
            tooltip: None,
            size: IconButtonSize::default(),
            variant: IconButtonVariant::default(),
            disabled: false,
            on_click: None,
        }
    }

    /// Set the tooltip text.
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Set the button size.
    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Set the button variant.
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.base.state.disabled = disabled;
        self
    }

    /// Set the click handler.
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

    fn background_color(&self, theme: &crate::theme::ThemeData) -> Color {
        if self.disabled {
            return Color::TRANSPARENT;
        }

        let base = match self.variant {
            IconButtonVariant::Ghost => Color::TRANSPARENT,
            IconButtonVariant::Filled => theme.colors.secondary,
            IconButtonVariant::Outline => Color::TRANSPARENT,
            IconButtonVariant::Destructive => {
                if self.base.state.hovered || self.base.state.pressed {
                    theme.colors.destructive
                } else {
                    Color::TRANSPARENT
                }
            }
        };

        if self.base.state.pressed {
            match self.variant {
                IconButtonVariant::Ghost => theme.colors.accent.with_alpha(0.3),
                IconButtonVariant::Filled => base.darken(15.0),
                IconButtonVariant::Outline => theme.colors.accent.with_alpha(0.2),
                IconButtonVariant::Destructive => base.darken(15.0),
            }
        } else if self.base.state.hovered {
            match self.variant {
                IconButtonVariant::Ghost => theme.colors.accent.with_alpha(0.1),
                IconButtonVariant::Filled => base.darken(10.0),
                IconButtonVariant::Outline => theme.colors.accent.with_alpha(0.1),
                IconButtonVariant::Destructive => base,
            }
        } else {
            base
        }
    }

    fn foreground_color(&self, theme: &crate::theme::ThemeData) -> Color {
        if self.disabled {
            return theme.colors.muted_foreground;
        }

        match self.variant {
            IconButtonVariant::Ghost => theme.colors.foreground,
            IconButtonVariant::Filled => theme.colors.secondary_foreground,
            IconButtonVariant::Outline => theme.colors.foreground,
            IconButtonVariant::Destructive => {
                if self.base.state.hovered || self.base.state.pressed {
                    theme.colors.destructive_foreground
                } else {
                    theme.colors.destructive
                }
            }
        }
    }
}

impl Widget for IconButton {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "icon-button"
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
        let radius = BorderRadius::all(theme.radii.md * theme.typography.base_size);

        // Background
        let bg_color = self.background_color(theme);
        if bg_color != Color::TRANSPARENT {
            painter.fill_rounded_rect(rect, bg_color, radius);
        }

        // Border for outline variant
        if self.variant == IconButtonVariant::Outline {
            let border_color = if self.base.state.focused {
                theme.colors.ring
            } else {
                theme.colors.border
            };
            painter.stroke_rect(rect, border_color, 1.0);
        }

        // Icon
        let icon_size = self.size.icon_size();
        let icon_x = rect.x() + (rect.width() - icon_size * 0.6) / 2.0;
        let icon_y = rect.y() + (rect.height() + icon_size * 0.8) / 2.0;

        painter.draw_text(
            &self.icon,
            Point::new(icon_x, icon_y),
            self.foreground_color(theme),
            icon_size,
        );

        // Focus ring
        if self.base.state.focused && ctx.focus_visible {
            let ring_rect = Rect::new(
                rect.x() - 2.0,
                rect.y() - 2.0,
                rect.width() + 4.0,
                rect.height() + 4.0,
            );
            painter.stroke_rect(ring_rect, theme.colors.ring, 2.0);
        }

        // TODO: Draw tooltip on hover
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if self.disabled {
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
                MouseEventKind::Down if in_bounds => {
                    if mouse.button == Some(MouseButton::Left) {
                        self.base.state.pressed = true;
                        ctx.request_focus(self.base.id);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
                }
                MouseEventKind::Up => {
                    if self.base.state.pressed && mouse.button == Some(MouseButton::Left) {
                        self.base.state.pressed = false;
                        if in_bounds {
                            if let Some(handler) = &self.on_click {
                                handler();
                            }
                        }
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
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
