//! VPN Status indicator widget.
//!
//! Displays the current VPN connection status with a visual indicator
//! similar to OpenVPN Connect's status display.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// VPN connection status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VpnConnectionStatus {
    /// Not connected to any VPN
    #[default]
    Disconnected,
    /// Currently establishing connection
    Connecting,
    /// Authenticating with the server
    Authenticating,
    /// Fully connected and active
    Connected,
    /// Gracefully disconnecting
    Disconnecting,
    /// Connection error occurred
    Error,
}

impl VpnConnectionStatus {
    /// Get the display label for this status.
    pub fn label(&self) -> &'static str {
        match self {
            VpnConnectionStatus::Disconnected => "Disconnected",
            VpnConnectionStatus::Connecting => "Connecting...",
            VpnConnectionStatus::Authenticating => "Authenticating...",
            VpnConnectionStatus::Connected => "Connected",
            VpnConnectionStatus::Disconnecting => "Disconnecting...",
            VpnConnectionStatus::Error => "Error",
        }
    }

    /// Get the color associated with this status.
    pub fn color(&self) -> Color {
        match self {
            VpnConnectionStatus::Disconnected => Color::rgb(0.5, 0.5, 0.5),
            VpnConnectionStatus::Connecting | VpnConnectionStatus::Authenticating => {
                Color::rgb(1.0, 0.757, 0.027) // Amber/warning
            }
            VpnConnectionStatus::Connected => Color::rgb(0.133, 0.773, 0.369), // Green
            VpnConnectionStatus::Disconnecting => Color::rgb(1.0, 0.757, 0.027),
            VpnConnectionStatus::Error => Color::rgb(0.937, 0.267, 0.267), // Red
        }
    }
}

/// VPN Status widget size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VpnStatusSize {
    /// Compact indicator (just the dot)
    Compact,
    /// Small status with icon and short text
    Small,
    /// Medium status (default)
    #[default]
    Medium,
    /// Large status with full details
    Large,
}

impl VpnStatusSize {
    fn indicator_size(&self) -> f32 {
        match self {
            VpnStatusSize::Compact => 12.0,
            VpnStatusSize::Small => 16.0,
            VpnStatusSize::Medium => 20.0,
            VpnStatusSize::Large => 28.0,
        }
    }

    fn font_size(&self) -> f32 {
        match self {
            VpnStatusSize::Compact => 0.0,
            VpnStatusSize::Small => 12.0,
            VpnStatusSize::Medium => 14.0,
            VpnStatusSize::Large => 18.0,
        }
    }
}

/// A VPN status indicator widget.
///
/// Displays the current VPN connection status with a colored indicator
/// and optional label text.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let status = VpnStatus::new()
///     .status(VpnConnectionStatus::Connected)
///     .size(VpnStatusSize::Large)
///     .show_label(true);
///
/// // With pulse animation when connecting
/// let connecting = VpnStatus::new()
///     .status(VpnConnectionStatus::Connecting)
///     .pulse(true);
/// ```
pub struct VpnStatus {
    base: WidgetBase,
    status: VpnConnectionStatus,
    size: VpnStatusSize,
    show_label: bool,
    server_name: Option<String>,
    pulse: bool,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl VpnStatus {
    /// Create a new VPN status widget.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("vpn-status"),
            status: VpnConnectionStatus::default(),
            size: VpnStatusSize::default(),
            show_label: true,
            server_name: None,
            pulse: true,
            on_click: None,
        }
    }

    /// Set the connection status.
    pub fn status(mut self, status: VpnConnectionStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the size variant.
    pub fn size(mut self, size: VpnStatusSize) -> Self {
        self.size = size;
        self
    }

    /// Set whether to show the status label.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Set the connected server name.
    pub fn server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = Some(name.into());
        self
    }

    /// Set whether to show pulse animation for transitional states.
    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the current status.
    pub fn get_status(&self) -> VpnConnectionStatus {
        self.status
    }

    /// Set the status programmatically.
    pub fn set_status(&mut self, status: VpnConnectionStatus) {
        self.status = status;
    }

    fn is_transitional(&self) -> bool {
        matches!(
            self.status,
            VpnConnectionStatus::Connecting
                | VpnConnectionStatus::Authenticating
                | VpnConnectionStatus::Disconnecting
        )
    }
}

impl Default for VpnStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for VpnStatus {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "vpn-status"
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
        let indicator_size = self.size.indicator_size();
        let font_size = self.size.font_size();

        if self.size == VpnStatusSize::Compact {
            return Size::new(indicator_size, indicator_size);
        }

        let label_width = if self.show_label {
            let status_label_len = self.status.label().len() as f32 * font_size * 0.6;
            let server_label_len = self
                .server_name
                .as_ref()
                .map(|n| n.len() as f32 * font_size * 0.5)
                .unwrap_or(0.0);
            status_label_len.max(server_label_len) + 16.0
        } else {
            0.0
        };

        let height = match self.size {
            VpnStatusSize::Large if self.server_name.is_some() => indicator_size + font_size + 8.0,
            _ => indicator_size.max(font_size * 1.5),
        };

        Size::new(indicator_size + label_width, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let indicator_size = self.size.indicator_size();
        let status_color = self.status.color();

        // Draw pulse glow for transitional states
        if self.pulse && self.is_transitional() {
            let glow_size = indicator_size * 1.6;
            let glow_rect = Rect::new(
                rect.x() + (indicator_size - glow_size) / 2.0,
                rect.y() + (rect.height() - glow_size) / 2.0,
                glow_size,
                glow_size,
            );
            painter.fill_rounded_rect(
                glow_rect,
                status_color.with_alpha(0.3),
                BorderRadius::all(glow_size / 2.0),
            );
        }

        // Draw status indicator circle
        let indicator_rect = Rect::new(
            rect.x(),
            rect.y() + (rect.height() - indicator_size) / 2.0,
            indicator_size,
            indicator_size,
        );
        painter.fill_rounded_rect(
            indicator_rect,
            status_color,
            BorderRadius::all(indicator_size / 2.0),
        );

        // Draw inner highlight
        let highlight_size = indicator_size * 0.6;
        let highlight_rect = Rect::new(
            indicator_rect.x() + (indicator_size - highlight_size) / 2.0,
            indicator_rect.y() + (indicator_size - highlight_size) / 2.0,
            highlight_size,
            highlight_size,
        );
        painter.fill_rounded_rect(
            highlight_rect,
            status_color.lighten(20.0),
            BorderRadius::all(highlight_size / 2.0),
        );

        // Draw labels
        if self.show_label && self.size != VpnStatusSize::Compact {
            let font_size = self.size.font_size();
            let text_x = rect.x() + indicator_size + 12.0;

            if self.size == VpnStatusSize::Large && self.server_name.is_some() {
                // Two-line layout for large size with server name
                let status_y = rect.y() + font_size;
                painter.draw_text(
                    self.status.label(),
                    Point::new(text_x, status_y),
                    theme.colors.foreground,
                    font_size,
                );

                if let Some(server) = &self.server_name {
                    let server_y = status_y + font_size + 4.0;
                    painter.draw_text(
                        server,
                        Point::new(text_x, server_y),
                        theme.colors.muted_foreground,
                        font_size * 0.85,
                    );
                }
            } else {
                // Single line layout
                let text_y = rect.y() + (rect.height() + font_size * 0.8) / 2.0;
                painter.draw_text(
                    self.status.label(),
                    Point::new(text_x, text_y),
                    theme.colors.foreground,
                    font_size,
                );
            }
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            let in_bounds = self.bounds().contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Move | MouseEventKind::Enter => {
                    if in_bounds && !self.base.state.hovered {
                        self.base.state.hovered = true;
                        ctx.request_redraw();
                    } else if !in_bounds && self.base.state.hovered {
                        self.base.state.hovered = false;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Leave => {
                    if self.base.state.hovered {
                        self.base.state.hovered = false;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) && in_bounds => {
                    if let Some(handler) = &self.on_click {
                        handler();
                    }
                    ctx.request_redraw();
                    return EventResult::Handled;
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
