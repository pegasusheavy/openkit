//! System tray widget for status icons.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A system tray icon.
#[derive(Debug, Clone)]
pub struct TrayIcon {
    /// Unique identifier
    pub id: String,
    /// Icon (emoji or icon name)
    pub icon: String,
    /// Tooltip text
    pub tooltip: Option<String>,
    /// Whether this icon has a notification badge
    pub has_badge: bool,
    /// Badge count (if any)
    pub badge_count: Option<u32>,
}

impl TrayIcon {
    /// Create a new tray icon.
    pub fn new(id: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            tooltip: None,
            has_badge: false,
            badge_count: None,
        }
    }

    /// Set the tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set badge.
    pub fn badge(mut self, count: Option<u32>) -> Self {
        self.has_badge = count.is_some();
        self.badge_count = count;
        self
    }
}

/// A system tray widget containing status icons.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let tray = SystemTray::new()
///     .icon(TrayIcon::new("volume", "🔊").tooltip("Volume: 75%"))
///     .icon(TrayIcon::new("wifi", "📶").tooltip("Connected to WiFi"))
///     .icon(TrayIcon::new("battery", "🔋").tooltip("Battery: 85%"))
///     .icon(TrayIcon::new("notifications", "🔔").badge(Some(3)))
///     .on_click(|id| println!("Clicked: {}", id))
///     .on_right_click(|id| println!("Right-clicked: {}", id));
/// ```
#[allow(clippy::type_complexity)]
pub struct SystemTray {
    base: WidgetBase,
    icons: Vec<TrayIcon>,
    icon_size: f32,
    icon_spacing: f32,
    hovered_icon: Option<String>,
    on_click: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_right_click: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl SystemTray {
    /// Create a new system tray.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("system-tray"),
            icons: Vec::new(),
            icon_size: 20.0,
            icon_spacing: 8.0,
            hovered_icon: None,
            on_click: None,
            on_right_click: None,
        }
    }

    /// Add an icon.
    pub fn icon(mut self, icon: TrayIcon) -> Self {
        self.icons.push(icon);
        self
    }

    /// Set all icons.
    pub fn icons(mut self, icons: Vec<TrayIcon>) -> Self {
        self.icons = icons;
        self
    }

    /// Set the icon size.
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    /// Set the spacing between icons.
    pub fn icon_spacing(mut self, spacing: f32) -> Self {
        self.icon_spacing = spacing;
        self
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set the right-click handler.
    pub fn on_right_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_right_click = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Update an icon by ID.
    pub fn update_icon<F>(&mut self, id: &str, f: F)
    where
        F: FnOnce(&mut TrayIcon),
    {
        if let Some(icon) = self.icons.iter_mut().find(|i| i.id == id) {
            f(icon);
        }
    }

    fn get_icon_rect(&self, index: usize) -> Rect {
        let cell_size = self.icon_size + self.icon_spacing;
        let x = self.base.bounds.x() + (index as f32) * cell_size;
        let y = self.base.bounds.y() + (self.base.bounds.height() - self.icon_size) / 2.0;
        Rect::new(x, y, self.icon_size, self.icon_size)
    }

    fn icon_at_point(&self, point: Point) -> Option<usize> {
        for (i, _) in self.icons.iter().enumerate() {
            let rect = self.get_icon_rect(i);
            // Expand hit area slightly
            let hit_rect = Rect::new(
                rect.x() - self.icon_spacing / 2.0,
                rect.y() - 4.0,
                rect.width() + self.icon_spacing,
                rect.height() + 8.0,
            );
            if hit_rect.contains(point) {
                return Some(i);
            }
        }
        None
    }
}

impl Default for SystemTray {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SystemTray {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "system-tray"
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
        let width = self.icons.len() as f32 * (self.icon_size + self.icon_spacing) - self.icon_spacing;
        Size::new(width.max(0.0), self.icon_size + 8.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, _rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        for (i, icon) in self.icons.iter().enumerate() {
            let icon_rect = self.get_icon_rect(i);
            let is_hovered = self.hovered_icon.as_ref() == Some(&icon.id);

            // Hover background
            if is_hovered {
                let hover_rect = Rect::new(
                    icon_rect.x() - 4.0,
                    icon_rect.y() - 4.0,
                    icon_rect.width() + 8.0,
                    icon_rect.height() + 8.0,
                );
                painter.fill_rect(hover_rect, theme.colors.accent.with_alpha(0.2));
            }

            // Icon
            painter.draw_text(
                &icon.icon,
                Point::new(icon_rect.x(), icon_rect.y() + self.icon_size * 0.85),
                theme.colors.foreground,
                self.icon_size,
            );

            // Badge
            if icon.has_badge {
                let badge_size = 12.0;
                let badge_x = icon_rect.x() + icon_rect.width() - badge_size / 2.0;
                let badge_y = icon_rect.y() - badge_size / 4.0;

                painter.fill_rect(
                    Rect::new(badge_x, badge_y, badge_size, badge_size),
                    theme.colors.destructive,
                );

                if let Some(count) = icon.badge_count {
                    let count_str = if count > 9 { "9+".to_string() } else { count.to_string() };
                    painter.draw_text(
                        &count_str,
                        Point::new(badge_x + 2.0, badge_y + badge_size * 0.8),
                        theme.colors.destructive_foreground,
                        8.0,
                    );
                }
            }
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Move => {
                    let new_hovered = self.icon_at_point(mouse.position)
                        .map(|i| self.icons[i].id.clone());

                    if new_hovered != self.hovered_icon {
                        self.hovered_icon = new_hovered;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Leave => {
                    if self.hovered_icon.is_some() {
                        self.hovered_icon = None;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) => {
                    if let Some(index) = self.icon_at_point(mouse.position) {
                        let id = self.icons[index].id.clone();
                        if let Some(handler) = &self.on_click {
                            handler(&id);
                        }
                        return EventResult::Handled;
                    }
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Right) => {
                    if let Some(index) = self.icon_at_point(mouse.position) {
                        let id = self.icons[index].id.clone();
                        if let Some(handler) = &self.on_right_click {
                            handler(&id);
                        }
                        return EventResult::Handled;
                    }
                }
                _ => {}
            }
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
