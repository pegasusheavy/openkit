//! Dropdown/Select widget for choosing from a list of options.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, Key, KeyEventKind};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// An option in a dropdown.
#[derive(Debug, Clone)]
pub struct DropdownOption {
    /// Unique value/key
    pub value: String,
    /// Display label
    pub label: String,
    /// Optional icon name
    pub icon: Option<String>,
    /// Whether this option is disabled
    pub disabled: bool,
}

impl DropdownOption {
    /// Create a new option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

    /// Set an icon.
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

/// A dropdown select widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let dropdown = Dropdown::new()
///     .placeholder("Select a session...")
///     .options(vec![
///         DropdownOption::new("gnome", "GNOME"),
///         DropdownOption::new("plasma", "KDE Plasma"),
///         DropdownOption::new("xfce", "Xfce"),
///     ])
///     .on_change(|value| {
///         println!("Selected: {}", value);
///     });
/// ```
#[allow(clippy::type_complexity)]
pub struct Dropdown {
    base: WidgetBase,
    options: Vec<DropdownOption>,
    selected_value: Option<String>,
    placeholder: String,
    is_open: bool,
    hovered_index: Option<usize>,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl Dropdown {
    /// Create a new dropdown.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("dropdown"),
            options: Vec::new(),
            selected_value: None,
            placeholder: "Select...".to_string(),
            is_open: false,
            hovered_index: None,
            on_change: None,
        }
    }

    /// Set the options.
    pub fn options(mut self, options: Vec<DropdownOption>) -> Self {
        self.options = options;
        self
    }

    /// Add a single option.
    pub fn option(mut self, option: DropdownOption) -> Self {
        self.options.push(option);
        self
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the initially selected value.
    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected_value = Some(value.into());
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

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the currently selected value.
    pub fn value(&self) -> Option<&str> {
        self.selected_value.as_deref()
    }

    /// Get the selected option.
    pub fn selected_option(&self) -> Option<&DropdownOption> {
        self.selected_value.as_ref().and_then(|v| {
            self.options.iter().find(|o| &o.value == v)
        })
    }

    /// Open the dropdown.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Close the dropdown.
    pub fn close(&mut self) {
        self.is_open = false;
        self.hovered_index = None;
    }

    /// Toggle the dropdown.
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Select an option by value.
    pub fn select(&mut self, value: &str) {
        let found = self.options.iter()
            .find(|o| o.value == value && !o.disabled)
            .map(|o| o.value.clone());

        if let Some(selected_value) = found {
            self.selected_value = Some(selected_value.clone());
            self.close();
            if let Some(handler) = &self.on_change {
                handler(&selected_value);
            }
        }
    }

    fn option_height() -> f32 {
        36.0
    }

    fn dropdown_max_height(&self) -> f32 {
        // Show at most 6 items
        (self.options.len().min(6) as f32) * Self::option_height()
    }
}

impl Default for Dropdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Dropdown {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "dropdown"
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
        Size::new(200.0, 40.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let intrinsic = self.intrinsic_size(ctx);
        let size = Size::new(
            constraints.max_width.min(intrinsic.width.max(constraints.min_width)),
            intrinsic.height,
        );
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let radius = BorderRadius::all(theme.radii.md * theme.typography.base_size);

        // Main button background
        let bg_color = if self.base.state.disabled {
            theme.colors.muted
        } else if self.is_open || self.base.state.pressed {
            theme.colors.accent
        } else if self.base.state.hovered {
            theme.colors.secondary
        } else {
            theme.colors.background
        };
        painter.fill_rounded_rect(rect, bg_color, radius);

        // Border
        let border_color = if self.base.state.focused || self.is_open {
            theme.colors.ring
        } else {
            theme.colors.border
        };
        painter.stroke_rect(rect, border_color, 1.0);

        // Selected text or placeholder
        let font_size = 14.0;
        let padding = 12.0;
        let text_x = rect.x() + padding;
        let text_y = rect.y() + (rect.height() + font_size * 0.8) / 2.0;

        let (text, text_color) = if let Some(option) = self.selected_option() {
            (&option.label, theme.colors.foreground)
        } else {
            (&self.placeholder, theme.colors.muted_foreground)
        };

        painter.draw_text(text, Point::new(text_x, text_y), text_color, font_size);

        // Chevron icon
        let chevron_x = rect.x() + rect.width() - 24.0;
        let chevron_y = rect.y() + rect.height() / 2.0;

        // Draw simple chevron (▼ or ▲)
        let chevron = if self.is_open { "▲" } else { "▼" };
        painter.draw_text(
            chevron,
            Point::new(chevron_x, chevron_y + 4.0),
            theme.colors.muted_foreground,
            12.0,
        );

        // Draw dropdown menu if open
        if self.is_open && !self.options.is_empty() {
            let menu_y = rect.y() + rect.height() + 4.0;
            let menu_height = self.dropdown_max_height();
            let menu_rect = Rect::new(rect.x(), menu_y, rect.width(), menu_height);

            // Menu background
            painter.fill_rounded_rect(menu_rect, theme.colors.popover, radius);
            painter.stroke_rect(menu_rect, theme.colors.border, 1.0);

            // Draw options
            let option_height = Self::option_height();
            for (i, option) in self.options.iter().enumerate() {
                let option_y = menu_y + (i as f32) * option_height;
                let option_rect = Rect::new(rect.x(), option_y, rect.width(), option_height);

                // Highlight hovered option
                let is_selected = self.selected_value.as_ref() == Some(&option.value);
                let is_hovered = self.hovered_index == Some(i);

                if is_selected {
                    painter.fill_rect(option_rect, theme.colors.accent);
                } else if is_hovered && !option.disabled {
                    painter.fill_rect(option_rect, theme.colors.accent.with_alpha(0.5));
                }

                // Option text
                let text_color = if option.disabled {
                    theme.colors.muted_foreground
                } else if is_selected {
                    theme.colors.accent_foreground
                } else {
                    theme.colors.popover_foreground
                };

                painter.draw_text(
                    &option.label,
                    Point::new(rect.x() + padding, option_y + (option_height + font_size * 0.8) / 2.0),
                    text_color,
                    font_size,
                );

                // Check mark for selected
                if is_selected {
                    painter.draw_text(
                        "✓",
                        Point::new(rect.x() + rect.width() - 28.0, option_y + (option_height + font_size * 0.8) / 2.0),
                        theme.colors.accent_foreground,
                        font_size,
                    );
                }
            }
        }

        // Focus ring
        if self.base.state.focused && ctx.focus_visible && !self.is_open {
            let ring_rect = Rect::new(
                rect.x() - 2.0,
                rect.y() - 2.0,
                rect.width() + 4.0,
                rect.height() + 4.0,
            );
            painter.stroke_rect(ring_rect, theme.colors.ring, 2.0);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
                let in_button = self.bounds().contains(mouse.position);

                // Check if in dropdown menu
                let in_menu = if self.is_open {
                    let menu_y = self.bounds().y() + self.bounds().height() + 4.0;
                    let menu_rect = Rect::new(
                        self.bounds().x(),
                        menu_y,
                        self.bounds().width(),
                        self.dropdown_max_height(),
                    );
                    menu_rect.contains(mouse.position)
                } else {
                    false
                };

                match mouse.kind {
                    MouseEventKind::Move | MouseEventKind::Enter => {
                        if in_button && !self.base.state.hovered {
                            self.base.state.hovered = true;
                            ctx.request_redraw();
                        } else if !in_button && !in_menu && self.base.state.hovered {
                            self.base.state.hovered = false;
                            ctx.request_redraw();
                        }

                        // Track hovered option in menu
                        if in_menu {
                            let menu_y = self.bounds().y() + self.bounds().height() + 4.0;
                            let relative_y = mouse.position.y - menu_y;
                            let index = (relative_y / Self::option_height()) as usize;
                            if index < self.options.len() && self.hovered_index != Some(index) {
                                self.hovered_index = Some(index);
                                ctx.request_redraw();
                            }
                        } else if self.hovered_index.is_some() {
                            self.hovered_index = None;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Leave => {
                        self.base.state.hovered = false;
                        self.hovered_index = None;
                        ctx.request_redraw();
                    }
                    MouseEventKind::Down if in_button => {
                        self.base.state.pressed = true;
                        ctx.request_focus(self.base.id);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
                    MouseEventKind::Up => {
                        if self.base.state.pressed && in_button {
                            self.base.state.pressed = false;
                            self.toggle();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        } else if in_menu {
                            // Select clicked option
                            let menu_y = self.bounds().y() + self.bounds().height() + 4.0;
                            let relative_y = mouse.position.y - menu_y;
                            let index = (relative_y / Self::option_height()) as usize;
                            if index < self.options.len() {
                                let option = &self.options[index];
                                if !option.disabled {
                                    self.select(&option.value.clone());
                                    ctx.request_redraw();
                                    return EventResult::Handled;
                                }
                            }
                        } else if self.is_open {
                            // Click outside closes dropdown
                            self.close();
                            ctx.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            Event::Key(key) if self.base.state.focused => {
                if key.kind == KeyEventKind::Down {
                    match key.key {
                        Key::Enter | Key::Space => {
                            if self.is_open {
                                if let Some(idx) = self.hovered_index {
                                    if idx < self.options.len() {
                                        let value = self.options[idx].value.clone();
                                        self.select(&value);
                                    }
                                }
                            } else {
                                self.open();
                            }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Escape => {
                            self.close();
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                        Key::Up => {
                            if self.is_open {
                                let current = self.hovered_index.unwrap_or(0);
                                if current > 0 {
                                    self.hovered_index = Some(current - 1);
                                    ctx.request_redraw();
                                }
                            }
                            return EventResult::Handled;
                        }
                        Key::Down => {
                            if self.is_open {
                                let current = self.hovered_index.unwrap_or(0);
                                if current + 1 < self.options.len() {
                                    self.hovered_index = Some(current + 1);
                                    ctx.request_redraw();
                                }
                            } else {
                                self.open();
                                self.hovered_index = Some(0);
                                ctx.request_redraw();
                            }
                            return EventResult::Handled;
                        }
                        _ => {}
                    }
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
