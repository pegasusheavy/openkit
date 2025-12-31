//! Tab container widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Tab position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabPosition {
    /// Tabs on top (default)
    #[default]
    Top,
    /// Tabs on bottom
    Bottom,
    /// Tabs on left
    Left,
    /// Tabs on right
    Right,
}

/// Tab variant style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabVariant {
    /// Default tab style with bottom border
    #[default]
    Default,
    /// Boxed/card style tabs
    Boxed,
    /// Pill-shaped tabs
    Pills,
}

/// A single tab.
#[derive(Debug, Clone)]
pub struct Tab {
    /// Unique identifier
    pub id: String,
    /// Tab label
    pub label: String,
    /// Optional icon
    pub icon: Option<String>,
    /// Whether this tab can be closed
    pub closeable: bool,
    /// Whether this tab is disabled
    pub disabled: bool,
}

impl Tab {
    /// Create a new tab.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            closeable: false,
            disabled: false,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set closeable.
    pub fn closeable(mut self, closeable: bool) -> Self {
        self.closeable = closeable;
        self
    }

    /// Set disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A tab container widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let tabs = Tabs::new()
///     .tab(Tab::new("general", "General").icon("⚙️"))
///     .tab(Tab::new("appearance", "Appearance").icon("🎨"))
///     .tab(Tab::new("keyboard", "Keyboard").icon("⌨️"))
///     .selected("general")
///     .on_change(|id| println!("Tab changed: {}", id));
/// ```
#[allow(clippy::type_complexity)]
pub struct Tabs {
    base: WidgetBase,
    tabs: Vec<Tab>,
    selected_id: Option<String>,
    position: TabPosition,
    variant: TabVariant,
    tab_height: f32,
    hovered_tab: Option<String>,
    hovered_close: Option<String>,
    content: Option<Box<dyn Widget>>,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_close: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl Tabs {
    /// Create a new tabs container.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("tabs"),
            tabs: Vec::new(),
            selected_id: None,
            position: TabPosition::default(),
            variant: TabVariant::default(),
            tab_height: 40.0,
            hovered_tab: None,
            hovered_close: None,
            content: None,
            on_change: None,
            on_close: None,
        }
    }

    /// Add a tab.
    pub fn tab(mut self, tab: Tab) -> Self {
        if self.selected_id.is_none() && !tab.disabled {
            self.selected_id = Some(tab.id.clone());
        }
        self.tabs.push(tab);
        self
    }

    /// Set the tabs.
    pub fn tabs(mut self, tabs: Vec<Tab>) -> Self {
        self.tabs = tabs;
        if self.selected_id.is_none() {
            self.selected_id = self.tabs.iter().find(|t| !t.disabled).map(|t| t.id.clone());
        }
        self
    }

    /// Set the selected tab.
    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    /// Set the tab position.
    pub fn position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the variant.
    pub fn variant(mut self, variant: TabVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the content for the selected tab.
    pub fn content<W: Widget + 'static>(mut self, content: W) -> Self {
        self.content = Some(Box::new(content));
        self
    }

    /// Set the change handler.
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Set the close handler.
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the selected tab ID.
    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    /// Select a tab by ID.
    pub fn select(&mut self, id: &str) {
        if let Some(tab) = self.tabs.iter().find(|t| t.id == id && !t.disabled) {
            self.selected_id = Some(tab.id.clone());
            if let Some(handler) = &self.on_change {
                handler(id);
            }
        }
    }

    fn get_tab_rect(&self, index: usize) -> Rect {
        let mut x = self.base.bounds.x();
        let y = self.base.bounds.y();

        for (i, tab) in self.tabs.iter().enumerate() {
            let width = self.calculate_tab_width(tab);
            if i == index {
                return Rect::new(x, y, width, self.tab_height);
            }
            x += width;
        }
        Rect::ZERO
    }

    fn calculate_tab_width(&self, tab: &Tab) -> f32 {
        let base_width = tab.label.len() as f32 * 8.0 + 32.0;
        let icon_width = if tab.icon.is_some() { 24.0 } else { 0.0 };
        let close_width = if tab.closeable { 24.0 } else { 0.0 };
        base_width + icon_width + close_width
    }

    fn tab_at_point(&self, point: Point) -> Option<(usize, bool)> {
        for (i, tab) in self.tabs.iter().enumerate() {
            let rect = self.get_tab_rect(i);
            if rect.contains(point) {
                // Check if clicking close button
                if tab.closeable {
                    let close_x = rect.x() + rect.width() - 24.0;
                    let in_close = point.x >= close_x;
                    return Some((i, in_close));
                }
                return Some((i, false));
            }
        }
        None
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Tabs {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "tabs"
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

    fn intrinsic_size(&self, ctx: &LayoutContext) -> Size {
        let tabs_width: f32 = self.tabs.iter().map(|t| self.calculate_tab_width(t)).sum();
        let content_size = self.content.as_ref().map(|c| c.intrinsic_size(ctx)).unwrap_or(Size::ZERO);

        Size::new(
            tabs_width.max(content_size.width),
            self.tab_height + content_size.height,
        )
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;

        // Layout content
        if let Some(content) = &mut self.content {
            let content_constraints = Constraints {
                min_width: 0.0,
                min_height: 0.0,
                max_width: size.width,
                max_height: size.height - self.tab_height,
            };
            let result = content.layout(content_constraints, ctx);
            content.set_bounds(Rect::new(
                self.base.bounds.x(),
                self.base.bounds.y() + self.tab_height,
                result.size.width,
                result.size.height,
            ));
        }

        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Tab bar background
        let tab_bar_rect = Rect::new(rect.x(), rect.y(), rect.width(), self.tab_height);
        painter.fill_rect(tab_bar_rect, theme.colors.muted.with_alpha(0.3));

        // Draw tabs
        for (i, tab) in self.tabs.iter().enumerate() {
            let tab_rect = self.get_tab_rect(i);
            let is_selected = self.selected_id.as_ref() == Some(&tab.id);
            let is_hovered = self.hovered_tab.as_ref() == Some(&tab.id);

            // Tab background
            let bg_color = match self.variant {
                TabVariant::Default => {
                    if is_selected {
                        theme.colors.background
                    } else if is_hovered && !tab.disabled {
                        theme.colors.muted.with_alpha(0.5)
                    } else {
                        Color::TRANSPARENT
                    }
                }
                TabVariant::Boxed => {
                    if is_selected {
                        theme.colors.card
                    } else if is_hovered && !tab.disabled {
                        theme.colors.muted.with_alpha(0.5)
                    } else {
                        Color::TRANSPARENT
                    }
                }
                TabVariant::Pills => {
                    if is_selected {
                        theme.colors.accent
                    } else if is_hovered && !tab.disabled {
                        theme.colors.accent.with_alpha(0.1)
                    } else {
                        Color::TRANSPARENT
                    }
                }
            };

            let radius = match self.variant {
                TabVariant::Default => BorderRadius::new(4.0, 4.0, 0.0, 0.0),
                TabVariant::Boxed => BorderRadius::new(4.0, 4.0, 0.0, 0.0),
                TabVariant::Pills => BorderRadius::all(self.tab_height / 2.0 - 4.0),
            };

            let pill_rect = if self.variant == TabVariant::Pills {
                Rect::new(tab_rect.x() + 4.0, tab_rect.y() + 4.0, tab_rect.width() - 8.0, tab_rect.height() - 8.0)
            } else {
                tab_rect
            };

            painter.fill_rounded_rect(pill_rect, bg_color, radius);

            // Bottom border for default variant
            if self.variant == TabVariant::Default && is_selected {
                painter.fill_rect(
                    Rect::new(tab_rect.x(), tab_rect.y() + tab_rect.height() - 2.0, tab_rect.width(), 2.0),
                    theme.colors.primary,
                );
            }

            let text_color = if tab.disabled {
                theme.colors.muted_foreground
            } else if is_selected && self.variant == TabVariant::Pills {
                theme.colors.accent_foreground
            } else {
                theme.colors.foreground
            };

            let mut content_x = tab_rect.x() + 12.0;

            // Icon
            if let Some(ref icon) = tab.icon {
                painter.draw_text(
                    icon,
                    Point::new(content_x, tab_rect.y() + self.tab_height * 0.65),
                    text_color,
                    16.0,
                );
                content_x += 24.0;
            }

            // Label
            painter.draw_text(
                &tab.label,
                Point::new(content_x, tab_rect.y() + self.tab_height * 0.65),
                text_color,
                14.0,
            );

            // Close button
            if tab.closeable {
                let close_x = tab_rect.x() + tab_rect.width() - 24.0;
                let close_hovered = self.hovered_close.as_ref() == Some(&tab.id);
                let close_color = if close_hovered {
                    theme.colors.destructive
                } else {
                    theme.colors.muted_foreground
                };
                painter.draw_text(
                    "✕",
                    Point::new(close_x + 4.0, tab_rect.y() + self.tab_height * 0.65),
                    close_color,
                    12.0,
                );
            }
        }

        // Tab bar bottom border
        if self.variant == TabVariant::Default {
            painter.fill_rect(
                Rect::new(rect.x(), rect.y() + self.tab_height - 1.0, rect.width(), 1.0),
                theme.colors.border,
            );
        }

        // Paint content
        if let Some(content) = &self.content {
            let content_rect = Rect::new(
                rect.x(),
                rect.y() + self.tab_height,
                rect.width(),
                rect.height() - self.tab_height,
            );
            content.paint(painter, content_rect, ctx);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Move => {
                        if let Some((index, in_close)) = self.tab_at_point(mouse.position) {
                            let tab = &self.tabs[index];
                            let new_hovered = Some(tab.id.clone());
                            let new_close = if in_close { Some(tab.id.clone()) } else { None };

                            if new_hovered != self.hovered_tab || new_close != self.hovered_close {
                                self.hovered_tab = new_hovered;
                                self.hovered_close = new_close;
                                ctx.request_redraw();
                            }
                        } else if self.hovered_tab.is_some() {
                            self.hovered_tab = None;
                            self.hovered_close = None;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Leave => {
                        if self.hovered_tab.is_some() || self.hovered_close.is_some() {
                            self.hovered_tab = None;
                            self.hovered_close = None;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Up if mouse.button == Some(MouseButton::Left) => {
                        if let Some((index, in_close)) = self.tab_at_point(mouse.position) {
                            let tab = &self.tabs[index];
                            if tab.disabled {
                                return EventResult::Ignored;
                            }

                            if in_close && tab.closeable {
                                let id = tab.id.clone();
                                if let Some(handler) = &self.on_close {
                                    handler(&id);
                                }
                            } else {
                                let id = tab.id.clone();
                                self.select(&id);
                            }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    _ => {}
                }

                // Forward to content
                if let Some(content) = &mut self.content {
                    if mouse.position.y > self.base.bounds.y() + self.tab_height {
                        return content.handle_event(event, ctx);
                    }
                }
            }
            _ => {
                if let Some(content) = &mut self.content {
                    return content.handle_event(event, ctx);
                }
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

    fn children(&self) -> &[Box<dyn Widget>] {
        if let Some(content) = &self.content {
            std::slice::from_ref(content)
        } else {
            &[]
        }
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        if let Some(content) = &mut self.content {
            std::slice::from_mut(content)
        } else {
            &mut []
        }
    }
}
