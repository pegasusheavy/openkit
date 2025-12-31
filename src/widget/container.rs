//! Container widgets (Column, Row).

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::css::FlexDirection;
use crate::event::{Event, EventResult};
use crate::geometry::{EdgeInsets, Point, Rect, Size};
use crate::layout::{Alignment, Constraints, FlexLayout, LayoutResult};
use crate::render::Painter;

/// A vertical stack container (Column).
pub struct Column {
    base: WidgetBase,
    children: Vec<Box<dyn Widget>>,
    gap: f32,
    align: Alignment,
    justify: Alignment,
    padding: EdgeInsets,
    child_positions: Vec<Point>,
}

impl Column {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("column"),
            children: Vec::new(),
            gap: 0.0,
            align: Alignment::Stretch,
            justify: Alignment::Start,
            padding: EdgeInsets::ZERO,
            child_positions: Vec::new(),
        }
    }

    /// Add a child widget.
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// Set the gap between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set cross-axis alignment.
    pub fn align(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }

    /// Set main-axis justification.
    pub fn justify(mut self, justify: Alignment) -> Self {
        self.justify = justify;
        self
    }

    /// Set padding.
    pub fn padding(mut self, padding: impl Into<EdgeInsets>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Column {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "column"
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
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        for child in &self.children {
            let child_size = child.intrinsic_size(ctx);
            width = width.max(child_size.width);
            height += child_size.height;
        }

        // Add gaps
        if self.children.len() > 1 {
            height += self.gap * (self.children.len() - 1) as f32;
        }

        // Add padding
        Size::new(
            width + self.padding.horizontal(),
            height + self.padding.vertical(),
        )
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        // First pass: get intrinsic sizes
        let child_sizes: Vec<Size> = self.children
            .iter()
            .map(|c| c.intrinsic_size(ctx))
            .collect();

        // Calculate total content size
        let total_height: f32 = child_sizes.iter().map(|s| s.height).sum::<f32>()
            + self.gap * (self.children.len().saturating_sub(1)) as f32;
        let max_width = child_sizes.iter().map(|s| s.width).fold(0.0_f32, |a, b| a.max(b));

        // Container size
        let container_size = constraints.constrain(Size::new(
            max_width + self.padding.horizontal(),
            total_height + self.padding.vertical(),
        ));

        // Calculate positions using flex layout
        let flex = FlexLayout {
            direction: FlexDirection::Column,
            justify: self.justify,
            align: self.align,
            gap: self.gap,
        };

        self.child_positions = flex.calculate_positions(container_size, &child_sizes, self.padding);

        // Second pass: layout children with their positions
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_size = child_sizes[i];
            let available_width = match self.align {
                Alignment::Stretch => container_size.width - self.padding.horizontal(),
                _ => child_size.width,
            };

            let child_constraints = Constraints::new(
                0.0,
                available_width,
                0.0,
                child_size.height,
            );
            child.layout(child_constraints, ctx);

            if let Some(pos) = self.child_positions.get(i) {
                child.set_bounds(Rect::from_origin_size(*pos, child.bounds().size));
            }
        }

        self.base.bounds.size = container_size;
        LayoutResult::new(container_size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        // Paint children
        for (i, child) in self.children.iter().enumerate() {
            if let Some(pos) = self.child_positions.get(i) {
                let child_rect = Rect::from_origin_size(
                    Point::new(rect.x() + pos.x, rect.y() + pos.y),
                    child.bounds().size,
                );
                child.paint(painter, child_rect, ctx);
            }
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Propagate to children in reverse order (front to back)
        for child in self.children.iter_mut().rev() {
            if child.handle_event(event, ctx) == EventResult::Handled {
                return EventResult::Handled;
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
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut self.children
    }
}

/// A horizontal stack container (Row).
pub struct Row {
    base: WidgetBase,
    children: Vec<Box<dyn Widget>>,
    gap: f32,
    align: Alignment,
    justify: Alignment,
    padding: EdgeInsets,
    child_positions: Vec<Point>,
}

impl Row {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("row"),
            children: Vec::new(),
            gap: 0.0,
            align: Alignment::Center,
            justify: Alignment::Start,
            padding: EdgeInsets::ZERO,
            child_positions: Vec::new(),
        }
    }

    /// Add a child widget.
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// Set the gap between children.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set cross-axis alignment.
    pub fn align(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }

    /// Set main-axis justification.
    pub fn justify(mut self, justify: Alignment) -> Self {
        self.justify = justify;
        self
    }

    /// Set padding.
    pub fn padding(mut self, padding: impl Into<EdgeInsets>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Row {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "row"
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
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        for child in &self.children {
            let child_size = child.intrinsic_size(ctx);
            width += child_size.width;
            height = height.max(child_size.height);
        }

        // Add gaps
        if self.children.len() > 1 {
            width += self.gap * (self.children.len() - 1) as f32;
        }

        // Add padding
        Size::new(
            width + self.padding.horizontal(),
            height + self.padding.vertical(),
        )
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        // First pass: get intrinsic sizes
        let child_sizes: Vec<Size> = self.children
            .iter()
            .map(|c| c.intrinsic_size(ctx))
            .collect();

        // Calculate total content size
        let total_width: f32 = child_sizes.iter().map(|s| s.width).sum::<f32>()
            + self.gap * (self.children.len().saturating_sub(1)) as f32;
        let max_height = child_sizes.iter().map(|s| s.height).fold(0.0_f32, |a, b| a.max(b));

        // Container size
        let container_size = constraints.constrain(Size::new(
            total_width + self.padding.horizontal(),
            max_height + self.padding.vertical(),
        ));

        // Calculate positions using flex layout
        let flex = FlexLayout {
            direction: FlexDirection::Row,
            justify: self.justify,
            align: self.align,
            gap: self.gap,
        };

        self.child_positions = flex.calculate_positions(container_size, &child_sizes, self.padding);

        // Second pass: layout children with their positions
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_size = child_sizes[i];
            let available_height = match self.align {
                Alignment::Stretch => container_size.height - self.padding.vertical(),
                _ => child_size.height,
            };

            let child_constraints = Constraints::new(
                0.0,
                child_size.width,
                0.0,
                available_height,
            );
            child.layout(child_constraints, ctx);

            if let Some(pos) = self.child_positions.get(i) {
                child.set_bounds(Rect::from_origin_size(*pos, child.bounds().size));
            }
        }

        self.base.bounds.size = container_size;
        LayoutResult::new(container_size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        // Paint children
        for (i, child) in self.children.iter().enumerate() {
            if let Some(pos) = self.child_positions.get(i) {
                let child_rect = Rect::from_origin_size(
                    Point::new(rect.x() + pos.x, rect.y() + pos.y),
                    child.bounds().size,
                );
                child.paint(painter, child_rect, ctx);
            }
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Propagate to children in reverse order (front to back)
        for child in self.children.iter_mut().rev() {
            if child.handle_event(event, ctx) == EventResult::Handled {
                return EventResult::Handled;
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
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut self.children
    }
}
