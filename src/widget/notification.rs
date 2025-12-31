//! Notification/toast widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Notification urgency level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationUrgency {
    /// Low priority (subtle styling)
    Low,
    /// Normal priority (default)
    #[default]
    Normal,
    /// Critical/urgent (prominent styling)
    Critical,
}

/// A desktop notification widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let notification = Notification::new()
///     .title("New Message")
///     .body("You have received a new email from Alice.")
///     .icon("📧")
///     .urgency(NotificationUrgency::Normal)
///     .action("reply", "Reply")
///     .action("dismiss", "Dismiss")
///     .on_action(|action| println!("Action: {}", action))
///     .on_close(|| println!("Notification closed"));
/// ```
#[allow(clippy::type_complexity)]
pub struct Notification {
    base: WidgetBase,
    title: String,
    body: String,
    icon: Option<String>,
    app_name: Option<String>,
    urgency: NotificationUrgency,
    actions: Vec<(String, String)>, // (id, label)
    hovered_action: Option<usize>,
    close_hovered: bool,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    on_action: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_close: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Notification {
    /// Create a new notification.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("notification"),
            title: String::new(),
            body: String::new(),
            icon: None,
            app_name: None,
            urgency: NotificationUrgency::default(),
            actions: Vec::new(),
            hovered_action: None,
            close_hovered: false,
            on_click: None,
            on_action: None,
            on_close: None,
        }
    }

    /// Set the notification title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the notification body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Set the notification icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the application name.
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Set the urgency level.
    pub fn urgency(mut self, urgency: NotificationUrgency) -> Self {
        self.urgency = urgency;
        self
    }

    /// Add an action button.
    pub fn action(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.actions.push((id.into(), label.into()));
        self
    }

    /// Set the click handler (clicking the notification body).
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set the action handler.
    pub fn on_action<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_action = Some(Box::new(handler));
        self
    }

    /// Set the close handler.
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    fn get_close_button_rect(&self) -> Rect {
        Rect::new(
            self.base.bounds.x() + self.base.bounds.width() - 28.0,
            self.base.bounds.y() + 8.0,
            20.0,
            20.0,
        )
    }

    fn get_action_rects(&self) -> Vec<Rect> {
        let mut rects = Vec::new();
        let action_y = self.base.bounds.y() + self.base.bounds.height() - 40.0;
        let mut x = self.base.bounds.x() + 16.0;

        if self.icon.is_some() {
            x += 48.0;
        }

        for (_, label) in &self.actions {
            let width = label.len() as f32 * 8.0 + 24.0;
            rects.push(Rect::new(x, action_y, width, 28.0));
            x += width + 8.0;
        }
        rects
    }
}

impl Default for Notification {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Notification {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "notification"
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
        let height = if self.actions.is_empty() { 80.0 } else { 120.0 };
        Size::new(360.0, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let radius = BorderRadius::all(theme.radii.lg * theme.typography.base_size);

        // Urgency-based accent color
        let accent = match self.urgency {
            NotificationUrgency::Low => theme.colors.muted,
            NotificationUrgency::Normal => theme.colors.card,
            NotificationUrgency::Critical => theme.colors.destructive.with_alpha(0.1),
        };

        // Shadow
        let shadow_rect = Rect::new(rect.x() + 2.0, rect.y() + 4.0, rect.width(), rect.height());
        painter.fill_rounded_rect(shadow_rect, Color::BLACK.with_alpha(0.2), radius);

        // Background
        painter.fill_rounded_rect(rect, accent, radius);
        painter.stroke_rect(rect, theme.colors.border, 1.0);

        // Urgency indicator bar
        if self.urgency == NotificationUrgency::Critical {
            let bar_rect = Rect::new(rect.x(), rect.y(), 4.0, rect.height());
            painter.fill_rect(bar_rect, theme.colors.destructive);
        }

        let mut content_x = rect.x() + 16.0;

        // Icon
        if let Some(ref icon) = self.icon {
            painter.draw_text(
                icon,
                Point::new(content_x, rect.y() + 40.0),
                theme.colors.foreground,
                32.0,
            );
            content_x += 48.0;
        }

        // App name
        if let Some(ref app_name) = self.app_name {
            painter.draw_text(
                app_name,
                Point::new(content_x, rect.y() + 20.0),
                theme.colors.muted_foreground,
                11.0,
            );
        }

        // Title
        let title_y = if self.app_name.is_some() { 38.0 } else { 28.0 };
        painter.draw_text(
            &self.title,
            Point::new(content_x, rect.y() + title_y),
            theme.colors.foreground,
            14.0,
        );

        // Body
        if !self.body.is_empty() {
            painter.draw_text(
                &self.body,
                Point::new(content_x, rect.y() + title_y + 20.0),
                theme.colors.muted_foreground,
                13.0,
            );
        }

        // Close button
        let close_rect = self.get_close_button_rect();
        let close_color = if self.close_hovered {
            theme.colors.foreground
        } else {
            theme.colors.muted_foreground
        };
        painter.draw_text("✕", Point::new(close_rect.x() + 4.0, close_rect.y() + 14.0), close_color, 12.0);

        // Action buttons
        if !self.actions.is_empty() {
            let action_rects = self.get_action_rects();
            for (i, ((_, label), rect)) in self.actions.iter().zip(action_rects.iter()).enumerate() {
                let bg_color = if self.hovered_action == Some(i) {
                    theme.colors.accent
                } else {
                    theme.colors.secondary
                };
                let fg_color = if self.hovered_action == Some(i) {
                    theme.colors.accent_foreground
                } else {
                    theme.colors.secondary_foreground
                };

                let btn_radius = BorderRadius::all(theme.radii.sm * theme.typography.base_size);
                painter.fill_rounded_rect(*rect, bg_color, btn_radius);
                painter.draw_text(
                    label,
                    Point::new(rect.x() + 12.0, rect.y() + 18.0),
                    fg_color,
                    12.0,
                );
            }
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            let in_bounds = self.bounds().contains(mouse.position);
            let close_rect = self.get_close_button_rect();
            let in_close = close_rect.contains(mouse.position);
            let action_rects = self.get_action_rects();
            let in_action = action_rects.iter().position(|r| r.contains(mouse.position));

            match mouse.kind {
                MouseEventKind::Move => {
                    let old_close = self.close_hovered;
                    let old_action = self.hovered_action;

                    self.close_hovered = in_close;
                    self.hovered_action = in_action;

                    if old_close != self.close_hovered || old_action != self.hovered_action {
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) => {
                    if in_close {
                        if let Some(handler) = &self.on_close {
                            handler();
                        }
                        return EventResult::Handled;
                    }

                    if let Some(action_idx) = in_action {
                        if let Some((id, _)) = self.actions.get(action_idx) {
                            let id = id.clone();
                            if let Some(handler) = &self.on_action {
                                handler(&id);
                            }
                            return EventResult::Handled;
                        }
                    }

                    if in_bounds {
                        if let Some(handler) = &self.on_click {
                            handler();
                        }
                        return EventResult::Handled;
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
}
