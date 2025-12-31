//! Spacer widget for flexible layout spacing.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A flexible spacer widget that expands to fill available space.
///
/// Commonly used in flex layouts to push content apart.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Push items to opposite ends
/// row![8;
///     label!("Left"),
///     Spacer::new(),
///     label!("Right"),
/// ]
///
/// // Fixed size spacer
/// row![8;
///     button!("A"),
///     Spacer::fixed(16.0),
///     button!("B"),
/// ]
/// ```
pub struct Spacer {
    base: WidgetBase,
    /// Fixed size (if set, spacer won't flex)
    fixed_size: Option<f32>,
    /// Flex factor
    flex: f32,
}

impl Spacer {
    /// Create a new flexible spacer.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("spacer"),
            fixed_size: None,
            flex: 1.0,
        }
    }

    /// Create a spacer with a fixed size.
    pub fn fixed(size: f32) -> Self {
        Self {
            base: WidgetBase::new().with_class("spacer"),
            fixed_size: Some(size),
            flex: 0.0,
        }
    }

    /// Set the flex factor (default is 1.0).
    pub fn flex(mut self, flex: f32) -> Self {
        self.flex = flex;
        self
    }

    /// Get the flex factor.
    pub fn get_flex(&self) -> f32 {
        self.flex
    }

    /// Check if this is a fixed spacer.
    pub fn is_fixed(&self) -> bool {
        self.fixed_size.is_some()
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spacer {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "spacer"
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
        if let Some(size) = self.fixed_size {
            Size::new(size, size)
        } else {
            Size::ZERO
        }
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        let size = if let Some(fixed) = self.fixed_size {
            Size::new(
                constraints.constrain(Size::new(fixed, 0.0)).width,
                constraints.constrain(Size::new(0.0, fixed)).height,
            )
        } else {
            // Flexible spacer expands to fill available space
            Size::new(constraints.max_width, constraints.max_height)
        };

        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, _painter: &mut Painter, _rect: Rect, _ctx: &PaintContext) {
        // Spacers are invisible
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        // Spacers don't handle events
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;
    }
}
