//! Browser navigation bar with back/forward/reload/home buttons.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Navigation button type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavButton {
    Back,
    Forward,
    Reload,
    Stop,
    Home,
}

impl NavButton {
    pub fn icon(&self) -> &'static str {
        match self {
            NavButton::Back => "←",
            NavButton::Forward => "→",
            NavButton::Reload => "↻",
            NavButton::Stop => "✕",
            NavButton::Home => "🏠",
        }
    }
}

/// Browser navigation bar widget.
#[allow(clippy::type_complexity)]
pub struct NavigationBar {
    base: WidgetBase,
    can_go_back: bool,
    can_go_forward: bool,
    is_loading: bool,
    show_home_button: bool,
    hovered_button: Option<NavButton>,
    on_back: Option<Box<dyn Fn() + Send + Sync>>,
    on_forward: Option<Box<dyn Fn() + Send + Sync>>,
    on_reload: Option<Box<dyn Fn() + Send + Sync>>,
    on_stop: Option<Box<dyn Fn() + Send + Sync>>,
    on_home: Option<Box<dyn Fn() + Send + Sync>>,
}

impl NavigationBar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("navigation-bar"),
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
            show_home_button: true,
            hovered_button: None,
            on_back: None,
            on_forward: None,
            on_reload: None,
            on_stop: None,
            on_home: None,
        }
    }

    pub fn can_go_back(mut self, can: bool) -> Self { self.can_go_back = can; self }
    pub fn can_go_forward(mut self, can: bool) -> Self { self.can_go_forward = can; self }
    pub fn is_loading(mut self, loading: bool) -> Self { self.is_loading = loading; self }
    pub fn show_home_button(mut self, show: bool) -> Self { self.show_home_button = show; self }

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
    pub fn on_home<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static {
        self.on_home = Some(Box::new(handler)); self
    }

    fn get_width(&self) -> f32 {
        let mut width = 108.0; // Back + Forward + Reload
        if self.show_home_button { width += 36.0; }
        width
    }
}

impl Default for NavigationBar {
    fn default() -> Self { Self::new() }
}

impl Widget for NavigationBar {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "navigation-bar" }
    fn classes(&self) -> &ClassList { &self.base.classes }
    fn state(&self) -> WidgetState { self.base.state }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(self.get_width(), 36.0)
    }

    fn layout(&mut self, _constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        LayoutResult::new(Size::new(self.get_width(), 36.0))
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = &ctx.style_ctx.theme;
        let button_size = 32.0;
        let y_center = rect.y() + rect.height() / 2.0;
        let button_y = y_center - button_size / 2.0;
        let mut x = rect.x();

        let buttons = [
            (NavButton::Back, self.can_go_back),
            (NavButton::Forward, self.can_go_forward),
            (if self.is_loading { NavButton::Stop } else { NavButton::Reload }, true),
        ];

        for (button, enabled) in buttons {
            let is_hovered = self.hovered_button == Some(button);
            if is_hovered && enabled {
                painter.fill_rounded_rect(
                    Rect::new(x + 2.0, button_y, button_size, button_size),
                    theme.colors.surface_hover,
                    BorderRadius::all(6.0),
                );
            }

            let color = if enabled {
                if is_hovered { theme.colors.foreground } else { theme.colors.muted_foreground }
            } else {
                theme.colors.muted_foreground.with_alpha(0.4)
            };

            painter.draw_text(button.icon(), Point::new(x + 10.0, y_center - 9.0), color, 18.0);
            x += 36.0;
        }

        if self.show_home_button {
            let is_hovered = self.hovered_button == Some(NavButton::Home);
            if is_hovered {
                painter.fill_rounded_rect(
                    Rect::new(x + 2.0, button_y, button_size, button_size),
                    theme.colors.surface_hover,
                    BorderRadius::all(6.0),
                );
            }
            let color = if is_hovered { theme.colors.foreground } else { theme.colors.muted_foreground };
            painter.draw_text(NavButton::Home.icon(), Point::new(x + 10.0, y_center - 9.0), color, 18.0);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let bounds = self.base.bounds;

        if let Event::Mouse(mouse) = event {
            let pos = mouse.position;

            match mouse.kind {
                MouseEventKind::Move => {
                    if bounds.contains(pos) {
                        let x = pos.x - bounds.x();
                        let button_idx = (x / 36.0) as usize;
                        let buttons = [NavButton::Back, NavButton::Forward,
                            if self.is_loading { NavButton::Stop } else { NavButton::Reload }, NavButton::Home];
                        self.hovered_button = buttons.get(button_idx).copied();
                        ctx.request_redraw();
                        return EventResult::Handled;
                    } else if self.hovered_button.is_some() {
                        self.hovered_button = None;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down => {
                    if mouse.button == Some(MouseButton::Left) {
                        if let Some(button) = self.hovered_button {
                            match button {
                                NavButton::Back if self.can_go_back => {
                                    if let Some(ref h) = self.on_back { h(); }
                                }
                                NavButton::Forward if self.can_go_forward => {
                                    if let Some(ref h) = self.on_forward { h(); }
                                }
                                NavButton::Reload => {
                                    if let Some(ref h) = self.on_reload { h(); }
                                }
                                NavButton::Stop => {
                                    if let Some(ref h) = self.on_stop { h(); }
                                }
                                NavButton::Home => {
                                    if let Some(ref h) = self.on_home { h(); }
                                }
                                _ => {}
                            }
                            ctx.request_redraw();
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
