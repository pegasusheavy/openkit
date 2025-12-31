//! Context menu (right-click menu) widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, Key, KeyEventKind};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A menu item in a context menu.
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Optional icon
    pub icon: Option<String>,
    /// Keyboard shortcut text
    pub shortcut: Option<String>,
    /// Whether this item is disabled
    pub disabled: bool,
    /// Whether this is a separator
    pub separator: bool,
    /// Submenu items (if any)
    pub submenu: Option<Vec<MenuItem>>,
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            disabled: false,
            separator: false,
            submenu: None,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            shortcut: None,
            disabled: true,
            separator: true,
            submenu: None,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the keyboard shortcut display text.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set submenu items.
    pub fn submenu(mut self, items: Vec<MenuItem>) -> Self {
        self.submenu = Some(items);
        self
    }
}

/// A context menu widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let menu = ContextMenu::new()
///     .item(MenuItem::new("cut", "Cut").icon("✂️").shortcut("Ctrl+X"))
///     .item(MenuItem::new("copy", "Copy").icon("📋").shortcut("Ctrl+C"))
///     .item(MenuItem::new("paste", "Paste").icon("📄").shortcut("Ctrl+V"))
///     .item(MenuItem::separator())
///     .item(MenuItem::new("delete", "Delete").icon("🗑️"))
///     .on_select(|id| println!("Selected: {}", id));
/// ```
#[allow(clippy::type_complexity)]
pub struct ContextMenu {
    base: WidgetBase,
    items: Vec<MenuItem>,
    position: Point,
    visible: bool,
    hovered_index: Option<usize>,
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_close: Option<Box<dyn Fn() + Send + Sync>>,
}

impl ContextMenu {
    /// Create a new context menu.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("context-menu"),
            items: Vec::new(),
            position: Point::ZERO,
            visible: false,
            hovered_index: None,
            on_select: None,
            on_close: None,
        }
    }

    /// Add a menu item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set all items.
    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self
    }

    /// Set the select handler.
    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Set the close handler.
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Show the menu at a position.
    pub fn show_at(&mut self, position: Point) {
        self.position = position;
        self.visible = true;
        self.hovered_index = None;
    }

    /// Hide the menu.
    pub fn hide(&mut self) {
        self.visible = false;
        if let Some(handler) = &self.on_close {
            handler();
        }
    }

    /// Check if the menu is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    fn item_height() -> f32 {
        32.0
    }

    fn separator_height() -> f32 {
        9.0
    }

    fn menu_width() -> f32 {
        200.0
    }

    fn calculate_height(&self) -> f32 {
        self.items.iter().map(|item| {
            if item.separator {
                Self::separator_height()
            } else {
                Self::item_height()
            }
        }).sum()
    }

    #[allow(dead_code)]
    fn get_item_rect(&self, index: usize) -> Rect {
        let mut y = self.position.y;
        for (i, item) in self.items.iter().enumerate() {
            let height = if item.separator {
                Self::separator_height()
            } else {
                Self::item_height()
            };

            if i == index {
                return Rect::new(self.position.x, y, Self::menu_width(), height);
            }
            y += height;
        }
        Rect::ZERO
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ContextMenu {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "context-menu"
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
        if self.visible {
            Size::new(Self::menu_width(), self.calculate_height())
        } else {
            Size::ZERO
        }
    }

    fn layout(&mut self, _constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        let size = if self.visible {
            Size::new(Self::menu_width(), self.calculate_height())
        } else {
            Size::ZERO
        };
        self.base.bounds = Rect::new(self.position.x, self.position.y, size.width, size.height);
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, _rect: Rect, ctx: &PaintContext) {
        if !self.visible {
            return;
        }

        let theme = ctx.style_ctx.theme;
        let menu_rect = Rect::new(
            self.position.x,
            self.position.y,
            Self::menu_width(),
            self.calculate_height(),
        );
        let radius = BorderRadius::all(theme.radii.md * theme.typography.base_size);

        // Shadow
        let shadow_rect = Rect::new(
            menu_rect.x() + 2.0,
            menu_rect.y() + 4.0,
            menu_rect.width(),
            menu_rect.height(),
        );
        painter.fill_rounded_rect(shadow_rect, Color::BLACK.with_alpha(0.2), radius);

        // Background
        painter.fill_rounded_rect(menu_rect, theme.colors.popover, radius);
        painter.stroke_rect(menu_rect, theme.colors.border, 1.0);

        // Items
        let mut y = self.position.y;
        for (i, item) in self.items.iter().enumerate() {
            if item.separator {
                // Draw separator line
                let sep_y = y + Self::separator_height() / 2.0;
                painter.fill_rect(
                    Rect::new(self.position.x + 8.0, sep_y, Self::menu_width() - 16.0, 1.0),
                    theme.colors.border,
                );
                y += Self::separator_height();
                continue;
            }

            let item_rect = Rect::new(self.position.x, y, Self::menu_width(), Self::item_height());

            // Hover background
            if self.hovered_index == Some(i) && !item.disabled {
                painter.fill_rect(item_rect, theme.colors.accent);
            }

            let text_color = if item.disabled {
                theme.colors.muted_foreground
            } else if self.hovered_index == Some(i) {
                theme.colors.accent_foreground
            } else {
                theme.colors.popover_foreground
            };

            // Icon
            let mut text_x = self.position.x + 12.0;
            if let Some(ref icon) = item.icon {
                painter.draw_text(
                    icon,
                    Point::new(text_x, y + 22.0),
                    text_color,
                    14.0,
                );
                text_x += 24.0;
            }

            // Label
            painter.draw_text(
                &item.label,
                Point::new(text_x, y + 22.0),
                text_color,
                14.0,
            );

            // Shortcut
            if let Some(ref shortcut) = item.shortcut {
                let shortcut_x = self.position.x + Self::menu_width() - 12.0 - shortcut.len() as f32 * 7.0;
                painter.draw_text(
                    shortcut,
                    Point::new(shortcut_x, y + 22.0),
                    theme.colors.muted_foreground,
                    12.0,
                );
            }

            // Submenu arrow
            if item.submenu.is_some() {
                painter.draw_text(
                    "▶",
                    Point::new(self.position.x + Self::menu_width() - 20.0, y + 22.0),
                    text_color,
                    12.0,
                );
            }

            y += Self::item_height();
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if !self.visible {
            return EventResult::Ignored;
        }

        match event {
            Event::Mouse(mouse) => {
                let menu_rect = Rect::new(
                    self.position.x,
                    self.position.y,
                    Self::menu_width(),
                    self.calculate_height(),
                );

                match mouse.kind {
                    MouseEventKind::Move => {
                        if menu_rect.contains(mouse.position) {
                            // Find which item is hovered
                            let mut y = self.position.y;
                            for (i, item) in self.items.iter().enumerate() {
                                let height = if item.separator {
                                    Self::separator_height()
                                } else {
                                    Self::item_height()
                                };

                                if mouse.position.y >= y && mouse.position.y < y + height {
                                    if !item.separator && self.hovered_index != Some(i) {
                                        self.hovered_index = Some(i);
                                        ctx.request_redraw();
                                    }
                                    break;
                                }
                                y += height;
                            }
                        } else if self.hovered_index.is_some() {
                            self.hovered_index = None;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                        if !menu_rect.contains(mouse.position) {
                            self.hide();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    MouseEventKind::Up if mouse.button == Some(MouseButton::Left) => {
                        if let Some(index) = self.hovered_index {
                            if let Some(item) = self.items.get(index) {
                                if !item.disabled && !item.separator {
                                    let id = item.id.clone();
                                    self.hide();
                                    if let Some(handler) = &self.on_select {
                                        handler(&id);
                                    }
                                    ctx.request_redraw();
                                    return EventResult::Handled;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Down => {
                match key.key {
                    Key::Escape => {
                        self.hide();
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
                    Key::Up => {
                        let current = self.hovered_index.unwrap_or(0);
                        if current > 0 {
                            // Skip separators
                            let mut new_index = current - 1;
                            while new_index > 0 && self.items.get(new_index).map(|i| i.separator).unwrap_or(false) {
                                new_index -= 1;
                            }
                            self.hovered_index = Some(new_index);
                            ctx.request_redraw();
                        }
                        return EventResult::Handled;
                    }
                    Key::Down => {
                        let current = self.hovered_index.unwrap_or(0);
                        if current + 1 < self.items.len() {
                            // Skip separators
                            let mut new_index = current + 1;
                            while new_index + 1 < self.items.len() && self.items.get(new_index).map(|i| i.separator).unwrap_or(false) {
                                new_index += 1;
                            }
                            self.hovered_index = Some(new_index);
                            ctx.request_redraw();
                        }
                        return EventResult::Handled;
                    }
                    Key::Enter => {
                        if let Some(index) = self.hovered_index {
                            if let Some(item) = self.items.get(index) {
                                if !item.disabled && !item.separator {
                                    let id = item.id.clone();
                                    self.hide();
                                    if let Some(handler) = &self.on_select {
                                        handler(&id);
                                    }
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
