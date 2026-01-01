//! Search bar widget
//!
//! A search input with icon, clear button, and suggestion support.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Search suggestion item
#[derive(Debug, Clone)]
pub struct SearchSuggestion {
    pub id: String,
    pub text: String,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub data: Option<String>,
}

impl SearchSuggestion {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon: None,
            category: None,
            data: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }
}

/// Search bar widget
pub struct SearchBar {
    base: WidgetBase,
    query: String,
    placeholder: String,
    suggestions: Vec<SearchSuggestion>,
    suggestions_visible: bool,
    selected_index: Option<usize>,
    icon: String,
    show_clear: bool,
    width: f32,
    height: f32,
    background: Color,
    text_color: Color,
    placeholder_color: Color,
    border_radius: f32,
    is_focused: bool,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_suggestion_select: Option<Box<dyn Fn(&SearchSuggestion) + Send + Sync>>,
}

impl SearchBar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("search-bar"),
            query: String::new(),
            placeholder: String::from("Search..."),
            suggestions: Vec::new(),
            suggestions_visible: false,
            selected_index: None,
            icon: String::from("🔍"),
            show_clear: true,
            width: 300.0,
            height: 40.0,
            background: Color::rgba(1.0, 1.0, 1.0, 0.1),
            text_color: Color::WHITE,
            placeholder_color: Color::rgba(1.0, 1.0, 1.0, 0.5),
            border_radius: 8.0,
            is_focused: false,
            on_change: None,
            on_submit: None,
            on_suggestion_select: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    pub fn show_clear(mut self, show: bool) -> Self {
        self.show_clear = show;
        self
    }

    pub fn suggestions(mut self, suggestions: Vec<SearchSuggestion>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_submit = Some(Box::new(f));
        self
    }

    pub fn on_suggestion_select<F>(mut self, f: F) -> Self
    where
        F: Fn(&SearchSuggestion) + Send + Sync + 'static,
    {
        self.on_suggestion_select = Some(Box::new(f));
        self
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        if let Some(ref cb) = self.on_change {
            cb(&self.query);
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        if let Some(ref cb) = self.on_change {
            cb("");
        }
    }

    pub fn focus(&mut self) {
        self.is_focused = true;
        self.suggestions_visible = !self.suggestions.is_empty();
    }

    pub fn blur(&mut self) {
        self.is_focused = false;
        self.suggestions_visible = false;
    }

    pub fn set_suggestions(&mut self, suggestions: Vec<SearchSuggestion>) {
        self.suggestions = suggestions;
        self.suggestions_visible = self.is_focused && !self.suggestions.is_empty();
        self.selected_index = None;
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

impl Default for SearchBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SearchBar {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "search-bar"
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
        let mut height = self.height;
        if self.suggestions_visible && !self.suggestions.is_empty() {
            height += self.suggestions.len().min(5) as f32 * 40.0;
        }
        Size::new(self.width, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        // Search box background
        let search_rect = Rect::new(rect.x(), rect.y(), rect.width(), self.height);
        painter.fill_rounded_rect(search_rect, self.background, BorderRadius::all(self.border_radius));

        // Focus ring
        if self.is_focused {
            painter.stroke_rect(search_rect, Color::rgba(0.3, 0.5, 1.0, 0.5), 2.0);
        }

        // Icon
        painter.draw_text(&self.icon, Point::new(rect.x() + 12.0, rect.y() + self.height / 2.0 + 4.0), self.text_color, 16.0);

        // Text or placeholder
        let text = if self.query.is_empty() {
            &self.placeholder
        } else {
            &self.query
        };
        let text_color = if self.query.is_empty() {
            self.placeholder_color
        } else {
            self.text_color
        };
        painter.draw_text(text, Point::new(rect.x() + 40.0, rect.y() + self.height / 2.0 + 5.0), text_color, 14.0);

        // Suggestion dropdown
        if self.suggestions_visible && !self.suggestions.is_empty() {
            let dropdown_rect = Rect::new(
                rect.x(),
                rect.y() + self.height + 4.0,
                rect.width(),
                self.suggestions.len().min(5) as f32 * 40.0,
            );
            painter.fill_rounded_rect(dropdown_rect, Color::rgba(0.15, 0.15, 0.15, 0.95), BorderRadius::all(self.border_radius));

            for (i, suggestion) in self.suggestions.iter().take(5).enumerate() {
                let item_rect = Rect::new(
                    dropdown_rect.x(),
                    dropdown_rect.y() + i as f32 * 40.0,
                    dropdown_rect.width(),
                    40.0,
                );
                if self.selected_index == Some(i) {
                    painter.fill_rect(item_rect, Color::rgba(1.0, 1.0, 1.0, 0.1));
                }
                painter.draw_text(
                    &suggestion.text,
                    Point::new(item_rect.x() + 16.0, item_rect.y() + 25.0),
                    Color::WHITE,
                    14.0,
                );
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
