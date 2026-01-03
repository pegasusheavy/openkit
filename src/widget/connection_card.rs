//! VPN Connection Card widget.
//!
//! A comprehensive card displaying VPN connection information and controls,
//! similar to OpenVPN Connect's main view.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use super::vpn_status::VpnConnectionStatus;
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// VPN Connection statistics.
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Bytes downloaded
    pub bytes_rx: u64,
    /// Bytes uploaded
    pub bytes_tx: u64,
    /// Connection duration in seconds
    pub duration_secs: u64,
    /// Current download speed (bytes/sec)
    pub speed_rx: u64,
    /// Current upload speed (bytes/sec)
    pub speed_tx: u64,
    /// VPN IP address
    pub vpn_ip: Option<String>,
    /// Real IP address
    pub real_ip: Option<String>,
    /// Server location
    pub server_location: Option<String>,
}

impl ConnectionStats {
    /// Format bytes to human readable string.
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Format duration to human readable string.
    pub fn format_duration(secs: u64) -> String {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    /// Format speed to human readable string.
    pub fn format_speed(bytes_per_sec: u64) -> String {
        let bits_per_sec = bytes_per_sec * 8;
        const KBPS: u64 = 1000;
        const MBPS: u64 = KBPS * 1000;
        const GBPS: u64 = MBPS * 1000;

        if bits_per_sec >= GBPS {
            format!("{:.2} Gbps", bits_per_sec as f64 / GBPS as f64)
        } else if bits_per_sec >= MBPS {
            format!("{:.2} Mbps", bits_per_sec as f64 / MBPS as f64)
        } else if bits_per_sec >= KBPS {
            format!("{:.2} Kbps", bits_per_sec as f64 / KBPS as f64)
        } else {
            format!("{} bps", bits_per_sec)
        }
    }
}

/// Authentication method for the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMethod {
    /// Username/Password authentication
    #[default]
    UsernamePassword,
    /// Certificate-based authentication
    Certificate,
    /// OAuth2/OIDC authentication
    OAuth2,
    /// SAML authentication
    Saml,
}

impl AuthMethod {
    /// Get the display name for this auth method.
    pub fn label(&self) -> &'static str {
        match self {
            AuthMethod::UsernamePassword => "Username/Password",
            AuthMethod::Certificate => "Certificate",
            AuthMethod::OAuth2 => "OAuth2 SSO",
            AuthMethod::Saml => "SAML SSO",
        }
    }

    /// Check if this method requires a password input.
    pub fn requires_password(&self) -> bool {
        matches!(self, AuthMethod::UsernamePassword)
    }

    /// Check if this method uses SSO.
    pub fn is_sso(&self) -> bool {
        matches!(self, AuthMethod::OAuth2 | AuthMethod::Saml)
    }
}

/// VPN Connection Card widget.
///
/// Displays connection status, controls, and statistics in a card format.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let card = ConnectionCard::new()
///     .status(VpnConnectionStatus::Connected)
///     .server_name("US East - New York")
///     .auth_method(AuthMethod::OAuth2)
///     .stats(ConnectionStats {
///         bytes_rx: 1024 * 1024 * 150,
///         bytes_tx: 1024 * 1024 * 25,
///         duration_secs: 3600,
///         ..Default::default()
///     })
///     .on_connect(|| println!("Connect clicked"))
///     .on_disconnect(|| println!("Disconnect clicked"));
/// ```
pub struct ConnectionCard {
    base: WidgetBase,
    status: VpnConnectionStatus,
    server_name: Option<String>,
    server_host: Option<String>,
    auth_method: AuthMethod,
    stats: ConnectionStats,
    show_stats: bool,
    compact: bool,
    connect_button_hovered: bool,
    on_connect: Option<Box<dyn Fn() + Send + Sync>>,
    on_disconnect: Option<Box<dyn Fn() + Send + Sync>>,
    on_settings: Option<Box<dyn Fn() + Send + Sync>>,
}

impl ConnectionCard {
    /// Create a new connection card.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("connection-card"),
            status: VpnConnectionStatus::default(),
            server_name: None,
            server_host: None,
            auth_method: AuthMethod::default(),
            stats: ConnectionStats::default(),
            show_stats: true,
            compact: false,
            connect_button_hovered: false,
            on_connect: None,
            on_disconnect: None,
            on_settings: None,
        }
    }

    /// Set the connection status.
    pub fn status(mut self, status: VpnConnectionStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the server name.
    pub fn server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = Some(name.into());
        self
    }

    /// Set the server host.
    pub fn server_host(mut self, host: impl Into<String>) -> Self {
        self.server_host = Some(host.into());
        self
    }

    /// Set the authentication method.
    pub fn auth_method(mut self, method: AuthMethod) -> Self {
        self.auth_method = method;
        self
    }

    /// Set the connection statistics.
    pub fn stats(mut self, stats: ConnectionStats) -> Self {
        self.stats = stats;
        self
    }

    /// Set whether to show statistics.
    pub fn show_stats(mut self, show: bool) -> Self {
        self.show_stats = show;
        self
    }

    /// Set compact mode.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Set the connect handler.
    pub fn on_connect<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_connect = Some(Box::new(handler));
        self
    }

    /// Set the disconnect handler.
    pub fn on_disconnect<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_disconnect = Some(Box::new(handler));
        self
    }

    /// Set the settings handler.
    pub fn on_settings<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_settings = Some(Box::new(handler));
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

    /// Update stats programmatically.
    pub fn update_stats(&mut self, stats: ConnectionStats) {
        self.stats = stats;
    }

    fn is_connected(&self) -> bool {
        self.status == VpnConnectionStatus::Connected
    }

    fn is_connecting(&self) -> bool {
        matches!(
            self.status,
            VpnConnectionStatus::Connecting
                | VpnConnectionStatus::Authenticating
                | VpnConnectionStatus::Disconnecting
        )
    }

    fn get_button_rect(&self, card_rect: Rect) -> Rect {
        let button_width = if self.compact { 100.0 } else { 180.0 };
        let button_height = 48.0;
        let button_x = card_rect.x() + (card_rect.width() - button_width) / 2.0;
        let button_y = if self.compact {
            card_rect.y() + card_rect.height() - button_height - 20.0
        } else {
            card_rect.y() + 200.0
        };
        Rect::new(button_x, button_y, button_width, button_height)
    }

    fn paint_status_indicator(&self, painter: &mut Painter, rect: Rect, _theme: &crate::theme::ThemeData) {
        let status_color = self.status.color();
        let indicator_size = 80.0;

        // Outer glow
        if self.is_connecting() {
            let glow_size = indicator_size * 1.4;
            let glow_rect = Rect::new(
                rect.x() + (rect.width() - glow_size) / 2.0,
                rect.y() + 30.0,
                glow_size,
                glow_size,
            );
            painter.fill_rounded_rect(
                glow_rect,
                status_color.with_alpha(0.2),
                BorderRadius::all(glow_size / 2.0),
            );
        }

        // Main indicator
        let indicator_rect = Rect::new(
            rect.x() + (rect.width() - indicator_size) / 2.0,
            rect.y() + 30.0 + (80.0 - indicator_size) / 2.0 * (if self.is_connecting() { 1.4 } else { 1.0 }),
            indicator_size,
            indicator_size,
        );
        painter.fill_rounded_rect(
            indicator_rect,
            status_color,
            BorderRadius::all(indicator_size / 2.0),
        );

        // Inner highlight
        let highlight_size = indicator_size * 0.6;
        let highlight_rect = Rect::new(
            indicator_rect.x() + (indicator_size - highlight_size) / 2.0,
            indicator_rect.y() + (indicator_size - highlight_size) / 2.0,
            highlight_size,
            highlight_size,
        );
        painter.fill_rounded_rect(
            highlight_rect,
            status_color.lighten(30.0),
            BorderRadius::all(highlight_size / 2.0),
        );

        // Shield icon (simplified as text for now)
        let icon_size = 28.0;
        let icon_x = indicator_rect.x() + (indicator_size - icon_size * 0.6) / 2.0;
        let icon_y = indicator_rect.y() + (indicator_size + icon_size * 0.8) / 2.0;
        let icon = if self.is_connected() { "🛡️" } else { "⚪" };
        painter.draw_text(icon, Point::new(icon_x, icon_y), Color::WHITE, icon_size);
    }

    fn paint_stats(&self, painter: &mut Painter, rect: Rect, theme: &crate::theme::ThemeData) {
        let stats_y = rect.y() + 280.0;
        let col_width = rect.width() / 3.0;

        // Download stats
        let dl_x = rect.x() + col_width / 2.0;
        painter.draw_text("↓", Point::new(dl_x - 20.0, stats_y + 14.0), Color::rgb(0.133, 0.773, 0.369), 16.0);
        painter.draw_text(
            &ConnectionStats::format_bytes(self.stats.bytes_rx),
            Point::new(dl_x, stats_y + 14.0),
            theme.colors.foreground,
            14.0,
        );
        painter.draw_text(
            &ConnectionStats::format_speed(self.stats.speed_rx),
            Point::new(dl_x, stats_y + 32.0),
            theme.colors.muted_foreground,
            11.0,
        );

        // Upload stats
        let ul_x = rect.x() + col_width + col_width / 2.0;
        painter.draw_text("↑", Point::new(ul_x - 20.0, stats_y + 14.0), Color::rgb(0.937, 0.267, 0.267), 16.0);
        painter.draw_text(
            &ConnectionStats::format_bytes(self.stats.bytes_tx),
            Point::new(ul_x, stats_y + 14.0),
            theme.colors.foreground,
            14.0,
        );
        painter.draw_text(
            &ConnectionStats::format_speed(self.stats.speed_tx),
            Point::new(ul_x, stats_y + 32.0),
            theme.colors.muted_foreground,
            11.0,
        );

        // Duration
        let dur_x = rect.x() + col_width * 2.0 + col_width / 2.0;
        painter.draw_text("⏱", Point::new(dur_x - 20.0, stats_y + 14.0), theme.colors.muted_foreground, 14.0);
        painter.draw_text(
            &ConnectionStats::format_duration(self.stats.duration_secs),
            Point::new(dur_x, stats_y + 14.0),
            theme.colors.foreground,
            14.0,
        );
    }
}

impl Default for ConnectionCard {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ConnectionCard {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "connection-card"
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
        if self.compact {
            Size::new(280.0, 200.0)
        } else {
            let height = if self.show_stats && self.is_connected() { 360.0 } else { 300.0 };
            Size::new(340.0, height)
        }
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Card background
        let radius = BorderRadius::all(16.0);
        painter.fill_rounded_rect(rect, theme.colors.card, radius);
        painter.stroke_rect(rect, theme.colors.border, 1.0);

        // Status indicator (large circle)
        if !self.compact {
            self.paint_status_indicator(painter, rect, theme);
        }

        // Status text
        let status_text_y = if self.compact { rect.y() + 30.0 } else { rect.y() + 140.0 };
        let status_text_width = self.status.label().len() as f32 * 10.0;
        let status_text_x = rect.x() + (rect.width() - status_text_width) / 2.0;
        painter.draw_text(
            self.status.label(),
            Point::new(status_text_x, status_text_y),
            theme.colors.foreground,
            18.0,
        );

        // Server info
        if let Some(ref name) = self.server_name {
            let server_y = status_text_y + 28.0;
            let server_width = name.len() as f32 * 8.0;
            let server_x = rect.x() + (rect.width() - server_width) / 2.0;
            painter.draw_text(
                name,
                Point::new(server_x, server_y),
                theme.colors.muted_foreground,
                14.0,
            );
        }

        // Auth method indicator (for SSO)
        if self.auth_method.is_sso() && !self.is_connected() {
            let auth_y = status_text_y + 50.0;
            let auth_label = format!("🔐 {}", self.auth_method.label());
            let auth_width = auth_label.len() as f32 * 6.0;
            let auth_x = rect.x() + (rect.width() - auth_width) / 2.0;
            painter.draw_text(
                &auth_label,
                Point::new(auth_x, auth_y),
                theme.colors.primary,
                12.0,
            );
        }

        // Connect/Disconnect button
        let button_rect = self.get_button_rect(rect);
        let (button_text, button_bg, button_fg) = if self.is_connected() {
            ("Disconnect", theme.colors.destructive, theme.colors.destructive_foreground)
        } else if self.is_connecting() {
            ("Cancel", theme.colors.muted, theme.colors.muted_foreground)
        } else {
            ("Connect", theme.colors.primary, theme.colors.primary_foreground)
        };

        let button_color = if self.connect_button_hovered {
            button_bg.darken(10.0)
        } else {
            button_bg
        };

        painter.fill_rounded_rect(
            button_rect,
            button_color,
            BorderRadius::all(button_rect.height() / 2.0),
        );

        let text_width = button_text.len() as f32 * 10.0;
        let text_x = button_rect.x() + (button_rect.width() - text_width) / 2.0;
        let text_y = button_rect.y() + (button_rect.height() + 14.0) / 2.0;
        painter.draw_text(button_text, Point::new(text_x, text_y), button_fg, 16.0);

        // Connection stats (when connected)
        if self.show_stats && self.is_connected() && !self.compact {
            self.paint_stats(painter, rect, theme);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            let rect = self.bounds();
            let button_rect = self.get_button_rect(rect);
            let in_button = button_rect.contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Move | MouseEventKind::Enter => {
                    if self.connect_button_hovered != in_button {
                        self.connect_button_hovered = in_button;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Leave => {
                    if self.connect_button_hovered {
                        self.connect_button_hovered = false;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) && in_button => {
                    if self.is_connected() {
                        if let Some(handler) = &self.on_disconnect {
                            handler();
                        }
                    } else if !self.is_connecting() {
                        if let Some(handler) = &self.on_connect {
                            handler();
                        }
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
