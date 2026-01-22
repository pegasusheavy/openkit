//! App grid widget
//!
//! A grid of app icons for launchers and menus.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// An app in the grid
#[derive(Debug, Clone)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub exec: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub is_favorite: bool,
    pub usage_count: u32,
}

impl AppItem {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: None,
            description: None,
            exec: None,
            categories: Vec::new(),
            keywords: Vec::new(),
            is_favorite: false,
            usage_count: 0,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn exec(mut self, cmd: impl Into<String>) -> Self {
        self.exec = Some(cmd.into());
        self
    }

    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.categories.push(cat.into());
        self
    }

    pub fn keyword(mut self, kw: impl Into<String>) -> Self {
        self.keywords.push(kw.into());
        self
    }

    pub fn favorite(mut self, is_fav: bool) -> Self {
        self.is_favorite = is_fav;
        self
    }

    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query)
            || self.description.as_ref().is_some_and(|d| d.to_lowercase().contains(&query))
            || self.keywords.iter().any(|k| k.to_lowercase().contains(&query))
    }
}

/// Grid layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridLayout {
    #[default]
    Fixed,
    AutoFit,
    List,
}

/// App grid widget
pub struct AppGrid {
    base: WidgetBase,
    apps: Vec<AppItem>,
    selected_index: Option<usize>,
    hovered_index: Option<usize>,
    layout: GridLayout,
    columns: usize,
    icon_size: f32,
    cell_size: Size,
    spacing: f32,
    show_labels: bool,
    show_descriptions: bool,
    filter: String,
    #[allow(clippy::type_complexity)]
    on_app_click: Option<Box<dyn Fn(&AppItem) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_app_right_click: Option<Box<dyn Fn(&AppItem) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_drag_start: Option<Box<dyn Fn(&AppItem) + Send + Sync>>,
}

impl AppGrid {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("app-grid"),
            apps: Vec::new(),
            selected_index: None,
            hovered_index: None,
            layout: GridLayout::Fixed,
            columns: 4,
            icon_size: 48.0,
            cell_size: Size::new(96.0, 96.0),
            spacing: 8.0,
            show_labels: true,
            show_descriptions: true,
            filter: String::new(),
            on_app_click: None,
            on_app_right_click: None,
            on_drag_start: None,
        }
    }

    pub fn apps(mut self, apps: Vec<AppItem>) -> Self {
        self.apps = apps;
        self
    }

    pub fn app(mut self, app: AppItem) -> Self {
        self.apps.push(app);
        self
    }

    pub fn layout(mut self, layout: GridLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = cols.max(1);
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn cell_size(mut self, size: Size) -> Self {
        self.cell_size = size;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn show_descriptions(mut self, show: bool) -> Self {
        self.show_descriptions = show;
        self
    }

    pub fn filter(mut self, query: impl Into<String>) -> Self {
        self.filter = query.into();
        self
    }

    pub fn on_app_click<F>(mut self, f: F) -> Self
    where
        F: Fn(&AppItem) + Send + Sync + 'static,
    {
        self.on_app_click = Some(Box::new(f));
        self
    }

    pub fn on_app_right_click<F>(mut self, f: F) -> Self
    where
        F: Fn(&AppItem) + Send + Sync + 'static,
    {
        self.on_app_right_click = Some(Box::new(f));
        self
    }

    pub fn on_drag_start<F>(mut self, f: F) -> Self
    where
        F: Fn(&AppItem) + Send + Sync + 'static,
    {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    pub fn filtered_apps(&self) -> Vec<&AppItem> {
        if self.filter.is_empty() {
            self.apps.iter().collect()
        } else {
            self.apps.iter().filter(|a| a.matches(&self.filter)).collect()
        }
    }

    pub fn set_filter(&mut self, query: &str) {
        self.filter = query.to_string();
        self.selected_index = None;
    }

    pub fn select_next(&mut self) {
        let apps = self.filtered_apps();
        if apps.is_empty() { return; }
        self.selected_index = Some(match self.selected_index {
            Some(i) => (i + 1) % apps.len(),
            None => 0,
        });
    }

    pub fn select_prev(&mut self) {
        let apps = self.filtered_apps();
        if apps.is_empty() { return; }
        self.selected_index = Some(match self.selected_index {
            Some(0) | None => apps.len() - 1,
            Some(i) => i - 1,
        });
    }

    pub fn selected_app(&self) -> Option<&AppItem> {
        self.selected_index.and_then(|i| self.filtered_apps().get(i).copied())
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

impl Default for AppGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for AppGrid {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "app-grid"
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
        let apps = self.filtered_apps();
        let rows = (apps.len() as f32 / self.columns as f32).ceil() as usize;
        let width = self.columns as f32 * (self.cell_size.width + self.spacing) - self.spacing;
        let height = match self.layout {
            GridLayout::List => apps.len() as f32 * 48.0,
            _ => rows as f32 * (self.cell_size.height + self.spacing) - self.spacing,
        };
        Size::new(width.max(0.0), height.max(0.0))
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        let apps = self.filtered_apps();

        for (i, app) in apps.iter().enumerate() {
            let (row, col) = (i / self.columns, i % self.columns);

            let cell_rect = match self.layout {
                GridLayout::List => Rect::new(rect.x(), rect.y() + i as f32 * 48.0, rect.width(), 48.0),
                _ => Rect::new(
                    rect.x() + col as f32 * (self.cell_size.width + self.spacing),
                    rect.y() + row as f32 * (self.cell_size.height + self.spacing),
                    self.cell_size.width,
                    self.cell_size.height,
                ),
            };

            let is_selected = self.selected_index == Some(i);
            let is_hovered = self.hovered_index == Some(i);

            if is_selected {
                painter.fill_rounded_rect(cell_rect, Color::rgba(0.0, 0.47, 0.84, 0.3), BorderRadius::all(8.0));
            } else if is_hovered {
                painter.fill_rounded_rect(cell_rect, Color::rgba(1.0, 1.0, 1.0, 0.1), BorderRadius::all(8.0));
            }

            // Icon placeholder
            let icon_rect = match self.layout {
                GridLayout::List => Rect::new(cell_rect.x() + 8.0, cell_rect.y() + 8.0, 32.0, 32.0),
                _ => Rect::new(
                    cell_rect.x() + (cell_rect.width() - self.icon_size) / 2.0,
                    cell_rect.y() + 8.0,
                    self.icon_size,
                    self.icon_size,
                ),
            };
            painter.fill_rounded_rect(icon_rect, Color::rgba(1.0, 1.0, 1.0, 0.2), BorderRadius::all(8.0));

            // App name
            if self.show_labels {
                let text_x = match self.layout {
                    GridLayout::List => cell_rect.x() + 48.0,
                    _ => cell_rect.x() + 4.0,
                };
                let text_y = match self.layout {
                    GridLayout::List => cell_rect.y() + 28.0,
                    _ => cell_rect.y() + self.icon_size + 24.0,
                };
                painter.draw_text(&app.name, Point::new(text_x, text_y), Color::WHITE, 12.0);
            }
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
