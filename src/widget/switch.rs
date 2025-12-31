//! Toggle switch widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Toggle switch size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleSwitchSize {
    /// Small switch
    Small,
    /// Medium switch (default)
    #[default]
    Medium,
    /// Large switch
    Large,
}

impl ToggleSwitchSize {
    fn dimensions(&self) -> (f32, f32) {
        match self {
            ToggleSwitchSize::Small => (32.0, 18.0),
            ToggleSwitchSize::Medium => (44.0, 24.0),
            ToggleSwitchSize::Large => (56.0, 30.0),
        }
    }

    fn thumb_size(&self) -> f32 {
        match self {
            ToggleSwitchSize::Small => 14.0,
            ToggleSwitchSize::Medium => 20.0,
            ToggleSwitchSize::Large => 26.0,
        }
    }
}

/// A toggle switch widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let wifi_switch = ToggleSwitch::new()
///     .checked(true)
///     .label("Wi-Fi")
///     .on_change(|enabled| println!("Wi-Fi: {}", enabled));
///
/// let dark_mode = ToggleSwitch::new()
///     .checked(false)
///     .size(ToggleSwitchSize::Small);
/// ```
pub struct ToggleSwitch {
    base: WidgetBase,
    checked: bool,
    label: Option<String>,
    size: ToggleSwitchSize,
    disabled: bool,
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl ToggleSwitch {
    /// Create a new switch.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("toggle-switch"),
            checked: false,
            label: None,
            size: ToggleSwitchSize::default(),
            disabled: false,
            on_change: None,
        }
    }

    /// Set the checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the label text.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the size.
    pub fn size(mut self, size: ToggleSwitchSize) -> Self {
        self.size = size;
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.base.state.disabled = disabled;
        self
    }

    /// Set the change handler.
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the checked state.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Toggle the switch.
    pub fn toggle(&mut self) {
        if !self.disabled {
            self.checked = !self.checked;
            if let Some(handler) = &self.on_change {
                handler(self.checked);
            }
        }
    }

    /// Set the checked state programmatically.
    pub fn set_checked(&mut self, checked: bool) {
        if self.checked != checked {
            self.checked = checked;
            if let Some(handler) = &self.on_change {
                handler(self.checked);
            }
        }
    }

    fn track_rect(&self, rect: Rect) -> Rect {
        let (width, height) = self.size.dimensions();
        Rect::new(rect.x(), rect.y() + (rect.height() - height) / 2.0, width, height)
    }

    fn thumb_rect(&self, track: Rect) -> Rect {
        let thumb_size = self.size.thumb_size();
        let padding = (track.height() - thumb_size) / 2.0;

        let x = if self.checked {
            track.x() + track.width() - thumb_size - padding
        } else {
            track.x() + padding
        };

        Rect::new(x, track.y() + padding, thumb_size, thumb_size)
    }
}

impl Default for ToggleSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ToggleSwitch {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "toggle-switch"
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
        let (width, height) = self.size.dimensions();
        let label_width = self.label.as_ref().map(|l| l.len() as f32 * 8.0 + 12.0).unwrap_or(0.0);
        Size::new(width + label_width, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let track = self.track_rect(rect);
        let track_radius = BorderRadius::all(track.height() / 2.0);

        // Track
        let track_color = if self.disabled {
            theme.colors.muted
        } else if self.checked {
            theme.colors.primary
        } else {
            theme.colors.muted
        };
        painter.fill_rounded_rect(track, track_color, track_radius);

        // Track border
        if !self.checked {
            painter.stroke_rect(track, theme.colors.border, 1.0);
        }

        // Thumb
        let thumb = self.thumb_rect(track);
        let thumb_radius = BorderRadius::all(thumb.height() / 2.0);

        // Thumb shadow
        if !self.disabled {
            let shadow_rect = Rect::new(thumb.x() + 1.0, thumb.y() + 2.0, thumb.width(), thumb.height());
            painter.fill_rounded_rect(shadow_rect, Color::BLACK.with_alpha(0.15), thumb_radius);
        }

        let thumb_color = if self.disabled {
            theme.colors.muted_foreground
        } else {
            Color::WHITE
        };
        painter.fill_rounded_rect(thumb, thumb_color, thumb_radius);

        // Label
        if let Some(ref label) = self.label {
            let (switch_width, _switch_height) = self.size.dimensions();
            let label_x = rect.x() + switch_width + 12.0;
            let label_y = rect.y() + (rect.height() + 12.0) / 2.0;

            let label_color = if self.disabled {
                theme.colors.muted_foreground
            } else {
                theme.colors.foreground
            };

            painter.draw_text(label, Point::new(label_x, label_y), label_color, 14.0);
        }

        // Focus ring
        if self.base.state.focused && ctx.focus_visible {
            let ring_rect = Rect::new(
                track.x() - 2.0,
                track.y() - 2.0,
                track.width() + 4.0,
                track.height() + 4.0,
            );
            painter.stroke_rect(ring_rect, theme.colors.ring, 2.0);
        }
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
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) && in_bounds => {
                    self.toggle();
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
