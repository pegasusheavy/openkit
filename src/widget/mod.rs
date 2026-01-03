//! Widget system for OpenKit.

pub mod avatar;
pub mod bar;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod clock;
pub mod container;
pub mod context_menu;
pub mod desktop;
pub mod dropdown;
pub mod icon_button;
pub mod label;
pub mod list_view;
pub mod notification;
pub mod password_field;
pub mod progress;
pub mod scroll_view;
pub mod separator;
pub mod slider;
pub mod spacer;
pub mod spinner;
pub mod switch;
pub mod system_tray;
pub mod tabs;
pub mod textfield;
pub mod tooltip;
pub mod window;
pub mod workspace;

// Desktop shell widgets
pub mod action_center;
pub mod app_grid;
pub mod battery_indicator;
pub mod calendar;
pub mod chat_window;
pub mod glass_pane;
pub mod network_indicator;
pub mod panel;
pub mod search_bar;
pub mod start_button;
pub mod start_menu;
pub mod taskbar_button;
pub mod volume_control;

// Browser chrome widgets
pub mod address_bar;
pub mod bookmark_bar;
pub mod browser_tab;
pub mod browser_toolbar;
pub mod download_item;
pub mod find_bar;
pub mod navigation_bar;

// VPN widgets
pub mod vpn_status;
pub mod server_list;
pub mod connection_card;

use crate::css::{ClassList, ComputedStyle, StyleContext, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Type alias for event callback handlers to reduce type complexity.
pub type EventCallback<T = ()> = Option<Box<dyn Fn() -> T + Send + Sync>>;

/// Type alias for event handlers that take a string parameter.
pub type StringCallback = Option<Box<dyn Fn(&str) + Send + Sync>>;

use std::sync::atomic::{AtomicU64, Ordering};

/// Unique widget identifier.
pub type WidgetId = u64;

/// Global widget ID counter.
static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique widget ID.
pub fn next_widget_id() -> WidgetId {
    WIDGET_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Base widget trait.
pub trait Widget {
    /// Get the widget's unique ID.
    fn id(&self) -> WidgetId;

    /// Get the widget type name (for CSS type selectors).
    fn type_name(&self) -> &'static str;

    /// Get the widget's element ID (for CSS #id selectors).
    fn element_id(&self) -> Option<&str> {
        None
    }

    /// Get the widget's CSS classes.
    fn classes(&self) -> &ClassList;

    /// Get the widget's current state.
    fn state(&self) -> WidgetState;

    /// Calculate the intrinsic size of this widget.
    fn intrinsic_size(&self, ctx: &LayoutContext) -> Size;

    /// Perform layout with constraints.
    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult;

    /// Paint the widget.
    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext);

    /// Handle an event.
    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        EventResult::Ignored
    }

    /// Get the computed style for this widget.
    fn style(&self, _ctx: &StyleContext) -> ComputedStyle {
        ComputedStyle::default()
    }

    /// Get the bounds of this widget after layout.
    fn bounds(&self) -> Rect;

    /// Set the bounds of this widget.
    fn set_bounds(&mut self, bounds: Rect);

    /// Check if this widget contains a point.
    fn hit_test(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }

    /// Get children (for container widgets).
    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    /// Get mutable children.
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut []
    }
}

/// Context for layout operations.
pub struct LayoutContext<'a> {
    pub style_ctx: &'a StyleContext<'a>,
    pub scale_factor: f32,
}

impl<'a> LayoutContext<'a> {
    pub fn new(style_ctx: &'a StyleContext<'a>) -> Self {
        Self {
            style_ctx,
            scale_factor: 1.0,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale_factor = scale;
        self
    }
}

/// Context for paint operations.
pub struct PaintContext<'a> {
    pub style_ctx: &'a StyleContext<'a>,
    pub scale_factor: f32,
    pub focus_visible: bool,
}

impl<'a> PaintContext<'a> {
    pub fn new(style_ctx: &'a StyleContext<'a>) -> Self {
        Self {
            style_ctx,
            scale_factor: 1.0,
            focus_visible: false,
        }
    }
}

/// Context for event handling.
pub struct EventContext {
    pub focus: Option<WidgetId>,
    pub mouse_position: Point,
    pub should_redraw: bool,
}

impl EventContext {
    pub fn new() -> Self {
        Self {
            focus: None,
            mouse_position: Point::ZERO,
            should_redraw: false,
        }
    }

    pub fn request_focus(&mut self, widget_id: WidgetId) {
        self.focus = Some(widget_id);
    }

    pub fn release_focus(&mut self) {
        self.focus = None;
    }

    pub fn request_redraw(&mut self) {
        self.should_redraw = true;
    }
}

impl Default for EventContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Base struct containing common widget fields.
#[derive(Debug)]
pub struct WidgetBase {
    pub id: WidgetId,
    pub element_id: Option<String>,
    pub classes: ClassList,
    pub bounds: Rect,
    pub state: WidgetState,
}

impl WidgetBase {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            element_id: None,
            classes: ClassList::new(),
            bounds: Rect::ZERO,
            state: WidgetState::default(),
        }
    }

    pub fn with_class(mut self, class: &str) -> Self {
        self.classes.add(class);
        self
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.element_id = Some(id.to_string());
        self
    }
}

impl Default for WidgetBase {
    fn default() -> Self {
        Self::new()
    }
}
