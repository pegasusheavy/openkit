//! List view widget for displaying lists of items.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, Key, KeyEventKind};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A list item.
#[derive(Debug, Clone)]
pub struct ListItem {
    /// Unique identifier
    pub id: String,
    /// Primary text
    pub text: String,
    /// Secondary/subtitle text
    pub subtitle: Option<String>,
    /// Icon
    pub icon: Option<String>,
    /// Whether this item is selected
    pub selected: bool,
    /// Whether this item is disabled
    pub disabled: bool,
}

impl ListItem {
    /// Create a new list item.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            subtitle: None,
            icon: None,
            selected: false,
            disabled: false,
        }
    }

    /// Set the subtitle.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Selection mode for the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// No selection allowed
    None,
    /// Single item selection (default)
    #[default]
    Single,
    /// Multiple item selection
    Multiple,
}

/// A list view widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let file_list = ListView::new()
///     .items(vec![
///         ListItem::new("doc1", "Document.pdf").icon("📄"),
///         ListItem::new("img1", "Photo.jpg").icon("🖼️").subtitle("2.5 MB"),
///         ListItem::new("folder1", "Projects").icon("📁"),
///     ])
///     .selection_mode(SelectionMode::Single)
///     .on_select(|id| println!("Selected: {}", id))
///     .on_activate(|id| println!("Activated: {}", id));
/// ```
#[allow(clippy::type_complexity)]
pub struct ListView {
    base: WidgetBase,
    items: Vec<ListItem>,
    item_height: f32,
    selection_mode: SelectionMode,
    scroll_offset: f32,
    hovered_index: Option<usize>,
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_activate: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl ListView {
    /// Create a new list view.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("list-view"),
            items: Vec::new(),
            item_height: 48.0,
            selection_mode: SelectionMode::default(),
            scroll_offset: 0.0,
            hovered_index: None,
            on_select: None,
            on_activate: None,
        }
    }

    /// Set the items.
    pub fn items(mut self, items: Vec<ListItem>) -> Self {
        self.items = items;
        self
    }

    /// Add an item.
    pub fn item(mut self, item: ListItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the item height.
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// Set the selection mode.
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
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

    /// Set the activate handler (double-click or enter).
    pub fn on_activate<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_activate = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get selected item IDs.
    pub fn selected_items(&self) -> Vec<&str> {
        self.items.iter()
            .filter(|item| item.selected)
            .map(|item| item.id.as_str())
            .collect()
    }

    /// Select an item by ID.
    pub fn select(&mut self, id: &str) {
        if self.selection_mode == SelectionMode::None {
            return;
        }

        if self.selection_mode == SelectionMode::Single {
            // Clear other selections
            for item in &mut self.items {
                item.selected = false;
            }
        }

        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if !item.disabled {
                item.selected = true;
                if let Some(handler) = &self.on_select {
                    handler(id);
                }
            }
        }
    }

    /// Toggle selection for an item.
    pub fn toggle_selection(&mut self, id: &str) {
        if self.selection_mode != SelectionMode::Multiple {
            return;
        }

        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if !item.disabled {
                item.selected = !item.selected;
                if let Some(handler) = &self.on_select {
                    handler(id);
                }
            }
        }
    }

    /// Clear all selections.
    pub fn clear_selection(&mut self) {
        for item in &mut self.items {
            item.selected = false;
        }
    }

    fn total_height(&self) -> f32 {
        self.items.len() as f32 * self.item_height
    }

    fn item_at_point(&self, point: Point) -> Option<usize> {
        if !self.bounds().contains(point) {
            return None;
        }

        let relative_y = point.y - self.base.bounds.y() + self.scroll_offset;
        let index = (relative_y / self.item_height) as usize;

        if index < self.items.len() {
            Some(index)
        } else {
            None
        }
    }
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ListView {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "list-view"
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
        Size::new(200.0, self.total_height().min(300.0))
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        painter.fill_rect(rect, theme.colors.background);

        // Calculate visible range
        let first_visible = (self.scroll_offset / self.item_height) as usize;
        let visible_count = (rect.height() / self.item_height).ceil() as usize + 1;
        let last_visible = (first_visible + visible_count).min(self.items.len());

        // Draw visible items
        for i in first_visible..last_visible {
            let item = &self.items[i];
            let item_y = rect.y() + (i as f32) * self.item_height - self.scroll_offset;
            let item_rect = Rect::new(rect.x(), item_y, rect.width(), self.item_height);

            // Skip items outside visible area
            if item_y + self.item_height < rect.y() || item_y > rect.y() + rect.height() {
                continue;
            }

            // Selection/hover background
            if item.selected {
                painter.fill_rect(item_rect, theme.colors.accent);
            } else if self.hovered_index == Some(i) && !item.disabled {
                painter.fill_rect(item_rect, theme.colors.accent.with_alpha(0.1));
            }

            let text_color = if item.disabled {
                theme.colors.muted_foreground
            } else if item.selected {
                theme.colors.accent_foreground
            } else {
                theme.colors.foreground
            };

            let mut content_x = rect.x() + 12.0;

            // Icon
            if let Some(ref icon) = item.icon {
                painter.draw_text(
                    icon,
                    Point::new(content_x, item_y + self.item_height * 0.6),
                    text_color,
                    20.0,
                );
                content_x += 32.0;
            }

            // Primary text
            let text_y = if item.subtitle.is_some() {
                item_y + self.item_height * 0.4
            } else {
                item_y + self.item_height * 0.6
            };
            painter.draw_text(&item.text, Point::new(content_x, text_y), text_color, 14.0);

            // Subtitle
            if let Some(ref subtitle) = item.subtitle {
                let subtitle_color = if item.selected {
                    theme.colors.accent_foreground.with_alpha(0.7)
                } else {
                    theme.colors.muted_foreground
                };
                painter.draw_text(
                    subtitle,
                    Point::new(content_x, item_y + self.item_height * 0.75),
                    subtitle_color,
                    12.0,
                );
            }

            // Separator line
            if i < self.items.len() - 1 {
                let sep_y = item_y + self.item_height - 1.0;
                painter.fill_rect(
                    Rect::new(rect.x() + 12.0, sep_y, rect.width() - 24.0, 1.0),
                    theme.colors.border.with_alpha(0.5),
                );
            }
        }

        // Border
        painter.stroke_rect(rect, theme.colors.border, 1.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Move => {
                        let new_hovered = self.item_at_point(mouse.position);
                        if new_hovered != self.hovered_index {
                            self.hovered_index = new_hovered;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Leave => {
                        if self.hovered_index.is_some() {
                            self.hovered_index = None;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                        if let Some(index) = self.item_at_point(mouse.position) {
                            let id = self.items[index].id.clone();
                            if self.selection_mode == SelectionMode::Multiple {
                                self.toggle_selection(&id);
                            } else {
                                self.select(&id);
                            }
                            ctx.request_focus(self.base.id);
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    // Scroll handling would go here
                    _ => {}
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Down && self.base.state.focused => {
                match key.key {
                    Key::Enter => {
                        if let Some(item) = self.items.iter().find(|i| i.selected) {
                            let id = item.id.clone();
                            if let Some(handler) = &self.on_activate {
                                handler(&id);
                            }
                            return EventResult::Handled;
                        }
                    }
                    Key::Up => {
                        if let Some(index) = self.items.iter().position(|i| i.selected) {
                            if index > 0 {
                                let id = self.items[index - 1].id.clone();
                                self.select(&id);
                                ctx.request_redraw();
                            }
                        }
                        return EventResult::Handled;
                    }
                    Key::Down => {
                        if let Some(index) = self.items.iter().position(|i| i.selected) {
                            if index + 1 < self.items.len() {
                                let id = self.items[index + 1].id.clone();
                                self.select(&id);
                                ctx.request_redraw();
                            }
                        } else if !self.items.is_empty() {
                            let id = self.items[0].id.clone();
                            self.select(&id);
                            ctx.request_redraw();
                        }
                        return EventResult::Handled;
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
