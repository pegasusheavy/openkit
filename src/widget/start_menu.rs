//! Windows 10-style Start Menu widget
//!
//! A comprehensive start menu with pinned apps, all apps list, power options,
//! and user profile section.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A menu item in the start menu
#[derive(Debug, Clone)]
pub struct StartMenuItem {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Icon path or name
    pub icon: Option<String>,
    /// Command to execute
    pub command: Option<String>,
    /// Whether this is a folder/category
    pub is_folder: bool,
    /// Child items (for folders)
    pub children: Vec<StartMenuItem>,
}

impl StartMenuItem {
    /// Create a new app item
    pub fn app(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: None,
            command: None,
            is_folder: false,
            children: Vec::new(),
        }
    }

    /// Create a new folder item
    pub fn folder(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: None,
            command: None,
            is_folder: true,
            children: Vec::new(),
        }
    }

    /// Set the icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the command
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.command = Some(cmd.into());
        self
    }

    /// Add a child item
    pub fn child(mut self, item: StartMenuItem) -> Self {
        self.children.push(item);
        self
    }
}

/// Power action type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown,
    Restart,
    Suspend,
    Hibernate,
    Lock,
    Logout,
}

/// Start menu section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StartMenuSection {
    #[default]
    Pinned,
    Recent,
    AllApps,
    Power,
    User,
}

/// Windows 10-style Start Menu
pub struct StartMenu {
    base: WidgetBase,
    is_open: bool,
    current_section: StartMenuSection,
    pinned_apps: Vec<StartMenuItem>,
    recent_items: Vec<StartMenuItem>,
    all_apps: Vec<StartMenuItem>,
    user_name: String,
    user_avatar: Option<String>,
    search_query: String,
    search_active: bool,
    width: f32,
    height: f32,
    accent_color: Color,
    blur_enabled: bool,
    #[allow(clippy::type_complexity)]
    on_item_click: Option<Box<dyn Fn(&StartMenuItem) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_power_action: Option<Box<dyn Fn(PowerAction) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_search: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl StartMenu {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("start-menu"),
            is_open: false,
            current_section: StartMenuSection::Pinned,
            pinned_apps: Vec::new(),
            recent_items: Vec::new(),
            all_apps: Vec::new(),
            user_name: String::from("User"),
            user_avatar: None,
            search_query: String::new(),
            search_active: false,
            width: 600.0,
            height: 720.0,
            accent_color: Color::rgb(0.0, 0.47, 0.84),
            blur_enabled: true,
            on_item_click: None,
            on_power_action: None,
            on_search: None,
        }
    }

    pub fn user_name(mut self, name: impl Into<String>) -> Self {
        self.user_name = name.into();
        self
    }

    pub fn user_avatar(mut self, path: impl Into<String>) -> Self {
        self.user_avatar = Some(path.into());
        self
    }

    pub fn pinned(mut self, item: StartMenuItem) -> Self {
        self.pinned_apps.push(item);
        self
    }

    pub fn recent(mut self, item: StartMenuItem) -> Self {
        self.recent_items.push(item);
        self
    }

    pub fn app(mut self, item: StartMenuItem) -> Self {
        self.all_apps.push(item);
        self
    }

    pub fn apps(mut self, apps: Vec<StartMenuItem>) -> Self {
        self.all_apps = apps;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    pub fn blur(mut self, enabled: bool) -> Self {
        self.blur_enabled = enabled;
        self
    }

    pub fn on_item_click<F>(mut self, f: F) -> Self
    where
        F: Fn(&StartMenuItem) + Send + Sync + 'static,
    {
        self.on_item_click = Some(Box::new(f));
        self
    }

    pub fn on_power_action<F>(mut self, f: F) -> Self
    where
        F: Fn(PowerAction) + Send + Sync + 'static,
    {
        self.on_power_action = Some(Box::new(f));
        self
    }

    pub fn on_search<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_search = Some(Box::new(f));
        self
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.search_query.clear();
        self.search_active = false;
        self.current_section = StartMenuSection::Pinned;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_active = !query.is_empty();
        if let Some(ref cb) = self.on_search {
            cb(query);
        }
    }

    pub fn set_section(&mut self, section: StartMenuSection) {
        self.current_section = section;
    }

    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }

    pub fn get_filtered_apps(&self) -> Vec<&StartMenuItem> {
        if self.search_query.is_empty() {
            return self.all_apps.iter().collect();
        }
        let query = self.search_query.to_lowercase();
        self.all_apps
            .iter()
            .filter(|app| app.name.to_lowercase().contains(&query))
            .collect()
    }
}

impl Default for StartMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for StartMenu {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "start-menu"
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
        let bg_color = if self.blur_enabled {
            Color::rgba(0.1, 0.1, 0.1, 0.85)
        } else {
            Color::rgb(0.12, 0.12, 0.12)
        };
        painter.fill_rounded_rect(rect, bg_color, BorderRadius::all(8.0));

        // Accent bar at left
        let accent_bar = Rect::new(rect.x(), rect.y(), 3.0, rect.height());
        painter.fill_rect(accent_bar, self.accent_color);

        // Search box area
        let search_rect = Rect::new(rect.x() + 16.0, rect.y() + 16.0, rect.width() - 32.0, 40.0);
        painter.fill_rounded_rect(search_rect, Color::rgba(1.0, 1.0, 1.0, 0.1), BorderRadius::all(4.0));

        // User section at bottom
        let user_rect = Rect::new(rect.x(), rect.max_y() - 56.0, rect.width(), 56.0);
        painter.fill_rect(user_rect, Color::rgba(0.0, 0.0, 0.0, 0.2));

        // Draw user name
        painter.draw_text(
            &self.user_name,
            Point::new(rect.x() + 64.0, rect.max_y() - 24.0),
            Color::WHITE,
            14.0,
        );
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
