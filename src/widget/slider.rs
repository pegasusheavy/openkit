//! Slider widget for value selection.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Slider orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A slider widget for selecting a value within a range.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Volume slider
/// let volume = Slider::new()
///     .min(0.0)
///     .max(100.0)
///     .value(50.0)
///     .on_change(|v| println!("Volume: {}", v));
///
/// // Brightness slider with steps
/// let brightness = Slider::new()
///     .min(0.0)
///     .max(100.0)
///     .step(10.0)
///     .value(70.0);
/// ```
pub struct Slider {
    base: WidgetBase,
    min: f32,
    max: f32,
    value: f32,
    step: Option<f32>,
    orientation: SliderOrientation,
    show_value: bool,
    disabled: bool,
    dragging: bool,
    on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
}

impl Slider {
    /// Create a new slider.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("slider"),
            min: 0.0,
            max: 100.0,
            value: 0.0,
            step: None,
            orientation: SliderOrientation::default(),
            show_value: false,
            disabled: false,
            dragging: false,
            on_change: None,
        }
    }

    /// Set the minimum value.
    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    /// Set the maximum value.
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// Set the current value.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    /// Set the step size.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the orientation.
    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set whether to show the value.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
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
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the current value.
    pub fn get_value(&self) -> f32 {
        self.value
    }

    /// Set the value programmatically.
    pub fn set_value(&mut self, value: f32) {
        let new_value = self.snap_to_step(value.clamp(self.min, self.max));
        if (new_value - self.value).abs() > f32::EPSILON {
            self.value = new_value;
            if let Some(handler) = &self.on_change {
                handler(self.value);
            }
        }
    }

    fn snap_to_step(&self, value: f32) -> f32 {
        if let Some(step) = self.step {
            let steps = ((value - self.min) / step).round();
            (self.min + steps * step).clamp(self.min, self.max)
        } else {
            value
        }
    }

    fn value_to_position(&self, rect: Rect) -> f32 {
        let range = self.max - self.min;
        if range == 0.0 {
            return 0.0;
        }
        let ratio = (self.value - self.min) / range;

        match self.orientation {
            SliderOrientation::Horizontal => {
                rect.x() + ratio * (rect.width() - 16.0) + 8.0
            }
            SliderOrientation::Vertical => {
                rect.y() + (1.0 - ratio) * (rect.height() - 16.0) + 8.0
            }
        }
    }

    fn position_to_value(&self, pos: Point, rect: Rect) -> f32 {
        let ratio = match self.orientation {
            SliderOrientation::Horizontal => {
                ((pos.x - rect.x() - 8.0) / (rect.width() - 16.0)).clamp(0.0, 1.0)
            }
            SliderOrientation::Vertical => {
                1.0 - ((pos.y - rect.y() - 8.0) / (rect.height() - 16.0)).clamp(0.0, 1.0)
            }
        };
        self.min + ratio * (self.max - self.min)
    }

    fn track_rect(&self, rect: Rect) -> Rect {
        match self.orientation {
            SliderOrientation::Horizontal => {
                let track_height = 4.0;
                Rect::new(
                    rect.x() + 8.0,
                    rect.y() + (rect.height() - track_height) / 2.0,
                    rect.width() - 16.0,
                    track_height,
                )
            }
            SliderOrientation::Vertical => {
                let track_width = 4.0;
                Rect::new(
                    rect.x() + (rect.width() - track_width) / 2.0,
                    rect.y() + 8.0,
                    track_width,
                    rect.height() - 16.0,
                )
            }
        }
    }

    fn thumb_rect(&self, rect: Rect) -> Rect {
        let thumb_size = 16.0;
        let pos = self.value_to_position(rect);

        match self.orientation {
            SliderOrientation::Horizontal => {
                Rect::new(
                    pos - thumb_size / 2.0,
                    rect.y() + (rect.height() - thumb_size) / 2.0,
                    thumb_size,
                    thumb_size,
                )
            }
            SliderOrientation::Vertical => {
                Rect::new(
                    rect.x() + (rect.width() - thumb_size) / 2.0,
                    pos - thumb_size / 2.0,
                    thumb_size,
                    thumb_size,
                )
            }
        }
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Slider {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "slider"
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
        match self.orientation {
            SliderOrientation::Horizontal => Size::new(200.0, 24.0),
            SliderOrientation::Vertical => Size::new(24.0, 200.0),
        }
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Track background
        let track = self.track_rect(rect);
        let track_radius = BorderRadius::all(2.0);
        painter.fill_rounded_rect(track, theme.colors.muted, track_radius);

        // Filled portion
        let filled_rect = match self.orientation {
            SliderOrientation::Horizontal => {
                let thumb_x = self.value_to_position(rect);
                Rect::new(track.x(), track.y(), thumb_x - track.x(), track.height())
            }
            SliderOrientation::Vertical => {
                let thumb_y = self.value_to_position(rect);
                Rect::new(track.x(), thumb_y, track.width(), track.y() + track.height() - thumb_y)
            }
        };
        let fill_color = if self.disabled {
            theme.colors.muted_foreground
        } else {
            theme.colors.primary
        };
        painter.fill_rounded_rect(filled_rect, fill_color, track_radius);

        // Thumb
        let thumb = self.thumb_rect(rect);
        let thumb_radius = BorderRadius::all(8.0);
        let thumb_color = if self.disabled {
            theme.colors.muted_foreground
        } else if self.dragging {
            theme.colors.primary.darken(10.0)
        } else if self.base.state.hovered {
            theme.colors.primary.lighten(10.0)
        } else {
            theme.colors.primary
        };

        // Thumb shadow
        if !self.disabled {
            let shadow_rect = Rect::new(thumb.x() + 1.0, thumb.y() + 2.0, thumb.width(), thumb.height());
            painter.fill_rounded_rect(shadow_rect, Color::BLACK.with_alpha(0.2), thumb_radius);
        }

        painter.fill_rounded_rect(thumb, thumb_color, thumb_radius);

        // Value display
        if self.show_value {
            let value_text = format!("{:.0}", self.value);
            let text_x = match self.orientation {
                SliderOrientation::Horizontal => rect.x() + rect.width() + 8.0,
                SliderOrientation::Vertical => rect.x() + rect.width() + 4.0,
            };
            let text_y = match self.orientation {
                SliderOrientation::Horizontal => rect.y() + rect.height() / 2.0 + 4.0,
                SliderOrientation::Vertical => rect.y() - 4.0,
            };
            painter.draw_text(
                &value_text,
                Point::new(text_x, text_y),
                theme.colors.foreground,
                12.0,
            );
        }

        // Focus ring
        if self.base.state.focused && ctx.focus_visible {
            let ring_rect = Rect::new(
                thumb.x() - 2.0,
                thumb.y() - 2.0,
                thumb.width() + 4.0,
                thumb.height() + 4.0,
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
            let thumb = self.thumb_rect(self.bounds());
            let in_thumb = thumb.contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Move => {
                    if self.dragging {
                        let new_value = self.position_to_value(mouse.position, self.bounds());
                        self.set_value(new_value);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }

                    if (in_bounds || in_thumb) && !self.base.state.hovered {
                        self.base.state.hovered = true;
                        ctx.request_redraw();
                    } else if !in_bounds && !in_thumb && self.base.state.hovered && !self.dragging {
                        self.base.state.hovered = false;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                    if in_bounds || in_thumb {
                        self.dragging = true;
                        ctx.request_focus(self.base.id);
                        let new_value = self.position_to_value(mouse.position, self.bounds());
                        self.set_value(new_value);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) => {
                    if self.dragging {
                        self.dragging = false;
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
