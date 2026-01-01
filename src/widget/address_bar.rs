//! Browser address bar widget with URL input and security indicator.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, KeyEventKind, Key};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Security state of the current page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecurityState {
    #[default]
    Unknown,
    Secure,
    SecureEV,
    Insecure,
    Mixed,
    Dangerous,
}

impl SecurityState {
    pub fn icon(&self) -> &'static str {
        match self {
            SecurityState::Unknown => "🌐",
            SecurityState::Secure | SecurityState::SecureEV => "🔒",
            SecurityState::Insecure | SecurityState::Mixed => "⚠️",
            SecurityState::Dangerous => "🚫",
        }
    }

    pub fn color(&self, theme: &crate::theme::ThemeData) -> Color {
        match self {
            SecurityState::Unknown => theme.colors.muted_foreground,
            SecurityState::Secure | SecurityState::SecureEV => theme.colors.success,
            SecurityState::Insecure | SecurityState::Mixed => theme.colors.warning,
            SecurityState::Dangerous => theme.colors.destructive,
        }
    }
}

/// Browser address bar widget.
#[allow(clippy::type_complexity)]
pub struct AddressBar {
    base: WidgetBase,
    url: String,
    display_text: String,
    is_focused: bool,
    cursor_position: usize,
    security_state: SecurityState,
    placeholder: String,
    on_navigate: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_input: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl AddressBar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("address-bar"),
            url: String::new(),
            display_text: String::new(),
            is_focused: false,
            cursor_position: 0,
            security_state: SecurityState::Unknown,
            placeholder: "Search or enter address".to_string(),
            on_navigate: None,
            on_input: None,
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.display_text = self.format_display_url(&url);
        self.url = url;
        self
    }

    pub fn security_state(mut self, state: SecurityState) -> Self {
        self.security_state = state;
        self
    }

    pub fn on_navigate<F>(mut self, handler: F) -> Self
    where F: Fn(&str) + Send + Sync + 'static {
        self.on_navigate = Some(Box::new(handler));
        self
    }

    pub fn on_input<F>(mut self, handler: F) -> Self
    where F: Fn(&str) + Send + Sync + 'static {
        self.on_input = Some(Box::new(handler));
        self
    }

    fn format_display_url(&self, url: &str) -> String {
        let mut display = url.to_string();
        if let Some(stripped) = display.strip_prefix("https://") {
            display = stripped.to_string();
        } else if let Some(stripped) = display.strip_prefix("http://") {
            display = stripped.to_string();
        }
        if let Some(stripped) = display.strip_prefix("www.") {
            display = stripped.to_string();
        }
        if display.ends_with('/') && display.len() > 1 {
            display.pop();
        }
        display
    }

    fn looks_like_url(&self, input: &str) -> bool {
        input.contains('.') && !input.contains(' ')
            || input.starts_with("http://")
            || input.starts_with("https://")
            || input.starts_with("about:")
    }
}

impl Default for AddressBar {
    fn default() -> Self { Self::new() }
}

impl Widget for AddressBar {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "address-bar" }
    fn classes(&self) -> &ClassList { &self.base.classes }

    fn state(&self) -> WidgetState {
        let mut state = self.base.state;
        state.focused = self.is_focused;
        state
    }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(400.0, 36.0)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        LayoutResult::new(Size::new(constraints.max_width.clamp(200.0, 800.0), 36.0))
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = &ctx.style_ctx.theme;

        let bg_color = if self.is_focused { theme.colors.background } else { theme.colors.card };
        let border_color = if self.is_focused { theme.colors.accent } else { theme.colors.border };

        painter.fill_rounded_rect(rect, bg_color, BorderRadius::all(8.0));
        painter.stroke_rounded_rect(rect, border_color, BorderRadius::all(8.0), 1.0);

        // Security indicator
        let security_x = rect.x() + 12.0;
        painter.draw_text(
            self.security_state.icon(),
            Point::new(security_x, rect.y() + rect.height() / 2.0 - 8.0),
            self.security_state.color(theme),
            16.0,
        );

        // URL text
        let text_x = security_x + 28.0;
        let display = if self.display_text.is_empty() && !self.is_focused {
            &self.placeholder
        } else {
            &self.display_text
        };

        let text_color = if self.display_text.is_empty() {
            theme.colors.muted_foreground
        } else {
            theme.colors.foreground
        };

        painter.draw_text(display, Point::new(text_x, rect.y() + rect.height() / 2.0 - 7.0), text_color, 14.0);

        // Cursor
        if self.is_focused {
            let cursor_x = text_x + (self.cursor_position as f32 * 8.0);
            painter.fill_rect(Rect::new(cursor_x, rect.y() + 8.0, 1.5, rect.height() - 16.0), theme.colors.accent);
        }

        // Reload button
        let btn_x = rect.x() + rect.width() - 32.0;
        painter.draw_text("↻", Point::new(btn_x, rect.y() + rect.height() / 2.0 - 8.0), theme.colors.muted_foreground, 16.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let bounds = self.base.bounds;

        match event {
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down && mouse.button == Some(MouseButton::Left) {
                    if bounds.contains(mouse.position) {
                        self.is_focused = true;
                        self.cursor_position = self.display_text.len();
                        ctx.request_focus(self.base.id);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    } else if self.is_focused {
                        self.is_focused = false;
                        ctx.release_focus();
                        ctx.request_redraw();
                    }
                }
            }
            Event::Key(key) if self.is_focused => {
                if key.kind == KeyEventKind::Down {
                    match key.key {
                        Key::Enter => {
                            let input = self.display_text.trim();
                            if !input.is_empty() {
                                if let Some(ref handler) = self.on_navigate {
                                    let url = if self.looks_like_url(input) {
                                        if input.starts_with("http") || input.starts_with("about:") {
                                            input.to_string()
                                        } else {
                                            format!("https://{}", input)
                                        }
                                    } else {
                                        format!("https://duckduckgo.com/?q={}", input.replace(' ', "+"))
                                    };
                                    handler(&url);
                                }
                            }
                            return EventResult::Handled;
                        }
                        Key::Escape => {
                            self.display_text = self.format_display_url(&self.url);
                            self.is_focused = false;
                            ctx.release_focus();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Backspace => {
                            if self.cursor_position > 0 {
                                self.display_text.remove(self.cursor_position - 1);
                                self.cursor_position -= 1;
                                if let Some(ref handler) = self.on_input {
                                    handler(&self.display_text);
                                }
                                ctx.request_redraw();
                            }
                            return EventResult::Handled;
                        }
                        Key::Left => {
                            if self.cursor_position > 0 { self.cursor_position -= 1; }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Right => {
                            if self.cursor_position < self.display_text.len() { self.cursor_position += 1; }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Home => {
                            self.cursor_position = 0;
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::End => {
                            self.cursor_position = self.display_text.len();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        _ => {
                            // Handle text input
                            if let Some(ref text) = key.text {
                                for ch in text.chars() {
                                    self.display_text.insert(self.cursor_position, ch);
                                    self.cursor_position += 1;
                                }
                                if let Some(ref handler) = self.on_input {
                                    handler(&self.display_text);
                                }
                                ctx.request_redraw();
                                return EventResult::Handled;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        EventResult::Ignored
    }

    fn bounds(&self) -> Rect { self.base.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.base.bounds = bounds; }
}
