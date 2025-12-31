//! Checkbox widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A checkbox widget.
pub struct Checkbox {
    base: WidgetBase,
    label: Option<String>,
    checked: bool,
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("checkbox"),
            label: None,
            checked: false,
            on_change: None,
        }
    }

    /// Set the checkbox label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the initial checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self.base.state.checked = checked;
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

    /// Set the element ID.
    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }

    /// Get the current checked state.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Set the checked state.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
        self.base.state.checked = checked;
    }

    /// Toggle the checked state.
    pub fn toggle(&mut self) {
        self.set_checked(!self.checked);
        if let Some(handler) = &self.on_change {
            handler(self.checked);
        }
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Checkbox {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "checkbox"
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
        let box_size = 20.0;
        let gap = 8.0;
        let label_width = self.label.as_ref().map(|l| l.len() as f32 * 8.0).unwrap_or(0.0);

        if self.label.is_some() {
            Size::new(box_size + gap + label_width, box_size.max(20.0))
        } else {
            Size::new(box_size, box_size)
        }
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let intrinsic = self.intrinsic_size(ctx);
        let size = constraints.constrain(intrinsic);
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let box_size = 20.0;

        // Checkbox box
        let box_rect = Rect::new(rect.x(), rect.y(), box_size, box_size);
        let radius = BorderRadius::all(4.0);

        // Background
        let bg_color = if self.checked {
            theme.colors.primary
        } else if self.base.state.hovered {
            theme.colors.accent
        } else {
            theme.colors.background
        };
        painter.fill_rounded_rect(box_rect, bg_color, radius);

        // Border
        let border_color = if self.checked {
            theme.colors.primary
        } else {
            theme.colors.border
        };
        painter.stroke_rect(box_rect, border_color, 1.0);

        // Checkmark
        if self.checked {
            let check_color = theme.colors.primary_foreground;
            let cx = box_rect.center().x;
            let cy = box_rect.center().y;

            // Simple checkmark (two lines)
            painter.draw_line(
                Point::new(cx - 5.0, cy),
                Point::new(cx - 1.0, cy + 4.0),
                check_color,
                2.0,
            );
            painter.draw_line(
                Point::new(cx - 1.0, cy + 4.0),
                Point::new(cx + 5.0, cy - 4.0),
                check_color,
                2.0,
            );
        }

        // Label
        if let Some(label) = &self.label {
            let text_x = rect.x() + box_size + 8.0;
            let text_y = rect.y() + (rect.height() + 14.0 * 0.8) / 2.0;
            painter.draw_text(label, Point::new(text_x, text_y), theme.colors.foreground, 14.0);
        }

        // Focus ring
        if self.base.state.focused && ctx.focus_visible {
            let ring_rect = box_rect.offset(-2.0, -2.0);
            let ring_rect = Rect::new(
                ring_rect.x(),
                ring_rect.y(),
                box_size + 4.0,
                box_size + 4.0,
            );
            painter.stroke_rect(ring_rect, theme.colors.ring, 2.0);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            let in_bounds = self.bounds().contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Enter | MouseEventKind::Move => {
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
                MouseEventKind::Down => {
                    if in_bounds && mouse.button == Some(MouseButton::Left) {
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
                            self.toggle();
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
