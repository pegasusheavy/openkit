//! Configurable Panel widget
//!
//! A highly customizable panel/taskbar widget that can be positioned
//! on any edge of the screen with various layouts and behaviors.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{Color, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Panel position on screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

/// Panel autohide behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelAutohide {
    #[default]
    Never,
    Always,
    InFullscreen,
    Smart,
}

/// A configurable panel/taskbar widget
pub struct Panel {
    base: WidgetBase,
    position: PanelPosition,
    thickness: f32,
    background: Color,
    blur: bool,
    border_color: Option<Color>,
    autohide: PanelAutohide,
    is_hidden: bool,
    padding: f32,
    shadow: bool,
    monitor: usize,
    extend_to_edges: bool,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("panel"),
            position: PanelPosition::Bottom,
            thickness: 48.0,
            background: Color::rgba(0.1, 0.1, 0.1, 0.9),
            blur: true,
            border_color: None,
            autohide: PanelAutohide::Never,
            is_hidden: false,
            padding: 4.0,
            shadow: true,
            monitor: 0,
            extend_to_edges: true,
        }
    }

    pub fn position(mut self, pos: PanelPosition) -> Self {
        self.position = pos;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn blur(mut self, enabled: bool) -> Self {
        self.blur = enabled;
        self
    }

    pub fn border(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn autohide(mut self, behavior: PanelAutohide) -> Self {
        self.autohide = behavior;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn shadow(mut self, enabled: bool) -> Self {
        self.shadow = enabled;
        self
    }

    pub fn monitor(mut self, index: usize) -> Self {
        self.monitor = index;
        self
    }

    pub fn extend_edges(mut self, extend: bool) -> Self {
        self.extend_to_edges = extend;
        self
    }

    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self.position, PanelPosition::Top | PanelPosition::Bottom)
    }

    pub fn show(&mut self) {
        self.is_hidden = false;
    }

    pub fn hide(&mut self) {
        self.is_hidden = true;
    }

    pub fn effective_thickness(&self) -> f32 {
        if self.is_hidden { 0.0 } else { self.thickness }
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Panel {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "panel"
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
        if self.is_horizontal() {
            Size::new(0.0, self.thickness) // Width expands to fill
        } else {
            Size::new(self.thickness, 0.0) // Height expands to fill
        }
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        let size = if self.is_horizontal() {
            Size::new(constraints.max_width, self.thickness)
        } else {
            Size::new(self.thickness, constraints.max_height)
        };
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        if self.is_hidden {
            return;
        }

        // Draw shadow
        if self.shadow {
            let shadow_offset = match self.position {
                PanelPosition::Top => 3.0,
                PanelPosition::Bottom => -3.0,
                PanelPosition::Left => 3.0,
                PanelPosition::Right => -3.0,
            };
            let shadow_rect = if self.is_horizontal() {
                Rect::new(rect.x(), rect.y() + shadow_offset, rect.width(), rect.height())
            } else {
                Rect::new(rect.x() + shadow_offset, rect.y(), rect.width(), rect.height())
            };
            painter.fill_rect(shadow_rect, Color::rgba(0.0, 0.0, 0.0, 0.3));
        }

        // Draw background
        painter.fill_rect(rect, self.background);

        // Draw border
        if let Some(border) = self.border_color {
            let border_rect = match self.position {
                PanelPosition::Top => Rect::new(rect.x(), rect.max_y() - 1.0, rect.width(), 1.0),
                PanelPosition::Bottom => Rect::new(rect.x(), rect.y(), rect.width(), 1.0),
                PanelPosition::Left => Rect::new(rect.max_x() - 1.0, rect.y(), 1.0, rect.height()),
                PanelPosition::Right => Rect::new(rect.x(), rect.y(), 1.0, rect.height()),
            };
            painter.fill_rect(border_rect, border);
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
