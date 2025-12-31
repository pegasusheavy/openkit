//! Text field widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, KeyEventKind, Key};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A single-line text input widget.
#[allow(clippy::type_complexity)]
pub struct TextField {
    base: WidgetBase,
    value: String,
    placeholder: String,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
    cursor_position: usize,
}

impl TextField {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("textfield"),
            value: String::new(),
            placeholder: String::new(),
            on_change: None,
            on_submit: None,
            cursor_position: 0,
        }
    }

    /// Set the initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the change handler.
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Set the submit handler (called on Enter).
    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_submit = Some(Box::new(handler));
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

    /// Get the current value.
    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Set the value.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor_position = self.value.len();
        if let Some(handler) = &self.on_change {
            handler(&self.value);
        }
    }

    /// Insert text at the cursor position.
    fn insert_text(&mut self, text: &str) {
        self.value.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
        if let Some(handler) = &self.on_change {
            handler(&self.value);
        }
    }

    /// Delete character before cursor.
    fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
            if let Some(handler) = &self.on_change {
                handler(&self.value);
            }
        }
    }

    /// Delete character after cursor.
    fn delete(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
            if let Some(handler) = &self.on_change {
                handler(&self.value);
            }
        }
    }
}

impl Default for TextField {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextField {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "textfield"
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
        // Default text field size
        Size::new(200.0, 36.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let intrinsic = self.intrinsic_size(ctx);
        let size = constraints.constrain(intrinsic);
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let radius = BorderRadius::all(6.0);
        let padding = 12.0;

        // Background
        let bg_color = if self.base.state.focused {
            theme.colors.background
        } else {
            theme.colors.input
        };
        painter.fill_rounded_rect(rect, bg_color, radius);

        // Border
        let border_color = if self.base.state.focused {
            theme.colors.ring
        } else if self.base.state.hovered {
            theme.colors.border.darken(10.0)
        } else {
            theme.colors.border
        };
        painter.stroke_rect(rect, border_color, 1.0);

        // Text or placeholder
        let font_size = 14.0;
        let text_y = rect.y() + (rect.height() + font_size * 0.8) / 2.0;

        if self.value.is_empty() {
            // Show placeholder
            painter.draw_text(
                &self.placeholder,
                Point::new(rect.x() + padding, text_y),
                theme.colors.muted_foreground,
                font_size,
            );
        } else {
            // Show value
            painter.draw_text(
                &self.value,
                Point::new(rect.x() + padding, text_y),
                theme.colors.foreground,
                font_size,
            );
        }

        // Cursor (when focused)
        if self.base.state.focused {
            let cursor_x = rect.x() + padding + (self.cursor_position as f32 * font_size * 0.6);
            let cursor_y = rect.y() + (rect.height() - font_size) / 2.0;
            painter.draw_line(
                Point::new(cursor_x, cursor_y),
                Point::new(cursor_x, cursor_y + font_size),
                theme.colors.foreground,
                1.0,
            );
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
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
                            self.base.state.focused = true;
                            ctx.request_focus(self.base.id);

                            // Calculate cursor position from click
                            let padding = 12.0;
                            let font_size = 14.0;
                            let relative_x = mouse.position.x - self.bounds().x() - padding;
                            let char_index = (relative_x / (font_size * 0.6)).round() as usize;
                            self.cursor_position = char_index.min(self.value.len());

                            ctx.request_redraw();
                            return EventResult::Handled;
                        } else if !in_bounds && self.base.state.focused {
                            self.base.state.focused = false;
                            ctx.release_focus();
                            ctx.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            Event::Key(key) if self.base.state.focused => {
                if key.kind == KeyEventKind::Down {
                    match &key.key {
                        Key::Backspace => {
                            self.backspace();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Delete => {
                            self.delete();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Left => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                                ctx.request_redraw();
                            }
                            return EventResult::Handled;
                        }
                        Key::Right => {
                            if self.cursor_position < self.value.len() {
                                self.cursor_position += 1;
                                ctx.request_redraw();
                            }
                            return EventResult::Handled;
                        }
                        Key::Home => {
                            self.cursor_position = 0;
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::End => {
                            self.cursor_position = self.value.len();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Enter => {
                            if let Some(handler) = &self.on_submit {
                                handler(&self.value);
                            }
                            return EventResult::Handled;
                        }
                        _ => {
                            // Handle text input
                            if let Some(text) = &key.text {
                                if !text.is_empty() && !key.modifiers.control && !key.modifiers.alt {
                                    self.insert_text(text);
                                    ctx.request_redraw();
                                    return EventResult::Handled;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
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
