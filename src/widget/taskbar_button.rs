//! Taskbar button widget
//!
//! A button for the taskbar showing an app with optional preview.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Taskbar button state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskbarButtonState {
    #[default]
    Normal,
    Hovered,
    Active,
    Attention,
}

/// Taskbar button progress state
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TaskbarProgress {
    #[default]
    None,
    Indeterminate,
    Progress(f32),
    Paused(f32),
    Error(f32),
}

/// A window in a taskbar button group
#[derive(Debug, Clone)]
pub struct TaskbarWindow {
    pub id: String,
    pub title: String,
    pub preview: Option<String>,
    pub is_active: bool,
}

/// Taskbar button widget
pub struct TaskbarButton {
    base: WidgetBase,
    app_id: String,
    app_name: String,
    icon: Option<String>,
    windows: Vec<TaskbarWindow>,
    is_pinned: bool,
    state: TaskbarButtonState,
    progress: TaskbarProgress,
    badge: Option<u32>,
    icon_size: f32,
    button_size: f32,
    accent_color: Color,
    #[allow(clippy::type_complexity)]
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_right_click: Option<Box<dyn Fn() + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_window_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_window_close: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl TaskbarButton {
    pub fn new(app_id: impl Into<String>, app_name: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new().with_class("taskbar-button"),
            app_id: app_id.into(),
            app_name: app_name.into(),
            icon: None,
            windows: Vec::new(),
            is_pinned: false,
            state: TaskbarButtonState::Normal,
            progress: TaskbarProgress::None,
            badge: None,
            icon_size: 24.0,
            button_size: 48.0,
            accent_color: Color::rgb(0.0, 0.47, 0.84),
            on_click: None,
            on_right_click: None,
            on_window_select: None,
            on_window_close: None,
        }
    }

    pub fn icon(mut self, path: impl Into<String>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn window(mut self, window: TaskbarWindow) -> Self {
        self.windows.push(window);
        self
    }

    pub fn windows(mut self, windows: Vec<TaskbarWindow>) -> Self {
        self.windows = windows;
        self
    }

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.is_pinned = pinned;
        self
    }

    pub fn progress(mut self, progress: TaskbarProgress) -> Self {
        self.progress = progress;
        self
    }

    pub fn badge(mut self, count: u32) -> Self {
        self.badge = if count > 0 { Some(count) } else { None };
        self
    }

    pub fn sizes(mut self, icon_size: f32, button_size: f32) -> Self {
        self.icon_size = icon_size;
        self.button_size = button_size;
        self
    }

    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn on_right_click<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_right_click = Some(Box::new(f));
        self
    }

    pub fn on_window_select<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_window_select = Some(Box::new(f));
        self
    }

    pub fn on_window_close<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_window_close = Some(Box::new(f));
        self
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn has_windows(&self) -> bool {
        !self.windows.is_empty()
    }

    pub fn has_active_window(&self) -> bool {
        self.windows.iter().any(|w| w.is_active)
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn set_state(&mut self, state: TaskbarButtonState) {
        self.state = state;
    }

    pub fn add_window(&mut self, window: TaskbarWindow) {
        self.windows.push(window);
    }

    pub fn remove_window(&mut self, id: &str) {
        self.windows.retain(|w| w.id != id);
    }

    pub fn set_active_window(&mut self, id: &str) {
        for window in &mut self.windows {
            window.is_active = window.id == id;
        }
    }

    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }
}

impl Widget for TaskbarButton {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "taskbar-button"
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
        Size::new(self.button_size, self.button_size)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        // Background based on state
        let bg_color = match self.state {
            TaskbarButtonState::Normal => Color::TRANSPARENT,
            TaskbarButtonState::Hovered => Color::rgba(1.0, 1.0, 1.0, 0.1),
            TaskbarButtonState::Active => Color::rgba(1.0, 1.0, 1.0, 0.15),
            TaskbarButtonState::Attention => Color::rgba(1.0, 0.5, 0.0, 0.3),
        };
        painter.fill_rounded_rect(rect, bg_color, BorderRadius::all(4.0));

        // Icon placeholder
        let icon_rect = Rect::new(
            rect.x() + (rect.width() - self.icon_size) / 2.0,
            rect.y() + (rect.height() - self.icon_size) / 2.0 - 4.0,
            self.icon_size,
            self.icon_size,
        );
        painter.fill_rounded_rect(icon_rect, Color::rgba(1.0, 1.0, 1.0, 0.3), BorderRadius::all(4.0));

        // App name (first char as placeholder)
        if let Some(c) = self.app_name.chars().next() {
            painter.draw_text(
                &c.to_string(),
                Point::new(icon_rect.x() + 6.0, icon_rect.y() + self.icon_size - 4.0),
                Color::WHITE,
                self.icon_size - 8.0,
            );
        }

        // Active indicator
        if self.has_windows() {
            let indicator_width = if self.has_active_window() { 24.0 } else { 8.0 };
            let indicator_rect = Rect::new(
                rect.x() + (rect.width() - indicator_width) / 2.0,
                rect.max_y() - 3.0,
                indicator_width,
                3.0,
            );
            painter.fill_rounded_rect(indicator_rect, self.accent_color, BorderRadius::all(1.5));
        }

        // Progress bar
        match self.progress {
            TaskbarProgress::None => {}
            TaskbarProgress::Indeterminate => {
                let progress_rect = Rect::new(rect.x() + 4.0, rect.max_y() - 6.0, rect.width() - 8.0, 3.0);
                painter.fill_rounded_rect(progress_rect, Color::rgba(0.0, 0.47, 0.84, 0.5), BorderRadius::all(1.5));
            }
            TaskbarProgress::Progress(p) | TaskbarProgress::Paused(p) => {
                let progress_rect = Rect::new(rect.x() + 4.0, rect.max_y() - 6.0, (rect.width() - 8.0) * p, 3.0);
                let color = match self.progress {
                    TaskbarProgress::Paused(_) => Color::rgb(0.8, 0.6, 0.0),
                    _ => Color::rgb(0.0, 0.6, 0.2),
                };
                painter.fill_rounded_rect(progress_rect, color, BorderRadius::all(1.5));
            }
            TaskbarProgress::Error(p) => {
                let progress_rect = Rect::new(rect.x() + 4.0, rect.max_y() - 6.0, (rect.width() - 8.0) * p, 3.0);
                painter.fill_rounded_rect(progress_rect, Color::rgb(0.8, 0.2, 0.2), BorderRadius::all(1.5));
            }
        }

        // Badge
        if let Some(count) = self.badge {
            let badge_size = 16.0;
            let badge_rect = Rect::new(rect.max_x() - badge_size - 2.0, rect.y() + 2.0, badge_size, badge_size);
            painter.fill_rounded_rect(badge_rect, Color::rgb(0.8, 0.2, 0.2), BorderRadius::all(badge_size / 2.0));
            
            let badge_text = if count > 99 { "99+".to_string() } else { count.to_string() };
            painter.draw_text(
                &badge_text,
                Point::new(badge_rect.x() + 3.0, badge_rect.y() + 12.0),
                Color::WHITE,
                9.0,
            );
        }
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;
    }
}
