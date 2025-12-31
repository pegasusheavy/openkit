//! Password field widget with masked input.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, KeyEventKind, Key};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A password input field with masked characters.
///
/// The password field displays dots or asterisks instead of the actual
/// characters for security.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let password = PasswordField::new()
///     .placeholder("Enter password...")
///     .on_submit(|password| {
///         println!("Password submitted");
///     });
/// ```
#[allow(clippy::type_complexity)]
pub struct PasswordField {
    base: WidgetBase,
    value: String,
    placeholder: String,
    mask_char: char,
    show_toggle: bool,
    is_revealed: bool,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
    cursor_position: usize,
    cursor_visible: bool,
}

impl PasswordField {
    /// Create a new password field.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("password-field"),
            value: String::new(),
            placeholder: String::new(),
            mask_char: '•',
            show_toggle: true,
            is_revealed: false,
            on_change: None,
            on_submit: None,
            cursor_position: 0,
            cursor_visible: true,
        }
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the mask character (default: '•').
    pub fn mask_char(mut self, c: char) -> Self {
        self.mask_char = c;
        self
    }

    /// Enable or disable the show/hide toggle button.
    pub fn show_toggle(mut self, show: bool) -> Self {
        self.show_toggle = show;
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
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set the value programmatically.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor_position = self.value.len();
    }

    /// Clear the password field.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    /// Toggle password visibility.
    pub fn toggle_visibility(&mut self) {
        self.is_revealed = !self.is_revealed;
    }

    /// Get the display text (masked or revealed).
    fn display_text(&self) -> String {
        if self.is_revealed {
            self.value.clone()
        } else {
            self.mask_char.to_string().repeat(self.value.len())
        }
    }

    fn handle_key_input(&mut self, key: &Key, text: Option<&str>) -> bool {
        match key {
            Key::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.value.remove(self.cursor_position);
                    if let Some(handler) = &self.on_change {
                        handler(&self.value);
                    }
                    return true;
                }
            }
            Key::Delete => {
                if self.cursor_position < self.value.len() {
                    self.value.remove(self.cursor_position);
                    if let Some(handler) = &self.on_change {
                        handler(&self.value);
                    }
                    return true;
                }
            }
            Key::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    return true;
                }
            }
            Key::Right => {
                if self.cursor_position < self.value.len() {
                    self.cursor_position += 1;
                    return true;
                }
            }
            Key::Home => {
                self.cursor_position = 0;
                return true;
            }
            Key::End => {
                self.cursor_position = self.value.len();
                return true;
            }
            Key::Enter => {
                if let Some(handler) = &self.on_submit {
                    handler(&self.value);
                }
                return true;
            }
            _ => {}
        }

        // Handle text input
        if let Some(text) = text {
            for c in text.chars() {
                if !c.is_control() {
                    self.value.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
            }
            if let Some(handler) = &self.on_change {
                handler(&self.value);
            }
            return true;
        }

        false
    }
}

impl Default for PasswordField {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for PasswordField {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "password-field"
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
        Size::new(200.0, 40.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let intrinsic = self.intrinsic_size(ctx);
        let size = Size::new(
            constraints.max_width.min(intrinsic.width.max(constraints.min_width)),
            constraints.constrain(intrinsic).height,
        );
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        let bg_color = if self.base.state.disabled {
            theme.colors.muted
        } else {
            theme.colors.background
        };
        let radius = BorderRadius::all(theme.radii.md * theme.typography.base_size);
        painter.fill_rounded_rect(rect, bg_color, radius);

        // Border
        let border_color = if self.base.state.focused {
            theme.colors.ring
        } else if self.base.state.hovered {
            theme.colors.ring.with_alpha(0.5)
        } else {
            theme.colors.border
        };
        painter.stroke_rect(rect, border_color, 1.0);

        // Text or placeholder
        let font_size = 14.0;
        let padding = 12.0;
        let text_x = rect.x() + padding;
        let text_y = rect.y() + (rect.height() + font_size * 0.8) / 2.0;

        let toggle_width = if self.show_toggle { 32.0 } else { 0.0 };
        let _text_area_width = rect.width() - padding * 2.0 - toggle_width;

        if self.value.is_empty() {
            // Draw placeholder
            painter.draw_text(
                &self.placeholder,
                Point::new(text_x, text_y),
                theme.colors.muted_foreground,
                font_size,
            );
        } else {
            // Draw masked/revealed text
            let display = self.display_text();
            painter.draw_text(
                &display,
                Point::new(text_x, text_y),
                theme.colors.foreground,
                font_size,
            );

            // Draw cursor if focused
            if self.base.state.focused && self.cursor_visible {
                let cursor_text = if self.is_revealed {
                    &self.value[..self.cursor_position]
                } else {
                    &self.mask_char.to_string().repeat(self.cursor_position)
                };
                let cursor_x = text_x + cursor_text.len() as f32 * font_size * 0.6;
                painter.fill_rect(
                    Rect::new(cursor_x, rect.y() + 8.0, 2.0, rect.height() - 16.0),
                    theme.colors.foreground,
                );
            }
        }

        // Draw toggle button
        if self.show_toggle {
            let toggle_x = rect.x() + rect.width() - toggle_width - 4.0;
            let toggle_y = rect.y() + (rect.height() - 16.0) / 2.0;

            // Draw eye icon (simplified)
            let icon_color = if self.is_revealed {
                theme.colors.primary
            } else {
                theme.colors.muted_foreground
            };

            // Simple eye shape
            painter.fill_rect(
                Rect::new(toggle_x + 4.0, toggle_y + 6.0, 16.0, 4.0),
                icon_color,
            );
        }

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
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
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
                        self.base.state.focused = true;
                        ctx.request_focus(self.base.id);
                        ctx.request_redraw();

                        // Check if toggle button was clicked
                        if self.show_toggle {
                            let toggle_x = self.base.bounds.x() + self.base.bounds.width() - 36.0;
                            if mouse.position.x >= toggle_x {
                                self.toggle_visibility();
                            }
                        }

                        return EventResult::Handled;
                    }
                    _ => {}
                }
            }
            Event::Key(key) if self.base.state.focused => {
                if key.kind == KeyEventKind::Down
                    && self.handle_key_input(&key.key, key.text.as_deref()) {
                        ctx.request_redraw();
                        return EventResult::Handled;
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
