//! Action Center / Quick Settings widget
//!
//! Windows 10/11-style action center with quick toggles and notifications.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Quick action toggle
#[derive(Debug, Clone)]
pub struct QuickAction {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub is_active: bool,
    pub is_available: bool,
}

impl QuickAction {
    pub fn new(id: impl Into<String>, name: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            is_active: false,
            is_available: true,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn available(mut self, available: bool) -> Self {
        self.is_available = available;
        self
    }
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationPriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// A notification in the action center
#[derive(Debug, Clone)]
pub struct ActionNotification {
    pub id: String,
    pub app_name: String,
    pub app_icon: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub timestamp: u64,
    pub priority: NotificationPriority,
    pub is_read: bool,
    pub actions: Vec<(String, String)>,
}

impl ActionNotification {
    pub fn new(id: impl Into<String>, app: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            app_name: app.into(),
            app_icon: None,
            title: title.into(),
            body: None,
            timestamp: 0,
            priority: NotificationPriority::Normal,
            is_read: false,
            actions: Vec::new(),
        }
    }

    pub fn body(mut self, text: impl Into<String>) -> Self {
        self.body = Some(text.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.app_icon = Some(icon.into());
        self
    }

    pub fn priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn action(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.actions.push((id.into(), label.into()));
        self
    }
}

/// Action center widget
pub struct ActionCenter {
    base: WidgetBase,
    quick_actions: Vec<QuickAction>,
    notifications: Vec<ActionNotification>,
    is_open: bool,
    current_section: usize,
    width: f32,
    height: f32,
    brightness: f32,
    show_brightness: bool,
    do_not_disturb: bool,
    #[allow(clippy::type_complexity)]
    on_action_toggle: Option<Box<dyn Fn(&str, bool) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_notification_action: Option<Box<dyn Fn(&str, &str) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_notification_dismiss: Option<Box<dyn Fn(&str) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_clear_all: Option<Box<dyn Fn() + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_brightness_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
}

impl ActionCenter {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("action-center"),
            quick_actions: vec![
                QuickAction::new("wifi", "Wi-Fi", "📶"),
                QuickAction::new("bluetooth", "Bluetooth", "🔵"),
                QuickAction::new("airplane", "Airplane Mode", "✈️"),
                QuickAction::new("battery_saver", "Battery Saver", "🔋"),
                QuickAction::new("night_light", "Night Light", "🌙"),
                QuickAction::new("focus", "Focus Assist", "🎯"),
            ],
            notifications: Vec::new(),
            is_open: false,
            current_section: 0,
            width: 380.0,
            height: 600.0,
            brightness: 0.75,
            show_brightness: true,
            do_not_disturb: false,
            on_action_toggle: None,
            on_notification_action: None,
            on_notification_dismiss: None,
            on_clear_all: None,
            on_brightness_change: None,
        }
    }

    pub fn quick_actions(mut self, actions: Vec<QuickAction>) -> Self {
        self.quick_actions = actions;
        self
    }

    pub fn action(mut self, action: QuickAction) -> Self {
        self.quick_actions.push(action);
        self
    }

    pub fn notifications(mut self, notifications: Vec<ActionNotification>) -> Self {
        self.notifications = notifications;
        self
    }

    pub fn notification(mut self, notification: ActionNotification) -> Self {
        self.notifications.push(notification);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn show_brightness(mut self, show: bool) -> Self {
        self.show_brightness = show;
        self
    }

    pub fn brightness(mut self, level: f32) -> Self {
        self.brightness = level.clamp(0.0, 1.0);
        self
    }

    pub fn do_not_disturb(mut self, enabled: bool) -> Self {
        self.do_not_disturb = enabled;
        self
    }

    pub fn on_action_toggle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, bool) + Send + Sync + 'static,
    {
        self.on_action_toggle = Some(Box::new(f));
        self
    }

    pub fn on_notification_action<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.on_notification_action = Some(Box::new(f));
        self
    }

    pub fn on_notification_dismiss<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_notification_dismiss = Some(Box::new(f));
        self
    }

    pub fn on_clear_all<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_clear_all = Some(Box::new(f));
        self
    }

    pub fn on_brightness_change<F>(mut self, f: F) -> Self
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_brightness_change = Some(Box::new(f));
        self
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn add_notification(&mut self, notification: ActionNotification) {
        self.notifications.insert(0, notification);
    }

    pub fn remove_notification(&mut self, id: &str) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    pub fn unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.is_read).count()
    }

    pub fn toggle_action(&mut self, id: &str) {
        if let Some(action) = self.quick_actions.iter_mut().find(|a| a.id == id) {
            action.is_active = !action.is_active;
            let active = action.is_active;
            if let Some(ref cb) = self.on_action_toggle {
                cb(id, active);
            }
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

impl Default for ActionCenter {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ActionCenter {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "action-center"
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
        if !self.is_open {
            Size::ZERO
        } else {
            Size::new(self.width, self.height)
        }
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        if !self.is_open {
            return;
        }

        // Background
        painter.fill_rounded_rect(rect, Color::rgba(0.1, 0.1, 0.1, 0.95), BorderRadius::all(8.0));

        // Header
        let header_rect = Rect::new(rect.x(), rect.y(), rect.width(), 48.0);
        painter.fill_rect(header_rect, Color::rgba(1.0, 1.0, 1.0, 0.05));

        // Header title
        painter.draw_text(
            "Quick actions",
            Point::new(rect.x() + 16.0, rect.y() + 30.0),
            Color::WHITE,
            14.0,
        );

        // Quick actions grid
        let grid_top = rect.y() + 56.0;
        let tile_size = 80.0;
        let tiles_per_row = 4;
        let padding = 8.0;

        for (i, action) in self.quick_actions.iter().enumerate() {
            let row = i / tiles_per_row;
            let col = i % tiles_per_row;

            let tile_rect = Rect::new(
                rect.x() + padding + (col as f32 * (tile_size + padding)),
                grid_top + (row as f32 * (tile_size + padding)),
                tile_size,
                tile_size,
            );

            let bg_color = if action.is_active {
                Color::rgba(0.0, 0.47, 0.84, 0.8)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.1)
            };
            painter.fill_rounded_rect(tile_rect, bg_color, BorderRadius::all(4.0));

            // Icon
            painter.draw_text(
                &action.icon,
                Point::new(tile_rect.x() + tile_size / 2.0 - 10.0, tile_rect.y() + 36.0),
                Color::WHITE,
                20.0,
            );

            // Label
            painter.draw_text(
                &action.name,
                Point::new(tile_rect.x() + 4.0, tile_rect.y() + tile_size - 8.0),
                Color::WHITE,
                10.0,
            );
        }

        // Brightness slider
        if self.show_brightness {
            let rows = (self.quick_actions.len() as f32 / tiles_per_row as f32).ceil();
            let slider_top = grid_top + (rows * (tile_size + padding)) + 16.0;

            painter.draw_text("☀️", Point::new(rect.x() + 16.0, slider_top + 4.0), Color::WHITE, 16.0);

            let track_rect = Rect::new(rect.x() + 48.0, slider_top, rect.width() - 64.0, 4.0);
            painter.fill_rounded_rect(track_rect, Color::rgba(1.0, 1.0, 1.0, 0.2), BorderRadius::all(2.0));

            let fill_width = (rect.width() - 64.0) * self.brightness;
            let fill_rect = Rect::new(rect.x() + 48.0, slider_top, fill_width, 4.0);
            painter.fill_rounded_rect(fill_rect, Color::rgb(1.0, 0.8, 0.2), BorderRadius::all(2.0));
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
