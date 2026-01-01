//! Browser tab widget for tabbed browsing interfaces.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Browser tab loading state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabLoadingState {
    #[default]
    Idle,
    Loading,
    Complete,
    Error,
}

/// A browser tab.
#[derive(Debug, Clone)]
pub struct BrowserTabData {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub loading_state: TabLoadingState,
    pub loading_progress: f32,
    pub is_pinned: bool,
    pub is_muted: bool,
    pub is_playing_audio: bool,
}

impl BrowserTabData {
    pub fn new(id: impl Into<String>, title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            url: url.into(),
            favicon: None,
            loading_state: TabLoadingState::Idle,
            loading_progress: 0.0,
            is_pinned: false,
            is_muted: false,
            is_playing_audio: false,
        }
    }

    pub fn favicon(mut self, favicon: impl Into<String>) -> Self {
        self.favicon = Some(favicon.into());
        self
    }

    pub fn display_title(&self, max_chars: usize) -> String {
        if self.title.chars().count() > max_chars {
            let mut title: String = self.title.chars().take(max_chars - 1).collect();
            title.push('…');
            title
        } else {
            self.title.clone()
        }
    }

    pub fn status_icon(&self) -> Option<&'static str> {
        if self.loading_state == TabLoadingState::Loading {
            Some("⟳")
        } else if self.is_playing_audio {
            if self.is_muted { Some("🔇") } else { Some("🔊") }
        } else {
            None
        }
    }
}

/// Browser tab bar widget.
#[allow(clippy::type_complexity)]
pub struct BrowserTabBar {
    base: WidgetBase,
    tabs: Vec<BrowserTabData>,
    selected_id: Option<String>,
    hovered_tab: Option<String>,
    hovered_close: Option<String>,
    tab_min_width: f32,
    tab_max_width: f32,
    show_new_tab_button: bool,
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_close: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_new_tab: Option<Box<dyn Fn() + Send + Sync>>,
}

impl BrowserTabBar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("browser-tab-bar"),
            tabs: Vec::new(),
            selected_id: None,
            hovered_tab: None,
            hovered_close: None,
            tab_min_width: 80.0,
            tab_max_width: 240.0,
            show_new_tab_button: true,
            on_select: None,
            on_close: None,
            on_new_tab: None,
        }
    }

    pub fn tab(mut self, tab: BrowserTabData) -> Self {
        self.tabs.push(tab);
        self
    }

    pub fn tabs(mut self, tabs: Vec<BrowserTabData>) -> Self {
        self.tabs = tabs;
        self
    }

    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where F: Fn(&str) + Send + Sync + 'static {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn on_close<F>(mut self, handler: F) -> Self
    where F: Fn(&str) + Send + Sync + 'static {
        self.on_close = Some(Box::new(handler));
        self
    }

    pub fn on_new_tab<F>(mut self, handler: F) -> Self
    where F: Fn() + Send + Sync + 'static {
        self.on_new_tab = Some(Box::new(handler));
        self
    }

    fn calculate_tab_width(&self, available_width: f32) -> f32 {
        let num_tabs = self.tabs.len() as f32;
        let new_tab_width = if self.show_new_tab_button { 40.0 } else { 0.0 };
        let available = available_width - new_tab_width - 8.0;

        if num_tabs == 0.0 {
            return self.tab_max_width;
        }

        (available / num_tabs).clamp(self.tab_min_width, self.tab_max_width)
    }
}

impl Default for BrowserTabBar {
    fn default() -> Self { Self::new() }
}

impl Widget for BrowserTabBar {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "browser-tab-bar" }
    fn classes(&self) -> &ClassList { &self.base.classes }
    fn state(&self) -> WidgetState { self.base.state }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(400.0, 36.0)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        LayoutResult::new(Size::new(constraints.max_width.min(1920.0), 36.0))
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = &ctx.style_ctx.theme;
        let tab_width = self.calculate_tab_width(rect.width());

        // Draw background
        painter.fill_rect(rect, theme.colors.card);

        // Draw tabs
        for (i, tab) in self.tabs.iter().enumerate() {
            let tab_x = rect.x() + (i as f32 * tab_width);
            let is_selected = self.selected_id.as_ref() == Some(&tab.id);
            let is_hovered = self.hovered_tab.as_ref() == Some(&tab.id);

            let bg_color = if is_selected {
                theme.colors.background
            } else if is_hovered {
                theme.colors.surface_hover
            } else {
                Color::TRANSPARENT
            };

            painter.fill_rounded_rect(
                Rect::new(tab_x + 2.0, rect.y() + 4.0, tab_width - 4.0, rect.height() - 4.0),
                bg_color,
                BorderRadius::all(4.0),
            );

            // Loading progress bar
            if tab.loading_state == TabLoadingState::Loading {
                let progress_width = (tab_width - 8.0) * tab.loading_progress;
                painter.fill_rect(
                    Rect::new(tab_x + 4.0, rect.y() + rect.height() - 3.0, progress_width, 2.0),
                    theme.colors.accent,
                );
            }

            // Content
            let mut content_x = tab_x + 12.0;

            if !tab.is_pinned {
                // Favicon or status
                if let Some(icon) = tab.status_icon() {
                    painter.draw_text(icon, Point::new(content_x, rect.y() + rect.height() / 2.0 - 8.0), theme.colors.foreground, 14.0);
                    content_x += 20.0;
                } else if let Some(favicon) = &tab.favicon {
                    painter.draw_text(favicon, Point::new(content_x, rect.y() + rect.height() / 2.0 - 8.0), theme.colors.foreground, 14.0);
                    content_x += 20.0;
                }

                // Title
                let title_width = tab_width - content_x + tab_x - 32.0;
                if title_width > 20.0 {
                    let title = tab.display_title((title_width / 7.0) as usize);
                    painter.draw_text(
                        &title,
                        Point::new(content_x, rect.y() + rect.height() / 2.0 - 7.0),
                        if is_selected { theme.colors.foreground } else { theme.colors.muted_foreground },
                        13.0,
                    );
                }

                // Close button
                if is_hovered {
                    let close_x = tab_x + tab_width - 28.0;
                    painter.draw_text("×", Point::new(close_x + 5.0, rect.y() + 10.0), theme.colors.muted_foreground, 14.0);
                }
            } else {
                // Pinned tab
                if let Some(favicon) = &tab.favicon {
                    painter.draw_text(favicon, Point::new(tab_x + tab_width / 2.0 - 8.0, rect.y() + rect.height() / 2.0 - 8.0), theme.colors.foreground, 16.0);
                }
            }
        }

        // New tab button
        if self.show_new_tab_button {
            let btn_x = rect.x() + (self.tabs.len() as f32 * tab_width) + 4.0;
            painter.fill_rounded_rect(
                Rect::new(btn_x, rect.y() + 8.0, 28.0, 20.0),
                theme.colors.surface_hover,
                BorderRadius::all(4.0),
            );
            painter.draw_text("+", Point::new(btn_x + 8.0, rect.y() + 10.0), theme.colors.muted_foreground, 14.0);
        }

        // Bottom border
        painter.fill_rect(Rect::new(rect.x(), rect.y() + rect.height() - 1.0, rect.width(), 1.0), theme.colors.border);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let tab_width = self.calculate_tab_width(self.base.bounds.width());

        if let Event::Mouse(mouse) = event {
            let pos = mouse.position;
            let bounds = self.base.bounds;

            match mouse.kind {
                MouseEventKind::Move => {
                    if bounds.contains(pos) {
                        let x_offset = pos.x - bounds.x();
                        let tab_index = (x_offset / tab_width) as usize;
                        self.hovered_tab = self.tabs.get(tab_index).map(|t| t.id.clone());
                        ctx.request_redraw();
                        return EventResult::Handled;
                    } else if self.hovered_tab.is_some() {
                        self.hovered_tab = None;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down => {
                    if mouse.button == Some(MouseButton::Left) && bounds.contains(pos) {
                        let x_offset = pos.x - bounds.x();
                        let tab_index = (x_offset / tab_width) as usize;

                        // Check new tab button
                        let btn_x = bounds.x() + (self.tabs.len() as f32 * tab_width) + 4.0;
                        if pos.x >= btn_x && pos.x < btn_x + 28.0 && self.show_new_tab_button {
                            if let Some(ref handler) = self.on_new_tab {
                                handler();
                            }
                            return EventResult::Handled;
                        }

                        // Select tab
                        if let Some(tab) = self.tabs.get(tab_index) {
                            let tab_id = tab.id.clone();
                            self.selected_id = Some(tab_id.clone());
                            if let Some(ref handler) = self.on_select {
                                handler(&tab_id);
                            }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    // Middle click to close
                    if mouse.button == Some(MouseButton::Middle) && bounds.contains(pos) {
                        let x_offset = pos.x - bounds.x();
                        let tab_index = (x_offset / tab_width) as usize;
                        if let Some(tab) = self.tabs.get(tab_index) {
                            if let Some(ref handler) = self.on_close {
                                handler(&tab.id);
                            }
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
