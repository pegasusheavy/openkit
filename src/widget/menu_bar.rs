//! Menu bar widget for application menus.
//!
//! Provides a traditional menu bar with dropdown menus, keyboard shortcuts,
//! submenus, checkable items, and separators.
//!
//! # Example
//!
//! ```rust,ignore
//! use openkit::widget::menu_bar::*;
//!
//! let menu_bar = MenuBar::new()
//!     .menu(
//!         Menu::new("File")
//!             .item(MenuItem::new("New", "Ctrl+N").on_click(|| println!("New")))
//!             .item(MenuItem::new("Open...", "Ctrl+O").on_click(|| println!("Open")))
//!             .item(MenuItem::new("Save", "Ctrl+S").on_click(|| println!("Save")))
//!             .separator()
//!             .item(MenuItem::new("Exit", "Alt+F4").on_click(|| println!("Exit")))
//!     )
//!     .menu(
//!         Menu::new("Edit")
//!             .item(MenuItem::new("Undo", "Ctrl+Z"))
//!             .item(MenuItem::new("Redo", "Ctrl+Y"))
//!             .separator()
//!             .item(MenuItem::new("Cut", "Ctrl+X"))
//!             .item(MenuItem::new("Copy", "Ctrl+C"))
//!             .item(MenuItem::new("Paste", "Ctrl+V"))
//!     )
//!     .menu(
//!         Menu::new("View")
//!             .item(MenuItem::checkbox("Show Toolbar", true))
//!             .item(MenuItem::checkbox("Show Status Bar", true))
//!             .separator()
//!             .submenu(
//!                 Menu::new("Zoom")
//!                     .item(MenuItem::new("Zoom In", "Ctrl++"))
//!                     .item(MenuItem::new("Zoom Out", "Ctrl+-"))
//!                     .item(MenuItem::new("Reset Zoom", "Ctrl+0"))
//!             )
//!     );
//! ```

use super::{EventCallback, Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, Key, KeyEventKind};
use crate::geometry::{Point, Rect, Size, BorderRadius};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A menu item that can be a regular item, checkbox, radio, separator, or submenu.
#[derive(Debug, Clone)]
pub enum MenuItemKind {
    /// Regular clickable item
    Action {
        label: String,
        shortcut: Option<String>,
        icon: Option<String>,
        enabled: bool,
    },
    /// Checkbox item (toggleable)
    Checkbox {
        label: String,
        checked: bool,
        enabled: bool,
    },
    /// Radio item (one of a group)
    Radio {
        label: String,
        selected: bool,
        group: String,
        enabled: bool,
    },
    /// Visual separator
    Separator,
    /// Submenu containing more items
    Submenu {
        label: String,
        menu: Menu,
        enabled: bool,
    },
}

/// A single menu item.
#[allow(clippy::type_complexity)]
pub struct MenuItem {
    id: String,
    kind: MenuItemKind,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    on_toggle: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl MenuItem {
    /// Create a new action menu item.
    pub fn new(label: impl Into<String>, shortcut: impl Into<String>) -> Self {
        let label = label.into();
        let id = label.to_lowercase().replace(' ', "_");
        Self {
            id,
            kind: MenuItemKind::Action {
                label,
                shortcut: Some(shortcut.into()),
                icon: None,
                enabled: true,
            },
            on_click: None,
            on_toggle: None,
        }
    }

    /// Create a menu item without a shortcut.
    pub fn action(label: impl Into<String>) -> Self {
        let label = label.into();
        let id = label.to_lowercase().replace(' ', "_");
        Self {
            id,
            kind: MenuItemKind::Action {
                label,
                shortcut: None,
                icon: None,
                enabled: true,
            },
            on_click: None,
            on_toggle: None,
        }
    }

    /// Create a checkbox menu item.
    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        let label = label.into();
        let id = label.to_lowercase().replace(' ', "_");
        Self {
            id,
            kind: MenuItemKind::Checkbox {
                label,
                checked,
                enabled: true,
            },
            on_click: None,
            on_toggle: None,
        }
    }

    /// Create a radio menu item.
    pub fn radio(label: impl Into<String>, group: impl Into<String>, selected: bool) -> Self {
        let label = label.into();
        let id = label.to_lowercase().replace(' ', "_");
        Self {
            id,
            kind: MenuItemKind::Radio {
                label,
                selected,
                group: group.into(),
                enabled: true,
            },
            on_click: None,
            on_toggle: None,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self {
            id: "separator".to_string(),
            kind: MenuItemKind::Separator,
            on_click: None,
            on_toggle: None,
        }
    }

    /// Set the item ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set an icon for the item.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        if let MenuItemKind::Action { icon: ref mut i, .. } = self.kind {
            *i = Some(icon.into());
        }
        self
    }

    /// Set enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        match &mut self.kind {
            MenuItemKind::Action { enabled: e, .. } => *e = enabled,
            MenuItemKind::Checkbox { enabled: e, .. } => *e = enabled,
            MenuItemKind::Radio { enabled: e, .. } => *e = enabled,
            MenuItemKind::Submenu { enabled: e, .. } => *e = enabled,
            MenuItemKind::Separator => {}
        }
        self
    }

    /// Set click handler.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set toggle handler (for checkbox/radio items).
    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Get the label text.
    pub fn label(&self) -> &str {
        match &self.kind {
            MenuItemKind::Action { label, .. } => label,
            MenuItemKind::Checkbox { label, .. } => label,
            MenuItemKind::Radio { label, .. } => label,
            MenuItemKind::Submenu { label, .. } => label,
            MenuItemKind::Separator => "",
        }
    }

    /// Check if the item is enabled.
    pub fn is_enabled(&self) -> bool {
        match &self.kind {
            MenuItemKind::Action { enabled, .. } => *enabled,
            MenuItemKind::Checkbox { enabled, .. } => *enabled,
            MenuItemKind::Radio { enabled, .. } => *enabled,
            MenuItemKind::Submenu { enabled, .. } => *enabled,
            MenuItemKind::Separator => false,
        }
    }

    /// Check if this is a separator.
    pub fn is_separator(&self) -> bool {
        matches!(self.kind, MenuItemKind::Separator)
    }

    /// Toggle checkbox state.
    pub fn toggle(&mut self) {
        if let MenuItemKind::Checkbox { checked, enabled, .. } = &mut self.kind {
            if *enabled {
                *checked = !*checked;
                if let Some(handler) = &self.on_toggle {
                    handler(*checked);
                }
            }
        }
    }
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .finish()
    }
}

/// A dropdown menu containing menu items.
pub struct Menu {
    label: String,
    items: Vec<MenuItem>,
    min_width: f32,
}

impl Menu {
    /// Create a new menu.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
            min_width: 150.0,
        }
    }

    /// Add a menu item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add a separator.
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem::separator());
        self
    }

    /// Add a submenu.
    pub fn submenu(mut self, menu: Menu) -> Self {
        let label = menu.label.clone();
        self.items.push(MenuItem {
            id: label.to_lowercase().replace(' ', "_"),
            kind: MenuItemKind::Submenu {
                label,
                menu,
                enabled: true,
            },
            on_click: None,
            on_toggle: None,
        });
        self
    }

    /// Set minimum width.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Get the menu label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get the menu items.
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Get mutable items.
    pub fn items_mut(&mut self) -> &mut [MenuItem] {
        &mut self.items
    }
}

impl std::fmt::Debug for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menu")
            .field("label", &self.label)
            .field("items", &self.items.len())
            .finish()
    }
}

impl Clone for Menu {
    fn clone(&self) -> Self {
        // We can't clone the handlers, so we create a shallow clone
        Self {
            label: self.label.clone(),
            items: Vec::new(), // Items with handlers can't be cloned
            min_width: self.min_width,
        }
    }
}

/// Menu bar widget.
///
/// The menu bar sits at the top of a window and contains dropdown menus.
#[allow(clippy::type_complexity)]
pub struct MenuBar {
    base: WidgetBase,
    menus: Vec<Menu>,
    height: f32,
    active_menu: Option<usize>,
    hovered_menu: Option<usize>,
    hovered_item: Option<usize>,
    menu_open: bool,
    dropdown_rect: Option<Rect>,
    on_action: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl MenuBar {
    /// Create a new menu bar.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("menu-bar"),
            menus: Vec::new(),
            height: 28.0,
            active_menu: None,
            hovered_menu: None,
            hovered_item: None,
            menu_open: false,
            dropdown_rect: None,
            on_action: None,
        }
    }

    /// Add a menu.
    pub fn menu(mut self, menu: Menu) -> Self {
        self.menus.push(menu);
        self
    }

    /// Set the height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set action handler (called when any menu item is clicked).
    pub fn on_action<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_action = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the menus.
    pub fn menus(&self) -> &[Menu] {
        &self.menus
    }

    /// Get mutable menus.
    pub fn menus_mut(&mut self) -> &mut [Menu] {
        &mut self.menus
    }

    /// Close any open menu.
    pub fn close_menu(&mut self) {
        self.menu_open = false;
        self.active_menu = None;
        self.hovered_item = None;
        self.dropdown_rect = None;
    }

    /// Calculate menu header positions.
    fn menu_header_rects(&self) -> Vec<Rect> {
        let bounds = self.base.bounds;
        let mut rects = Vec::new();
        let mut x = bounds.x() + 8.0;
        let padding = 16.0;

        for menu in &self.menus {
            let label_width = menu.label.len() as f32 * 8.0 + padding * 2.0;
            rects.push(Rect::new(x, bounds.y(), label_width, self.height));
            x += label_width;
        }

        rects
    }

    /// Calculate dropdown rect for a menu.
    fn calculate_dropdown_rect(&self, menu_idx: usize) -> Rect {
        let header_rects = self.menu_header_rects();
        let header = header_rects.get(menu_idx).copied().unwrap_or(Rect::ZERO);
        let menu = &self.menus[menu_idx];

        let item_height = 28.0;
        let separator_height = 9.0;
        let padding = 4.0;

        let mut total_height = padding * 2.0;
        for item in &menu.items {
            if item.is_separator() {
                total_height += separator_height;
            } else {
                total_height += item_height;
            }
        }

        let width = menu.min_width.max(header.width());

        Rect::new(
            header.x(),
            header.y() + header.height(),
            width,
            total_height,
        )
    }

    /// Get item at point within dropdown.
    fn item_at_point(&self, menu_idx: usize, point: Point) -> Option<usize> {
        let dropdown_rect = self.calculate_dropdown_rect(menu_idx);
        if !dropdown_rect.contains(point) {
            return None;
        }

        let menu = &self.menus[menu_idx];
        let item_height = 28.0;
        let separator_height = 9.0;
        let padding = 4.0;

        let mut y = dropdown_rect.y() + padding;
        for (i, item) in menu.items.iter().enumerate() {
            let height = if item.is_separator() { separator_height } else { item_height };
            if point.y >= y && point.y < y + height {
                if !item.is_separator() && item.is_enabled() {
                    return Some(i);
                }
                return None;
            }
            y += height;
        }

        None
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MenuBar {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "menu-bar"
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
        Size::new(f32::INFINITY, self.height)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        let width = constraints.max_width;
        let size = Size::new(width, self.height);
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Menu bar background
        painter.fill_rect(rect, theme.colors.surface);
        painter.fill_rect(
            Rect::new(rect.x(), rect.y() + rect.height() - 1.0, rect.width(), 1.0),
            theme.colors.border,
        );

        // Draw menu headers
        let header_rects = self.menu_header_rects();
        for (i, menu) in self.menus.iter().enumerate() {
            let header_rect = header_rects[i];

            // Background for hovered/active menu
            let is_active = self.active_menu == Some(i);
            let is_hovered = self.hovered_menu == Some(i);

            if is_active {
                painter.fill_rect(header_rect, theme.colors.accent.with_alpha(0.2));
            } else if is_hovered {
                painter.fill_rect(header_rect, theme.colors.muted.with_alpha(0.5));
            }

            // Menu label
            let text_color = if is_active {
                theme.colors.accent
            } else {
                theme.colors.foreground
            };

            painter.draw_text(
                &menu.label,
                Point::new(header_rect.x() + 16.0, header_rect.y() + self.height / 2.0 + 4.0),
                text_color,
                13.0,
            );
        }

        // Draw dropdown if open
        if self.menu_open {
            if let Some(menu_idx) = self.active_menu {
                self.paint_dropdown(painter, menu_idx, ctx);
            }
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let header_rects = self.menu_header_rects();

        match event {
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Move => {
                        // Check if over a menu header
                        let mut new_hovered = None;
                        for (i, rect) in header_rects.iter().enumerate() {
                            if rect.contains(mouse.position) {
                                new_hovered = Some(i);
                                break;
                            }
                        }

                        if new_hovered != self.hovered_menu {
                            self.hovered_menu = new_hovered;

                            // If menu is open and hovering different header, switch menu
                            if self.menu_open && new_hovered.is_some() {
                                self.active_menu = new_hovered;
                                self.hovered_item = None;
                            }

                            ctx.request_redraw();
                        }

                        // Check if over dropdown item
                        if self.menu_open {
                            if let Some(menu_idx) = self.active_menu {
                                let dropdown_rect = self.calculate_dropdown_rect(menu_idx);
                                if dropdown_rect.contains(mouse.position) {
                                    let new_item = self.item_at_point(menu_idx, mouse.position);
                                    if new_item != self.hovered_item {
                                        self.hovered_item = new_item;
                                        ctx.request_redraw();
                                    }
                                    return EventResult::Handled;
                                }
                            }
                        }
                    }
                    MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                        // Check click on menu header
                        for (i, rect) in header_rects.iter().enumerate() {
                            if rect.contains(mouse.position) {
                                if self.menu_open && self.active_menu == Some(i) {
                                    // Close menu if clicking same header
                                    self.close_menu();
                                } else {
                                    // Open menu
                                    self.active_menu = Some(i);
                                    self.menu_open = true;
                                    self.hovered_item = None;
                                }
                                ctx.request_redraw();
                                return EventResult::Handled;
                            }
                        }

                        // Check click on dropdown item
                        if self.menu_open {
                            if let Some(menu_idx) = self.active_menu {
                                let dropdown_rect = self.calculate_dropdown_rect(menu_idx);
                                if dropdown_rect.contains(mouse.position) {
                                    if let Some(item_idx) = self.item_at_point(menu_idx, mouse.position) {
                                        let menu = &mut self.menus[menu_idx];
                                        let item = &mut menu.items[item_idx];

                                        // Handle click based on item type
                                        match &mut item.kind {
                                            MenuItemKind::Action { .. } => {
                                                if let Some(handler) = &item.on_click {
                                                    handler();
                                                }
                                                if let Some(action_handler) = &self.on_action {
                                                    action_handler(&item.id);
                                                }
                                            }
                                            MenuItemKind::Checkbox { checked, enabled, .. } => {
                                                if *enabled {
                                                    *checked = !*checked;
                                                    if let Some(handler) = &item.on_toggle {
                                                        handler(*checked);
                                                    }
                                                }
                                            }
                                            MenuItemKind::Radio { selected, enabled, .. } => {
                                                if *enabled && !*selected {
                                                    *selected = true;
                                                    // TODO: Deselect other radio items in group
                                                    if let Some(handler) = &item.on_toggle {
                                                        handler(true);
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }

                                        self.close_menu();
                                        ctx.request_redraw();
                                        return EventResult::Handled;
                                    }
                                } else {
                                    // Click outside dropdown, close it
                                    self.close_menu();
                                    ctx.request_redraw();
                                }
                            }
                        }
                    }
                    MouseEventKind::Leave => {
                        if self.hovered_menu.is_some() && !self.menu_open {
                            self.hovered_menu = None;
                            ctx.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Down => {
                // Handle keyboard navigation
                match key.key {
                    Key::Escape => {
                        if self.menu_open {
                            self.close_menu();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    Key::Left => {
                        if self.menu_open {
                            if let Some(idx) = self.active_menu {
                                self.active_menu = Some(if idx == 0 { self.menus.len() - 1 } else { idx - 1 });
                                self.hovered_item = None;
                                ctx.request_redraw();
                                return EventResult::Handled;
                            }
                        }
                    }
                    Key::Right => {
                        if self.menu_open {
                            if let Some(idx) = self.active_menu {
                                self.active_menu = Some((idx + 1) % self.menus.len());
                                self.hovered_item = None;
                                ctx.request_redraw();
                                return EventResult::Handled;
                            }
                        }
                    }
                    Key::Up => {
                        if self.menu_open {
                            if let Some(menu_idx) = self.active_menu {
                                let menu = &self.menus[menu_idx];
                                let item_count = menu.items.len();
                                if item_count > 0 {
                                    let current = self.hovered_item.unwrap_or(0);
                                    // Find previous non-separator, enabled item
                                    let mut new_idx = if current == 0 { item_count - 1 } else { current - 1 };
                                    for _ in 0..item_count {
                                        if !menu.items[new_idx].is_separator() && menu.items[new_idx].is_enabled() {
                                            break;
                                        }
                                        new_idx = if new_idx == 0 { item_count - 1 } else { new_idx - 1 };
                                    }
                                    self.hovered_item = Some(new_idx);
                                    ctx.request_redraw();
                                    return EventResult::Handled;
                                }
                            }
                        }
                    }
                    Key::Down => {
                        if self.menu_open {
                            if let Some(menu_idx) = self.active_menu {
                                let menu = &self.menus[menu_idx];
                                let item_count = menu.items.len();
                                if item_count > 0 {
                                    let current = self.hovered_item.unwrap_or(item_count - 1);
                                    // Find next non-separator, enabled item
                                    let mut new_idx = (current + 1) % item_count;
                                    for _ in 0..item_count {
                                        if !menu.items[new_idx].is_separator() && menu.items[new_idx].is_enabled() {
                                            break;
                                        }
                                        new_idx = (new_idx + 1) % item_count;
                                    }
                                    self.hovered_item = Some(new_idx);
                                    ctx.request_redraw();
                                    return EventResult::Handled;
                                }
                            }
                        }
                    }
                    Key::Enter => {
                        if self.menu_open {
                            if let (Some(menu_idx), Some(item_idx)) = (self.active_menu, self.hovered_item) {
                                let menu = &mut self.menus[menu_idx];
                                let item = &mut menu.items[item_idx];

                                if item.is_enabled() {
                                    match &mut item.kind {
                                        MenuItemKind::Action { .. } => {
                                            if let Some(handler) = &item.on_click {
                                                handler();
                                            }
                                            if let Some(action_handler) = &self.on_action {
                                                action_handler(&item.id);
                                            }
                                        }
                                        MenuItemKind::Checkbox { checked, .. } => {
                                            *checked = !*checked;
                                            if let Some(handler) = &item.on_toggle {
                                                handler(*checked);
                                            }
                                        }
                                        _ => {}
                                    }

                                    self.close_menu();
                                    ctx.request_redraw();
                                    return EventResult::Handled;
                                }
                            }
                        }
                    }
                    _ => {}
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

impl MenuBar {
    fn paint_dropdown(&self, painter: &mut Painter, menu_idx: usize, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let menu = &self.menus[menu_idx];
        let dropdown_rect = self.calculate_dropdown_rect(menu_idx);

        // Dropdown shadow
        let shadow_offset = 4.0;
        painter.fill_rounded_rect(
            Rect::new(
                dropdown_rect.x() + shadow_offset,
                dropdown_rect.y() + shadow_offset,
                dropdown_rect.width(),
                dropdown_rect.height(),
            ),
            theme.colors.background.with_alpha(0.3),
            BorderRadius::all(4.0),
        );

        // Dropdown background
        painter.fill_rounded_rect(dropdown_rect, theme.colors.card, BorderRadius::all(4.0));
        painter.stroke_rect(dropdown_rect, theme.colors.border, 1.0);

        // Draw items
        let item_height = 28.0;
        let separator_height = 9.0;
        let padding = 4.0;
        let horizontal_padding = 12.0;

        let mut y = dropdown_rect.y() + padding;

        for (i, item) in menu.items.iter().enumerate() {
            if item.is_separator() {
                // Draw separator
                let sep_y = y + separator_height / 2.0;
                painter.fill_rect(
                    Rect::new(
                        dropdown_rect.x() + horizontal_padding,
                        sep_y,
                        dropdown_rect.width() - horizontal_padding * 2.0,
                        1.0,
                    ),
                    theme.colors.border,
                );
                y += separator_height;
                continue;
            }

            let item_rect = Rect::new(
                dropdown_rect.x() + padding,
                y,
                dropdown_rect.width() - padding * 2.0,
                item_height,
            );

            let is_hovered = self.hovered_item == Some(i);
            let is_enabled = item.is_enabled();

            // Item background
            if is_hovered && is_enabled {
                painter.fill_rounded_rect(item_rect, theme.colors.accent.with_alpha(0.2), BorderRadius::all(4.0));
            }

            // Item content
            let text_color = if !is_enabled {
                theme.colors.muted_foreground
            } else if is_hovered {
                theme.colors.accent
            } else {
                theme.colors.foreground
            };

            let mut text_x = item_rect.x() + horizontal_padding;

            // Checkbox/radio indicator
            match &item.kind {
                MenuItemKind::Checkbox { checked, .. } => {
                    let check = if *checked { "✓" } else { " " };
                    painter.draw_text(
                        check,
                        Point::new(text_x, y + item_height / 2.0 + 4.0),
                        text_color,
                        13.0,
                    );
                    text_x += 20.0;
                }
                MenuItemKind::Radio { selected, .. } => {
                    let dot = if *selected { "●" } else { "○" };
                    painter.draw_text(
                        dot,
                        Point::new(text_x, y + item_height / 2.0 + 4.0),
                        text_color,
                        12.0,
                    );
                    text_x += 20.0;
                }
                MenuItemKind::Action { icon, .. } => {
                    if let Some(icon_str) = icon {
                        painter.draw_text(
                            icon_str,
                            Point::new(text_x, y + item_height / 2.0 + 4.0),
                            text_color,
                            14.0,
                        );
                        text_x += 24.0;
                    }
                }
                _ => {}
            }

            // Label
            painter.draw_text(
                item.label(),
                Point::new(text_x, y + item_height / 2.0 + 4.0),
                text_color,
                13.0,
            );

            // Shortcut
            if let MenuItemKind::Action { shortcut: Some(shortcut), .. } = &item.kind {
                let shortcut_x = item_rect.x() + item_rect.width() - horizontal_padding - shortcut.len() as f32 * 7.0;
                painter.draw_text(
                    shortcut,
                    Point::new(shortcut_x, y + item_height / 2.0 + 4.0),
                    theme.colors.muted_foreground,
                    11.0,
                );
            }

            // Submenu arrow
            if let MenuItemKind::Submenu { .. } = &item.kind {
                let arrow_x = item_rect.x() + item_rect.width() - horizontal_padding - 10.0;
                painter.draw_text(
                    "▶",
                    Point::new(arrow_x, y + item_height / 2.0 + 4.0),
                    text_color,
                    10.0,
                );
            }

            y += item_height;
        }
    }
}
