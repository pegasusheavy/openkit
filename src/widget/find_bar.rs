//! Find in page bar widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, KeyEventKind, Key};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Find bar widget for searching within a page.
#[allow(clippy::type_complexity)]
pub struct FindBar {
    base: WidgetBase,
    query: String,
    is_focused: bool,
    cursor_position: usize,
    match_count: usize,
    current_match: usize,
    case_sensitive: bool,
    on_find: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_next: Option<Box<dyn Fn() + Send + Sync>>,
    on_prev: Option<Box<dyn Fn() + Send + Sync>>,
    on_close: Option<Box<dyn Fn() + Send + Sync>>,
}

impl FindBar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("find-bar"),
            query: String::new(),
            is_focused: true,
            cursor_position: 0,
            match_count: 0,
            current_match: 0,
            case_sensitive: false,
            on_find: None,
            on_next: None,
            on_prev: None,
            on_close: None,
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self.cursor_position = self.query.len();
        self
    }

    pub fn match_count(mut self, count: usize) -> Self { self.match_count = count; self }
    pub fn current_match(mut self, index: usize) -> Self { self.current_match = index; self }
    pub fn case_sensitive(mut self, enabled: bool) -> Self { self.case_sensitive = enabled; self }

    pub fn on_find<F>(mut self, handler: F) -> Self where F: Fn(&str) + Send + Sync + 'static {
        self.on_find = Some(Box::new(handler)); self
    }
    pub fn on_next<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_next = Some(Box::new(handler)); self
    }
    pub fn on_prev<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_prev = Some(Box::new(handler)); self
    }
    pub fn on_close<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_close = Some(Box::new(handler)); self
    }

    fn match_status(&self) -> String {
        if self.query.is_empty() { String::new() }
        else if self.match_count == 0 { "No matches".to_string() }
        else { format!("{} of {}", self.current_match, self.match_count) }
    }
}

impl Default for FindBar {
    fn default() -> Self { Self::new() }
}

impl Widget for FindBar {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "find-bar" }
    fn classes(&self) -> &ClassList { &self.base.classes }
    fn state(&self) -> WidgetState {
        let mut state = self.base.state;
        state.focused = self.is_focused;
        state
    }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(400.0, 40.0)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        LayoutResult::new(Size::new(constraints.max_width.clamp(300.0, 500.0), 40.0))
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = &ctx.style_ctx.theme;

        painter.fill_rect(rect, theme.colors.card);
        painter.fill_rect(Rect::new(rect.x(), rect.y() + rect.height() - 1.0, rect.width(), 1.0), theme.colors.border);

        let y_center = rect.y() + rect.height() / 2.0;

        // Search input
        let input_x = rect.x() + 12.0;
        let input_rect = Rect::new(input_x, y_center - 14.0, 200.0, 28.0);

        let border_color = if self.is_focused && self.match_count == 0 && !self.query.is_empty() {
            theme.colors.destructive
        } else if self.is_focused {
            theme.colors.accent
        } else {
            theme.colors.border
        };

        painter.fill_rounded_rect(input_rect, theme.colors.background, BorderRadius::all(6.0));
        painter.stroke_rounded_rect(input_rect, border_color, BorderRadius::all(6.0), 1.0);

        // Query text
        let text_x = input_x + 8.0;
        if self.query.is_empty() {
            painter.draw_text("Find in page...", Point::new(text_x, y_center - 6.0), theme.colors.muted_foreground, 13.0);
        } else {
            painter.draw_text(&self.query, Point::new(text_x, y_center - 6.0), theme.colors.foreground, 13.0);
        }

        // Cursor
        if self.is_focused {
            let cursor_x = text_x + (self.cursor_position as f32 * 8.0);
            painter.fill_rect(Rect::new(cursor_x, y_center - 10.0, 1.5, 20.0), theme.colors.accent);
        }

        // Match count
        let status_x = input_x + 208.0;
        let status_color = if self.match_count == 0 && !self.query.is_empty() { theme.colors.destructive } else { theme.colors.muted_foreground };
        painter.draw_text(&self.match_status(), Point::new(status_x, y_center - 6.0), status_color, 12.0);

        // Nav buttons
        let btn_size = 28.0;
        let nav_x = status_x + 80.0;

        painter.draw_text("↑", Point::new(nav_x + 8.0, y_center - 8.0), theme.colors.muted_foreground, 16.0);
        painter.draw_text("↓", Point::new(nav_x + 40.0, y_center - 8.0), theme.colors.muted_foreground, 16.0);

        // Case toggle
        let case_bg = if self.case_sensitive { theme.colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT };
        painter.fill_rounded_rect(Rect::new(nav_x + 76.0, y_center - btn_size / 2.0, btn_size, btn_size), case_bg, BorderRadius::all(6.0));
        let case_color = if self.case_sensitive { theme.colors.accent } else { theme.colors.muted_foreground };
        painter.draw_text("Aa", Point::new(nav_x + 80.0, y_center - 6.0), case_color, 12.0);

        // Close button
        let close_x = rect.x() + rect.width() - btn_size - 8.0;
        painter.draw_text("✕", Point::new(close_x + 8.0, y_center - 8.0), theme.colors.muted_foreground, 14.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let bounds = self.base.bounds;
        let y_center = bounds.y() + bounds.height() / 2.0;

        match event {
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down && mouse.button == Some(MouseButton::Left) {
                    // Input focus
                    let input_rect = Rect::new(bounds.x() + 12.0, y_center - 14.0, 200.0, 28.0);
                    if input_rect.contains(mouse.position) {
                        self.is_focused = true;
                        ctx.request_focus(self.base.id);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }

                    // Close button
                    let close_x = bounds.x() + bounds.width() - 36.0;
                    if mouse.position.x >= close_x {
                        if let Some(ref handler) = self.on_close {
                            handler();
                        }
                        return EventResult::Handled;
                    }

                    // Nav buttons
                    let nav_x = bounds.x() + 300.0;
                    if mouse.position.x >= nav_x && mouse.position.x < nav_x + 32.0 && self.match_count > 0 {
                        if let Some(ref handler) = self.on_prev { handler(); }
                        return EventResult::Handled;
                    }
                    if mouse.position.x >= nav_x + 32.0 && mouse.position.x < nav_x + 64.0 && self.match_count > 0 {
                        if let Some(ref handler) = self.on_next { handler(); }
                        return EventResult::Handled;
                    }
                }
            }
            Event::Key(key) if self.is_focused => {
                if key.kind == KeyEventKind::Down {
                    match key.key {
                        Key::Escape => {
                            if let Some(ref handler) = self.on_close { handler(); }
                            return EventResult::Handled;
                        }
                        Key::Enter => {
                            if self.match_count > 0 {
                                if let Some(ref handler) = self.on_next { handler(); }
                            }
                            return EventResult::Handled;
                        }
                        Key::Backspace => {
                            if self.cursor_position > 0 {
                                self.query.remove(self.cursor_position - 1);
                                self.cursor_position -= 1;
                                if let Some(ref handler) = self.on_find { handler(&self.query); }
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
                            if self.cursor_position < self.query.len() { self.cursor_position += 1; }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        _ => {
                            if let Some(ref text) = key.text {
                                for ch in text.chars() {
                                    self.query.insert(self.cursor_position, ch);
                                    self.cursor_position += 1;
                                }
                                if let Some(ref handler) = self.on_find { handler(&self.query); }
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
