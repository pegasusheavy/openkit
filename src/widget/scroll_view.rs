//! Scrollable container widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Scroll bar visibility mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollBarVisibility {
    /// Always visible
    Always,
    /// Visible when content overflows (default)
    #[default]
    Auto,
    /// Always hidden
    Hidden,
}

/// A scrollable container widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let scroll = ScrollView::new()
///     .content(col![8;
///         // Many items that overflow the container
///         label!("Item 1"),
///         label!("Item 2"),
///         // ... more items
///     ])
///     .max_height(300.0);
/// ```
pub struct ScrollView {
    base: WidgetBase,
    content: Option<Box<dyn Widget>>,
    scroll_x: f32,
    scroll_y: f32,
    content_size: Size,
    max_width: Option<f32>,
    max_height: Option<f32>,
    horizontal_scroll: ScrollBarVisibility,
    vertical_scroll: ScrollBarVisibility,
    scrollbar_width: f32,
    dragging_scrollbar: bool,
    drag_start_y: f32,
    drag_start_scroll: f32,
}

impl ScrollView {
    /// Create a new scroll view.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("scroll-view"),
            content: None,
            scroll_x: 0.0,
            scroll_y: 0.0,
            content_size: Size::ZERO,
            max_width: None,
            max_height: None,
            horizontal_scroll: ScrollBarVisibility::Hidden,
            vertical_scroll: ScrollBarVisibility::Auto,
            scrollbar_width: 8.0,
            dragging_scrollbar: false,
            drag_start_y: 0.0,
            drag_start_scroll: 0.0,
        }
    }

    /// Set the content widget.
    pub fn content<W: Widget + 'static>(mut self, content: W) -> Self {
        self.content = Some(Box::new(content));
        self
    }

    /// Set the maximum width.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set the maximum height.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set horizontal scroll bar visibility.
    pub fn horizontal_scroll(mut self, visibility: ScrollBarVisibility) -> Self {
        self.horizontal_scroll = visibility;
        self
    }

    /// Set vertical scroll bar visibility.
    pub fn vertical_scroll(mut self, visibility: ScrollBarVisibility) -> Self {
        self.vertical_scroll = visibility;
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Scroll to a position.
    pub fn scroll_to(&mut self, x: f32, y: f32) {
        self.scroll_x = x.max(0.0);
        self.scroll_y = y.max(0.0);
        self.clamp_scroll();
    }

    /// Scroll to top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_y = 0.0;
    }

    /// Scroll to bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_y = (self.content_size.height - self.base.bounds.height()).max(0.0);
    }

    fn clamp_scroll(&mut self) {
        let max_scroll_x = (self.content_size.width - self.viewport_width()).max(0.0);
        let max_scroll_y = (self.content_size.height - self.viewport_height()).max(0.0);
        self.scroll_x = self.scroll_x.clamp(0.0, max_scroll_x);
        self.scroll_y = self.scroll_y.clamp(0.0, max_scroll_y);
    }

    fn viewport_width(&self) -> f32 {
        let scrollbar = if self.should_show_vertical_scrollbar() { self.scrollbar_width } else { 0.0 };
        self.base.bounds.width() - scrollbar
    }

    fn viewport_height(&self) -> f32 {
        let scrollbar = if self.should_show_horizontal_scrollbar() { self.scrollbar_width } else { 0.0 };
        self.base.bounds.height() - scrollbar
    }

    fn should_show_vertical_scrollbar(&self) -> bool {
        match self.vertical_scroll {
            ScrollBarVisibility::Always => true,
            ScrollBarVisibility::Hidden => false,
            ScrollBarVisibility::Auto => self.content_size.height > self.base.bounds.height(),
        }
    }

    fn should_show_horizontal_scrollbar(&self) -> bool {
        match self.horizontal_scroll {
            ScrollBarVisibility::Always => true,
            ScrollBarVisibility::Hidden => false,
            ScrollBarVisibility::Auto => self.content_size.width > self.base.bounds.width(),
        }
    }

    fn vertical_scrollbar_rect(&self) -> Rect {
        Rect::new(
            self.base.bounds.x() + self.base.bounds.width() - self.scrollbar_width,
            self.base.bounds.y(),
            self.scrollbar_width,
            self.viewport_height(),
        )
    }

    fn vertical_thumb_rect(&self) -> Rect {
        let track = self.vertical_scrollbar_rect();
        let content_height = self.content_size.height;
        let viewport_height = self.viewport_height();

        if content_height <= viewport_height {
            return Rect::new(track.x(), track.y(), track.width(), track.height());
        }

        let thumb_height = (viewport_height / content_height * track.height()).max(20.0);
        let max_scroll = content_height - viewport_height;
        let thumb_y = track.y() + (self.scroll_y / max_scroll) * (track.height() - thumb_height);

        Rect::new(track.x() + 2.0, thumb_y, track.width() - 4.0, thumb_height)
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ScrollView {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "scroll-view"
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
        if let Some(content) = &self.content {
            content.intrinsic_size(ctx)
        } else {
            Size::new(100.0, 100.0)
        }
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        // Calculate our size with max constraints
        let max_w = self.max_width.unwrap_or(constraints.max_width);
        let max_h = self.max_height.unwrap_or(constraints.max_height);

        // Layout content with unlimited space in scroll direction
        if let Some(content) = &mut self.content {
            let content_constraints = Constraints {
                min_width: 0.0,
                min_height: 0.0,
                max_width: if self.horizontal_scroll != ScrollBarVisibility::Hidden { f32::MAX } else { max_w },
                max_height: if self.vertical_scroll != ScrollBarVisibility::Hidden { f32::MAX } else { max_h },
            };

            let result = content.layout(content_constraints, ctx);
            self.content_size = result.size;
        }

        // Our size is constrained
        let size = Size::new(
            max_w.min(constraints.max_width),
            max_h.min(constraints.max_height),
        );
        self.base.bounds.size = size;
        self.clamp_scroll();

        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        painter.fill_rect(rect, theme.colors.background);

        // Paint content (with clipping conceptually - real clipping needs render layer support)
        if let Some(content) = &self.content {
            let _viewport = Rect::new(
                rect.x(),
                rect.y(),
                self.viewport_width(),
                self.viewport_height(),
            );

            // Offset content by scroll position
            let content_rect = Rect::new(
                rect.x() - self.scroll_x,
                rect.y() - self.scroll_y,
                self.content_size.width,
                self.content_size.height,
            );

            content.paint(painter, content_rect, ctx);
        }

        // Vertical scrollbar
        if self.should_show_vertical_scrollbar() {
            let track = self.vertical_scrollbar_rect();
            painter.fill_rect(track, theme.colors.muted.with_alpha(0.3));

            let thumb = self.vertical_thumb_rect();
            let thumb_color = if self.dragging_scrollbar {
                theme.colors.muted_foreground
            } else {
                theme.colors.muted_foreground.with_alpha(0.5)
            };
            let thumb_radius = BorderRadius::all(self.scrollbar_width / 2.0 - 2.0);
            painter.fill_rounded_rect(thumb, thumb_color, thumb_radius);
        }

        // Border
        painter.stroke_rect(rect, theme.colors.border, 1.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
                let scrollbar_rect = self.vertical_scrollbar_rect();
                let in_scrollbar = scrollbar_rect.contains(mouse.position);

                match mouse.kind {
                    MouseEventKind::Move => {
                        if self.dragging_scrollbar {
                            let track = self.vertical_scrollbar_rect();
                            let thumb_height = self.vertical_thumb_rect().height();
                            let max_thumb_travel = track.height() - thumb_height;

                            if max_thumb_travel > 0.0 {
                                let delta_y = mouse.position.y - self.drag_start_y;
                                let scroll_ratio = delta_y / max_thumb_travel;
                                let max_scroll = self.content_size.height - self.viewport_height();
                                self.scroll_y = (self.drag_start_scroll + scroll_ratio * max_scroll).clamp(0.0, max_scroll);
                                ctx.request_redraw();
                            }
                            return EventResult::Handled;
                        }
                    }
                    MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                        if in_scrollbar {
                            let thumb = self.vertical_thumb_rect();
                            if thumb.contains(mouse.position) {
                                self.dragging_scrollbar = true;
                                self.drag_start_y = mouse.position.y;
                                self.drag_start_scroll = self.scroll_y;
                            } else {
                                // Click on track - jump to position
                                let track = self.vertical_scrollbar_rect();
                                let ratio = (mouse.position.y - track.y()) / track.height();
                                self.scroll_y = ratio * (self.content_size.height - self.viewport_height());
                                self.clamp_scroll();
                            }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    MouseEventKind::Up if mouse.button == Some(MouseButton::Left) => {
                        if self.dragging_scrollbar {
                            self.dragging_scrollbar = false;
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    // Wheel scroll would go here
                    _ => {}
                }

                // Forward to content
                if let Some(content) = &mut self.content {
                    return content.handle_event(event, ctx);
                }
            }
            _ => {
                if let Some(content) = &mut self.content {
                    return content.handle_event(event, ctx);
                }
            }
        }
        EventResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.base.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.base.bounds = bounds;

        if let Some(content) = &mut self.content {
            content.set_bounds(Rect::new(
                bounds.x() - self.scroll_x,
                bounds.y() - self.scroll_y,
                self.content_size.width,
                self.content_size.height,
            ));
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        if let Some(content) = &self.content {
            std::slice::from_ref(content)
        } else {
            &[]
        }
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        if let Some(content) = &mut self.content {
            std::slice::from_mut(content)
        } else {
            &mut []
        }
    }
}
