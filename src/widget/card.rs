//! Card container widget with styled background.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Rect, Size, EdgeInsets};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Card variant for different styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardVariant {
    /// Default card with border and shadow
    #[default]
    Default,
    /// Elevated card with more shadow
    Elevated,
    /// Outlined card (border only, no shadow)
    Outlined,
    /// Ghost card (minimal styling)
    Ghost,
    /// Glass effect (semi-transparent)
    Glass,
}

/// A card container widget with background, border, and shadow.
///
/// Cards are commonly used to group related content.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let login_card = Card::new()
///     .variant(CardVariant::Elevated)
///     .padding(24.0)
///     .child(col![16;
///         label!("Welcome"),
///         textfield!("Username"),
///         password_field!(),
///         button!("Login"),
///     ]);
/// ```
pub struct Card {
    base: WidgetBase,
    child: Option<Box<dyn Widget>>,
    variant: CardVariant,
    padding: EdgeInsets,
    border_radius: Option<f32>,
}

impl Card {
    /// Create a new card.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("card"),
            child: None,
            variant: CardVariant::default(),
            padding: EdgeInsets::all(16.0),
            border_radius: None,
        }
    }

    /// Set the child widget.
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set the card variant.
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set padding (all sides).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = EdgeInsets::all(padding);
        self
    }

    /// Set padding with individual sides.
    pub fn padding_xy(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding = EdgeInsets::new(vertical, horizontal, vertical, horizontal);
        self
    }

    /// Set custom border radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    fn get_border_radius(&self, theme: &crate::theme::ThemeData) -> BorderRadius {
        let radius = self.border_radius
            .unwrap_or(theme.radii.lg * theme.typography.base_size);
        BorderRadius::all(radius)
    }

    fn background_color(&self, theme: &crate::theme::ThemeData) -> Color {
        match self.variant {
            CardVariant::Default | CardVariant::Elevated | CardVariant::Outlined => {
                theme.colors.card
            }
            CardVariant::Ghost => Color::TRANSPARENT,
            CardVariant::Glass => theme.colors.card.with_alpha(0.8),
        }
    }

    fn should_draw_border(&self) -> bool {
        matches!(self.variant, CardVariant::Default | CardVariant::Outlined | CardVariant::Glass)
    }

    fn should_draw_shadow(&self) -> bool {
        matches!(self.variant, CardVariant::Default | CardVariant::Elevated)
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Card {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "card"
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
        if let Some(child) = &self.child {
            let child_size = child.intrinsic_size(ctx);
            Size::new(
                child_size.width + self.padding.left + self.padding.right,
                child_size.height + self.padding.top + self.padding.bottom,
            )
        } else {
            Size::new(
                self.padding.left + self.padding.right + 100.0,
                self.padding.top + self.padding.bottom + 100.0,
            )
        }
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let child_constraints = Constraints {
            min_width: 0.0,
            min_height: 0.0,
            max_width: constraints.max_width - self.padding.left - self.padding.right,
            max_height: constraints.max_height - self.padding.top - self.padding.bottom,
        };

        let child_size = if let Some(child) = &mut self.child {
            let result = child.layout(child_constraints, ctx);

            // Position child with padding offset
            child.set_bounds(Rect::new(
                self.base.bounds.x() + self.padding.left,
                self.base.bounds.y() + self.padding.top,
                result.size.width,
                result.size.height,
            ));

            result.size
        } else {
            Size::ZERO
        };

        let size = Size::new(
            child_size.width + self.padding.left + self.padding.right,
            child_size.height + self.padding.top + self.padding.bottom,
        );

        let size = constraints.constrain(size);
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let radius = self.get_border_radius(theme);

        // Shadow (simplified - real shadows would use blur)
        if self.should_draw_shadow() {
            let shadow_offset = match self.variant {
                CardVariant::Elevated => 8.0,
                _ => 2.0,
            };
            let shadow_rect = Rect::new(
                rect.x() + shadow_offset / 2.0,
                rect.y() + shadow_offset,
                rect.width(),
                rect.height(),
            );
            painter.fill_rounded_rect(
                shadow_rect,
                Color::BLACK.with_alpha(0.1),
                radius,
            );
        }

        // Background
        painter.fill_rounded_rect(rect, self.background_color(theme), radius);

        // Border
        if self.should_draw_border() {
            painter.stroke_rect(rect, theme.colors.border, 1.0);
        }

        // Paint child
        if let Some(child) = &self.child {
            let child_rect = Rect::new(
                rect.x() + self.padding.left,
                rect.y() + self.padding.top,
                rect.width() - self.padding.left - self.padding.right,
                rect.height() - self.padding.top - self.padding.bottom,
            );
            child.paint(painter, child_rect, ctx);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Forward events to child
        if let Some(child) = &mut self.child {
            return child.handle_event(event, ctx);
        }
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;

        // Update child bounds
        if let Some(child) = &mut self.child {
            child.set_bounds(Rect::new(
                bounds.x() + self.padding.left,
                bounds.y() + self.padding.top,
                bounds.width() - self.padding.left - self.padding.right,
                bounds.height() - self.padding.top - self.padding.bottom,
            ));
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        if let Some(child) = &self.child {
            std::slice::from_ref(child)
        } else {
            &[]
        }
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        if let Some(child) = &mut self.child {
            std::slice::from_mut(child)
        } else {
            &mut []
        }
    }
}
