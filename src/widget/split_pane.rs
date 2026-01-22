//! Split pane widget for resizable panel layouts.
//!
//! Provides horizontal and vertical split views with draggable dividers.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Split orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitOrientation {
    /// Horizontal split (left/right panels)
    #[default]
    Horizontal,
    /// Vertical split (top/bottom panels)
    Vertical,
}

/// Split pane widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::widget::split_pane::*;
///
/// let split = SplitPane::horizontal()
///     .first(sidebar_widget)
///     .second(main_content)
///     .initial_ratio(0.25)
///     .min_first_size(200.0)
///     .collapsible(true);
/// ```
pub struct SplitPane {
    base: WidgetBase,
    orientation: SplitOrientation,
    first: Option<Box<dyn Widget>>,
    second: Option<Box<dyn Widget>>,
    ratio: f32,
    min_first_size: f32,
    min_second_size: f32,
    divider_size: f32,
    divider_hovered: bool,
    dragging: bool,
    drag_offset: f32,
    collapsible: bool,
    first_collapsed: bool,
    second_collapsed: bool,
    show_collapse_buttons: bool,
}

impl SplitPane {
    /// Create a new horizontal split pane.
    pub fn horizontal() -> Self {
        Self::new(SplitOrientation::Horizontal)
    }

    /// Create a new vertical split pane.
    pub fn vertical() -> Self {
        Self::new(SplitOrientation::Vertical)
    }

    /// Create a new split pane with the given orientation.
    pub fn new(orientation: SplitOrientation) -> Self {
        Self {
            base: WidgetBase::new().with_class("split-pane"),
            orientation,
            first: None,
            second: None,
            ratio: 0.5,
            min_first_size: 100.0,
            min_second_size: 100.0,
            divider_size: 6.0,
            divider_hovered: false,
            dragging: false,
            drag_offset: 0.0,
            collapsible: false,
            first_collapsed: false,
            second_collapsed: false,
            show_collapse_buttons: false,
        }
    }

    /// Set the first panel widget.
    pub fn first<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.first = Some(Box::new(widget));
        self
    }

    /// Set the second panel widget.
    pub fn second<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.second = Some(Box::new(widget));
        self
    }

    /// Set the initial split ratio (0.0 to 1.0).
    pub fn initial_ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Set the minimum size for the first panel.
    pub fn min_first_size(mut self, size: f32) -> Self {
        self.min_first_size = size;
        self
    }

    /// Set the minimum size for the second panel.
    pub fn min_second_size(mut self, size: f32) -> Self {
        self.min_second_size = size;
        self
    }

    /// Set the divider size in pixels.
    pub fn divider_size(mut self, size: f32) -> Self {
        self.divider_size = size;
        self
    }

    /// Enable collapsible panels.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Show collapse buttons on the divider.
    pub fn show_collapse_buttons(mut self, show: bool) -> Self {
        self.show_collapse_buttons = show;
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Collapse the first panel.
    pub fn collapse_first(&mut self) {
        if self.collapsible {
            self.first_collapsed = true;
            self.second_collapsed = false;
        }
    }

    /// Collapse the second panel.
    pub fn collapse_second(&mut self) {
        if self.collapsible {
            self.second_collapsed = true;
            self.first_collapsed = false;
        }
    }

    /// Expand both panels.
    pub fn expand(&mut self) {
        self.first_collapsed = false;
        self.second_collapsed = false;
    }

    /// Get the current ratio.
    pub fn get_ratio(&self) -> f32 {
        self.ratio
    }

    /// Set the ratio programmatically.
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.0, 1.0);
    }

    fn divider_rect(&self) -> Rect {
        let bounds = self.base.bounds;
        let total_size = match self.orientation {
            SplitOrientation::Horizontal => bounds.width(),
            SplitOrientation::Vertical => bounds.height(),
        };

        let first_size = if self.first_collapsed {
            0.0
        } else if self.second_collapsed {
            total_size - self.divider_size
        } else {
            (total_size - self.divider_size) * self.ratio
        };

        match self.orientation {
            SplitOrientation::Horizontal => {
                Rect::new(bounds.x() + first_size, bounds.y(), self.divider_size, bounds.height())
            }
            SplitOrientation::Vertical => {
                Rect::new(bounds.x(), bounds.y() + first_size, bounds.width(), self.divider_size)
            }
        }
    }

    fn first_rect(&self) -> Rect {
        let bounds = self.base.bounds;
        let divider = self.divider_rect();

        match self.orientation {
            SplitOrientation::Horizontal => {
                Rect::new(bounds.x(), bounds.y(), divider.x() - bounds.x(), bounds.height())
            }
            SplitOrientation::Vertical => {
                Rect::new(bounds.x(), bounds.y(), bounds.width(), divider.y() - bounds.y())
            }
        }
    }

    fn second_rect(&self) -> Rect {
        let bounds = self.base.bounds;
        let divider = self.divider_rect();

        match self.orientation {
            SplitOrientation::Horizontal => {
                let x = divider.x() + divider.width();
                Rect::new(x, bounds.y(), bounds.x() + bounds.width() - x, bounds.height())
            }
            SplitOrientation::Vertical => {
                let y = divider.y() + divider.height();
                Rect::new(bounds.x(), y, bounds.width(), bounds.y() + bounds.height() - y)
            }
        }
    }

    fn point_on_divider(&self, point: Point) -> bool {
        let divider = self.divider_rect();
        // Extend hit area slightly for easier grabbing
        let hit_rect = match self.orientation {
            SplitOrientation::Horizontal => {
                Rect::new(divider.x() - 2.0, divider.y(), divider.width() + 4.0, divider.height())
            }
            SplitOrientation::Vertical => {
                Rect::new(divider.x(), divider.y() - 2.0, divider.width(), divider.height() + 4.0)
            }
        };
        hit_rect.contains(point)
    }

    fn update_ratio_from_position(&mut self, pos: Point) {
        let bounds = self.base.bounds;
        let (total_size, current_pos) = match self.orientation {
            SplitOrientation::Horizontal => (bounds.width(), pos.x - bounds.x()),
            SplitOrientation::Vertical => (bounds.height(), pos.y - bounds.y()),
        };

        let usable_size = total_size - self.divider_size;
        if usable_size <= 0.0 {
            return;
        }

        let new_first_size = current_pos - self.drag_offset;
        let new_ratio = new_first_size / usable_size;

        // Enforce minimum sizes
        let min_first_ratio = self.min_first_size / usable_size;
        let max_first_ratio = 1.0 - (self.min_second_size / usable_size);

        self.ratio = new_ratio.clamp(min_first_ratio, max_first_ratio);
    }
}

impl Default for SplitPane {
    fn default() -> Self {
        Self::horizontal()
    }
}

impl Widget for SplitPane {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "split-pane"
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
        Size::new(400.0, 300.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;

        // Layout children - compute rects before borrowing
        let first_rect = self.first_rect();
        let second_rect = self.second_rect();

        if let Some(ref mut first) = self.first {
            let child_constraints = Constraints::tight(first_rect.size);
            first.layout(child_constraints, ctx);
            first.set_bounds(first_rect);
        }

        if let Some(ref mut second) = self.second {
            let child_constraints = Constraints::tight(second_rect.size);
            second.layout(child_constraints, ctx);
            second.set_bounds(second_rect);
        }

        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        painter.fill_rect(rect, theme.colors.background);

        // Paint first panel
        if let Some(ref first) = self.first {
            if !self.first_collapsed {
                let first_rect = self.first_rect();
                first.paint(painter, first_rect, ctx);
            }
        }

        // Paint second panel
        if let Some(ref second) = self.second {
            if !self.second_collapsed {
                let second_rect = self.second_rect();
                second.paint(painter, second_rect, ctx);
            }
        }

        // Paint divider
        let divider = self.divider_rect();
        let divider_color = if self.dragging {
            theme.colors.accent
        } else if self.divider_hovered {
            theme.colors.accent.with_alpha(0.5)
        } else {
            theme.colors.border
        };

        painter.fill_rect(divider, divider_color);

        // Divider grip lines
        let grip_color = theme.colors.muted_foreground.with_alpha(0.5);
        match self.orientation {
            SplitOrientation::Horizontal => {
                let cx = divider.x() + divider.width() / 2.0;
                let cy = divider.y() + divider.height() / 2.0;
                for i in -1..=1 {
                    painter.fill_rect(
                        Rect::new(cx - 0.5, cy + i as f32 * 6.0 - 8.0, 1.0, 16.0),
                        grip_color,
                    );
                }
            }
            SplitOrientation::Vertical => {
                let cx = divider.x() + divider.width() / 2.0;
                let cy = divider.y() + divider.height() / 2.0;
                for i in -1..=1 {
                    painter.fill_rect(
                        Rect::new(cx + i as f32 * 6.0 - 8.0, cy - 0.5, 16.0, 1.0),
                        grip_color,
                    );
                }
            }
        }

        // Collapse buttons
        if self.show_collapse_buttons && self.collapsible {
            let btn_size = 16.0;
            let (btn_x, btn_y) = match self.orientation {
                SplitOrientation::Horizontal => (
                    divider.x() + (divider.width() - btn_size) / 2.0,
                    divider.y() + 8.0,
                ),
                SplitOrientation::Vertical => (
                    divider.x() + 8.0,
                    divider.y() + (divider.height() - btn_size) / 2.0,
                ),
            };

            let btn_color = if self.divider_hovered {
                theme.colors.foreground
            } else {
                theme.colors.muted_foreground
            };

            let icon = match self.orientation {
                SplitOrientation::Horizontal => if self.first_collapsed { "▶" } else { "◀" },
                SplitOrientation::Vertical => if self.first_collapsed { "▼" } else { "▲" },
            };

            painter.draw_text(
                icon,
                Point::new(btn_x, btn_y + btn_size / 2.0 + 4.0),
                btn_color,
                12.0,
            );
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // First, try to handle events in children
        if let Some(ref mut first) = self.first {
            if !self.first_collapsed {
                let result = first.handle_event(event, ctx);
                if result == EventResult::Handled {
                    return result;
                }
            }
        }

        if let Some(ref mut second) = self.second {
            if !self.second_collapsed {
                let result = second.handle_event(event, ctx);
                if result == EventResult::Handled {
                    return result;
                }
            }
        }

        // Handle divider interaction
        if let Event::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Move => {
                    if self.dragging {
                        self.update_ratio_from_position(mouse.position);
                        // Re-layout children - compute rects before borrowing
                        let first_rect = self.first_rect();
                        let second_rect = self.second_rect();
                        if let Some(ref mut first) = self.first {
                            first.set_bounds(first_rect);
                        }
                        if let Some(ref mut second) = self.second {
                            second.set_bounds(second_rect);
                        }
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }

                    let on_divider = self.point_on_divider(mouse.position);
                    if on_divider != self.divider_hovered {
                        self.divider_hovered = on_divider;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                    if self.point_on_divider(mouse.position) {
                        self.dragging = true;
                        // Calculate drag offset
                        let divider = self.divider_rect();
                        self.drag_offset = match self.orientation {
                            SplitOrientation::Horizontal => mouse.position.x - divider.x(),
                            SplitOrientation::Vertical => mouse.position.y - divider.y(),
                        };
                        ctx.request_focus(self.base.id);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) => {
                    if self.dragging {
                        self.dragging = false;
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
                }
                MouseEventKind::Leave => {
                    if self.divider_hovered {
                        self.divider_hovered = false;
                        ctx.request_redraw();
                    }
                }
                _ => {}
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
        &[]
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut []
    }
}
