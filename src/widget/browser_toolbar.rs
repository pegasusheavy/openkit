//! Complete browser toolbar combining navigation, address bar, and actions.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use super::address_bar::SecurityState;
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Browser toolbar action button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    Downloads,
    Extensions,
    Bookmarks,
    Settings,
    Menu,
}

impl ToolbarAction {
    pub fn icon(&self) -> &'static str {
        match self {
            ToolbarAction::Downloads => "⬇️",
            ToolbarAction::Extensions => "🧩",
            ToolbarAction::Bookmarks => "⭐",
            ToolbarAction::Settings => "⚙️",
            ToolbarAction::Menu => "☰",
        }
    }
}

/// Browser toolbar widget.
#[allow(clippy::type_complexity)]
pub struct BrowserToolbar {
    base: WidgetBase,
    can_go_back: bool,
    can_go_forward: bool,
    is_loading: bool,
    url: String,
    security_state: SecurityState,
    has_active_downloads: bool,
    download_count: u32,
    hovered_action: Option<ToolbarAction>,
    on_back: Option<Box<dyn Fn() + Send + Sync>>,
    on_forward: Option<Box<dyn Fn() + Send + Sync>>,
    on_reload: Option<Box<dyn Fn() + Send + Sync>>,
    on_stop: Option<Box<dyn Fn() + Send + Sync>>,
    on_navigate: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_action: Option<Box<dyn Fn(ToolbarAction) + Send + Sync>>,
}

impl BrowserToolbar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("browser-toolbar"),
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
            url: String::new(),
            security_state: SecurityState::Unknown,
            has_active_downloads: false,
            download_count: 0,
            hovered_action: None,
            on_back: None,
            on_forward: None,
            on_reload: None,
            on_stop: None,
            on_navigate: None,
            on_action: None,
        }
    }

    pub fn can_go_back(mut self, can: bool) -> Self { self.can_go_back = can; self }
    pub fn can_go_forward(mut self, can: bool) -> Self { self.can_go_forward = can; self }
    pub fn is_loading(mut self, loading: bool) -> Self { self.is_loading = loading; self }
    pub fn url(mut self, url: impl Into<String>) -> Self { self.url = url.into(); self }
    pub fn security_state(mut self, state: SecurityState) -> Self { self.security_state = state; self }
    pub fn has_active_downloads(mut self, has: bool) -> Self { self.has_active_downloads = has; self }
    pub fn download_count(mut self, count: u32) -> Self { self.download_count = count; self }

    pub fn on_back<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_back = Some(Box::new(handler)); self
    }
    pub fn on_forward<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_forward = Some(Box::new(handler)); self
    }
    pub fn on_reload<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_reload = Some(Box::new(handler)); self
    }
    pub fn on_stop<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_stop = Some(Box::new(handler)); self
    }
    pub fn on_navigate<F>(mut self, handler: F) -> Self where F: Fn(&str) + Send + Sync + 'static {
        self.on_navigate = Some(Box::new(handler)); self
    }
    pub fn on_action<F>(mut self, handler: F) -> Self where F: Fn(ToolbarAction) + Send + Sync + 'static {
        self.on_action = Some(Box::new(handler)); self
    }

    fn format_url(&self) -> String {
        let mut display = self.url.clone();
        if let Some(s) = display.strip_prefix("https://") { display = s.to_string(); }
        else if let Some(s) = display.strip_prefix("http://") { display = s.to_string(); }
        if let Some(s) = display.strip_prefix("www.") { display = s.to_string(); }
        if display.ends_with('/') && display.len() > 1 { display.pop(); }
        display
    }
}

impl Default for BrowserToolbar {
    fn default() -> Self { Self::new() }
}

impl Widget for BrowserToolbar {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "browser-toolbar" }
    fn classes(&self) -> &ClassList { &self.base.classes }
    fn state(&self) -> WidgetState { self.base.state }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(800.0, 44.0)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        LayoutResult::new(Size::new(constraints.max_width.min(1920.0), 44.0))
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = &ctx.style_ctx.theme;

        painter.fill_rect(rect, theme.colors.card);

        let y_center = rect.y() + rect.height() / 2.0;
        let button_size = 32.0;
        let button_y = y_center - button_size / 2.0;

        // Navigation buttons
        let nav_x = rect.x() + 8.0;

        // Back
        let back_color = if self.can_go_back { theme.colors.muted_foreground } else { theme.colors.muted_foreground.with_alpha(0.3) };
        painter.draw_text("←", Point::new(nav_x + 8.0, y_center - 9.0), back_color, 18.0);

        // Forward
        let fwd_x = nav_x + 36.0;
        let fwd_color = if self.can_go_forward { theme.colors.muted_foreground } else { theme.colors.muted_foreground.with_alpha(0.3) };
        painter.draw_text("→", Point::new(fwd_x + 8.0, y_center - 9.0), fwd_color, 18.0);

        // Reload/Stop
        let reload_x = fwd_x + 36.0;
        let reload_icon = if self.is_loading { "✕" } else { "↻" };
        painter.draw_text(reload_icon, Point::new(reload_x + 8.0, y_center - 9.0), theme.colors.muted_foreground, 18.0);

        // Address bar
        let addr_x = reload_x + 44.0;
        let actions_width = 5.0 * 36.0 + 16.0;
        let addr_width = rect.width() - addr_x + rect.x() - actions_width - 8.0;
        let addr_height = 32.0;
        let addr_y = y_center - addr_height / 2.0;
        let addr_rect = Rect::new(addr_x, addr_y, addr_width, addr_height);

        painter.fill_rounded_rect(addr_rect, theme.colors.background, BorderRadius::all(8.0));
        painter.stroke_rounded_rect(addr_rect, theme.colors.border, BorderRadius::all(8.0), 1.0);

        // Security icon
        painter.draw_text(
            self.security_state.icon(),
            Point::new(addr_x + 10.0, y_center - 8.0),
            self.security_state.color(theme),
            14.0,
        );

        // URL
        let url_display = self.format_url();
        painter.draw_text(&url_display, Point::new(addr_x + 32.0, y_center - 7.0), theme.colors.foreground, 13.0);

        // Loading indicator
        if self.is_loading {
            let progress_y = addr_y + addr_height - 2.0;
            painter.fill_rect(Rect::new(addr_x + 1.0, progress_y, addr_width * 0.3, 2.0), theme.colors.accent);
        }

        // Action buttons
        let actions = [ToolbarAction::Downloads, ToolbarAction::Extensions, ToolbarAction::Bookmarks, ToolbarAction::Settings, ToolbarAction::Menu];
        let mut action_x = rect.x() + rect.width() - actions_width;

        for action in actions {
            let is_hovered = self.hovered_action == Some(action);
            let btn_rect = Rect::new(action_x + 2.0, button_y, button_size, button_size);

            if is_hovered {
                painter.fill_rounded_rect(btn_rect, theme.colors.surface_hover, BorderRadius::all(6.0));
            }

            painter.draw_text(action.icon(), Point::new(action_x + 8.0, y_center - 8.0), theme.colors.muted_foreground, 16.0);

            // Badge for downloads
            if action == ToolbarAction::Downloads && self.has_active_downloads {
                let badge_x = action_x + button_size - 10.0;
                painter.fill_rounded_rect(
                    Rect::new(badge_x, button_y + 2.0, 12.0, 12.0),
                    theme.colors.accent,
                    BorderRadius::all(6.0),
                );
                if self.download_count > 0 && self.download_count < 10 {
                    painter.draw_text(&self.download_count.to_string(), Point::new(badge_x + 3.0, button_y + 3.0), Color::WHITE, 9.0);
                }
            }

            action_x += 36.0;
        }

        // Bottom border
        painter.fill_rect(Rect::new(rect.x(), rect.y() + rect.height() - 1.0, rect.width(), 1.0), theme.colors.border);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let bounds = self.base.bounds;

        if let Event::Mouse(mouse) = event {
            let pos = mouse.position;
            let y_center = bounds.y() + bounds.height() / 2.0;
            let button_size = 32.0;
            let button_y = y_center - button_size / 2.0;

            match mouse.kind {
                MouseEventKind::Move => {
                    // Check action button hovers
                    let actions_width = 5.0 * 36.0 + 16.0;
                    let action_start_x = bounds.x() + bounds.width() - actions_width;

                    if pos.x >= action_start_x && bounds.contains(pos) {
                        let idx = ((pos.x - action_start_x) / 36.0) as usize;
                        let actions = [ToolbarAction::Downloads, ToolbarAction::Extensions, ToolbarAction::Bookmarks, ToolbarAction::Settings, ToolbarAction::Menu];
                        self.hovered_action = actions.get(idx).copied();
                        ctx.request_redraw();
                        return EventResult::Handled;
                    } else if self.hovered_action.is_some() {
                        self.hovered_action = None;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down => {
                    if mouse.button == Some(MouseButton::Left) {
                        // Check action buttons
                        if let Some(action) = self.hovered_action {
                            if let Some(ref handler) = self.on_action {
                                handler(action);
                            }
                            return EventResult::Handled;
                        }

                        // Check nav buttons
                        let nav_x = bounds.x() + 8.0;
                        if Rect::new(nav_x, button_y, 32.0, 32.0).contains(pos) && self.can_go_back {
                            if let Some(ref h) = self.on_back { h(); }
                            return EventResult::Handled;
                        }
                        if Rect::new(nav_x + 36.0, button_y, 32.0, 32.0).contains(pos) && self.can_go_forward {
                            if let Some(ref h) = self.on_forward { h(); }
                            return EventResult::Handled;
                        }
                        if Rect::new(nav_x + 72.0, button_y, 32.0, 32.0).contains(pos) {
                            if self.is_loading {
                                if let Some(ref h) = self.on_stop { h(); }
                            } else if let Some(ref h) = self.on_reload { h(); }
                            return EventResult::Handled;
                        }
                    }
                }
                _ => {}
            }
        }

        EventResult::Ignored
    }

    fn bounds(&self) -> Rect { self.base.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.base.bounds = bounds; }
}
