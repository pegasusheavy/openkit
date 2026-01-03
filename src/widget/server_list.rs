//! VPN Server list widget.
//!
//! Displays a list of VPN servers for selection, similar to OpenVPN Connect.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// VPN Server information.
#[derive(Debug, Clone)]
pub struct VpnServer {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Server hostname or IP
    pub host: String,
    /// Server port
    pub port: u16,
    /// Protocol (UDP/TCP)
    pub protocol: String,
    /// Country code (e.g., "US", "DE", "JP")
    pub country_code: Option<String>,
    /// City name
    pub city: Option<String>,
    /// Load percentage (0-100)
    pub load: Option<u8>,
    /// Latency in milliseconds
    pub latency_ms: Option<u32>,
    /// Whether this is a favorite server
    pub favorite: bool,
    /// Whether this server supports SSO (OAuth2/SAML)
    pub sso_enabled: bool,
}

impl VpnServer {
    /// Create a new VPN server entry.
    pub fn new(id: impl Into<String>, name: impl Into<String>, host: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            host: host.into(),
            port: 1194,
            protocol: "UDP".to_string(),
            country_code: None,
            city: None,
            load: None,
            latency_ms: None,
            favorite: false,
            sso_enabled: false,
        }
    }

    /// Set the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the protocol.
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = protocol.into();
        self
    }

    /// Set the country code.
    pub fn country(mut self, code: impl Into<String>) -> Self {
        self.country_code = Some(code.into());
        self
    }

    /// Set the city.
    pub fn city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    /// Set the load percentage.
    pub fn load(mut self, load: u8) -> Self {
        self.load = Some(load.min(100));
        self
    }

    /// Set the latency.
    pub fn latency(mut self, ms: u32) -> Self {
        self.latency_ms = Some(ms);
        self
    }

    /// Set as favorite.
    pub fn favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }

    /// Set SSO enabled.
    pub fn sso_enabled(mut self, enabled: bool) -> Self {
        self.sso_enabled = enabled;
        self
    }

    /// Get display location (city + country or just country or host).
    pub fn location(&self) -> String {
        match (&self.city, &self.country_code) {
            (Some(city), Some(country)) => format!("{}, {}", city, country),
            (None, Some(country)) => country.clone(),
            (Some(city), None) => city.clone(),
            (None, None) => self.host.clone(),
        }
    }

    /// Get load color based on percentage.
    pub fn load_color(&self) -> Color {
        match self.load {
            Some(l) if l < 50 => Color::rgb(0.133, 0.773, 0.369),  // Green
            Some(l) if l < 80 => Color::rgb(1.0, 0.757, 0.027),    // Amber
            Some(_) => Color::rgb(0.937, 0.267, 0.267),            // Red
            None => Color::rgb(0.5, 0.5, 0.5),                     // Gray
        }
    }
}

/// Internal state for a server item.
struct ServerItemState {
    hovered: bool,
}

/// VPN Server list widget.
///
/// Displays a scrollable list of VPN servers with selection support.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let servers = vec![
///     VpnServer::new("us-1", "US East", "vpn-us-east.example.com")
///         .country("US")
///         .city("New York")
///         .load(45),
///     VpnServer::new("de-1", "Germany", "vpn-de.example.com")
///         .country("DE")
///         .city("Frankfurt")
///         .load(72),
/// ];
///
/// let list = ServerList::new()
///     .servers(servers)
///     .selected("us-1")
///     .on_select(|server| println!("Selected: {}", server.name));
/// ```
pub struct ServerList {
    base: WidgetBase,
    servers: Vec<VpnServer>,
    selected_id: Option<String>,
    item_height: f32,
    show_load: bool,
    show_latency: bool,
    show_protocol: bool,
    item_states: Vec<ServerItemState>,
    on_select: Option<Box<dyn Fn(&VpnServer) + Send + Sync>>,
    on_favorite: Option<Box<dyn Fn(&VpnServer) + Send + Sync>>,
}

impl ServerList {
    /// Create a new server list widget.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("server-list"),
            servers: Vec::new(),
            selected_id: None,
            item_height: 64.0,
            show_load: true,
            show_latency: true,
            show_protocol: false,
            item_states: Vec::new(),
            on_select: None,
            on_favorite: None,
        }
    }

    /// Set the list of servers.
    pub fn servers(mut self, servers: Vec<VpnServer>) -> Self {
        self.item_states = servers.iter().map(|_| ServerItemState { hovered: false }).collect();
        self.servers = servers;
        self
    }

    /// Set the selected server ID.
    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    /// Set the item height.
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// Set whether to show server load.
    pub fn show_load(mut self, show: bool) -> Self {
        self.show_load = show;
        self
    }

    /// Set whether to show latency.
    pub fn show_latency(mut self, show: bool) -> Self {
        self.show_latency = show;
        self
    }

    /// Set whether to show protocol.
    pub fn show_protocol(mut self, show: bool) -> Self {
        self.show_protocol = show;
        self
    }

    /// Set the selection handler.
    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(&VpnServer) + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Set the favorite toggle handler.
    pub fn on_favorite<F>(mut self, handler: F) -> Self
    where
        F: Fn(&VpnServer) + Send + Sync + 'static,
    {
        self.on_favorite = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the selected server.
    pub fn get_selected(&self) -> Option<&VpnServer> {
        self.selected_id
            .as_ref()
            .and_then(|id| self.servers.iter().find(|s| &s.id == id))
    }

    /// Select a server by ID.
    pub fn select(&mut self, id: &str) {
        if self.servers.iter().any(|s| s.id == id) {
            self.selected_id = Some(id.to_string());
        }
    }

    /// Get item index at position.
    fn item_at_position(&self, y: f32, rect: Rect) -> Option<usize> {
        if y < rect.y() {
            return None;
        }
        let relative_y = y - rect.y();
        let index = (relative_y / self.item_height) as usize;
        if index < self.servers.len() {
            Some(index)
        } else {
            None
        }
    }

    fn paint_server_item(
        &self,
        painter: &mut Painter,
        server: &VpnServer,
        rect: Rect,
        is_selected: bool,
        is_hovered: bool,
        theme: &crate::theme::ThemeData,
    ) {
        // Background
        let bg_color = if is_selected {
            theme.colors.primary.with_alpha(0.15)
        } else if is_hovered {
            theme.colors.muted.with_alpha(0.5)
        } else {
            Color::TRANSPARENT
        };

        painter.fill_rect(rect, bg_color);

        // Selection indicator
        if is_selected {
            let indicator_rect = Rect::new(rect.x(), rect.y(), 4.0, rect.height());
            painter.fill_rect(indicator_rect, theme.colors.primary);
        }

        let content_x = rect.x() + 16.0;

        // Favorite star
        if server.favorite {
            let star_x = rect.x() + rect.width() - 32.0;
            let star_y = rect.y() + rect.height() / 2.0 - 6.0;
            painter.draw_text("★", Point::new(star_x, star_y + 12.0), Color::rgb(1.0, 0.757, 0.027), 14.0);
        }

        // SSO indicator
        if server.sso_enabled {
            let sso_x = rect.x() + rect.width() - (if server.favorite { 56.0 } else { 32.0 });
            let sso_y = rect.y() + rect.height() / 2.0 - 6.0;
            painter.draw_text("🔐", Point::new(sso_x, sso_y + 12.0), theme.colors.muted_foreground, 12.0);
        }

        // Server name
        let name_y = rect.y() + 24.0;
        painter.draw_text(
            &server.name,
            Point::new(content_x, name_y),
            theme.colors.foreground,
            16.0,
        );

        // Location
        let location_y = rect.y() + 44.0;
        painter.draw_text(
            &server.location(),
            Point::new(content_x, location_y),
            theme.colors.muted_foreground,
            12.0,
        );

        // Load indicator
        if self.show_load {
            if let Some(load) = server.load {
                let load_x = rect.x() + rect.width() - 100.0;
                let load_y = rect.y() + rect.height() / 2.0;
                let load_text = format!("{}%", load);

                // Load bar background
                let bar_width = 40.0;
                let bar_height = 4.0;
                let bar_rect = Rect::new(
                    load_x,
                    load_y - bar_height / 2.0,
                    bar_width,
                    bar_height,
                );
                painter.fill_rounded_rect(
                    bar_rect,
                    theme.colors.muted,
                    BorderRadius::all(bar_height / 2.0),
                );

                // Load bar fill
                let fill_width = bar_width * (load as f32 / 100.0);
                let fill_rect = Rect::new(
                    load_x,
                    load_y - bar_height / 2.0,
                    fill_width,
                    bar_height,
                );
                painter.fill_rounded_rect(
                    fill_rect,
                    server.load_color(),
                    BorderRadius::all(bar_height / 2.0),
                );

                // Load text
                painter.draw_text(
                    &load_text,
                    Point::new(load_x + bar_width + 8.0, load_y + 4.0),
                    theme.colors.muted_foreground,
                    11.0,
                );
            }
        }

        // Latency
        if self.show_latency {
            if let Some(latency) = server.latency_ms {
                let latency_x = content_x + 200.0;
                let latency_y = rect.y() + 44.0;
                let latency_text = format!("{}ms", latency);
                let latency_color = match latency {
                    l if l < 50 => Color::rgb(0.133, 0.773, 0.369),
                    l if l < 100 => Color::rgb(1.0, 0.757, 0.027),
                    _ => Color::rgb(0.937, 0.267, 0.267),
                };
                painter.draw_text(
                    &latency_text,
                    Point::new(latency_x, latency_y),
                    latency_color,
                    11.0,
                );
            }
        }

        // Bottom separator
        let separator_rect = Rect::new(
            rect.x() + 16.0,
            rect.y() + rect.height() - 1.0,
            rect.width() - 32.0,
            1.0,
        );
        painter.fill_rect(separator_rect, theme.colors.border.with_alpha(0.3));
    }
}

impl Default for ServerList {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ServerList {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "server-list"
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
        let height = self.servers.len() as f32 * self.item_height;
        Size::new(300.0, height.max(100.0))
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let intrinsic = self.intrinsic_size(ctx);
        let size = Size::new(
            constraints.max_width.min(intrinsic.width.max(constraints.min_width)),
            constraints.max_height.min(intrinsic.height.max(constraints.min_height)),
        );
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        painter.fill_rect(rect, theme.colors.card);

        // Paint each server item
        for (i, server) in self.servers.iter().enumerate() {
            let item_rect = Rect::new(
                rect.x(),
                rect.y() + (i as f32 * self.item_height),
                rect.width(),
                self.item_height,
            );

            // Skip items outside visible area
            if item_rect.y() > rect.y() + rect.height() {
                break;
            }
            if item_rect.y() + item_rect.height() < rect.y() {
                continue;
            }

            let is_selected = self.selected_id.as_ref() == Some(&server.id);
            let is_hovered = self.item_states.get(i).map(|s| s.hovered).unwrap_or(false);

            self.paint_server_item(painter, server, item_rect, is_selected, is_hovered, theme);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            let rect = self.bounds();
            let in_bounds = rect.contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Move | MouseEventKind::Enter => {
                    // Update hover state for items
                    let hovered_index = if in_bounds {
                        self.item_at_position(mouse.position.y, rect)
                    } else {
                        None
                    };

                    let mut changed = false;
                    for (i, state) in self.item_states.iter_mut().enumerate() {
                        let should_hover = hovered_index == Some(i);
                        if state.hovered != should_hover {
                            state.hovered = should_hover;
                            changed = true;
                        }
                    }

                    if changed {
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Leave => {
                    for state in self.item_states.iter_mut() {
                        state.hovered = false;
                    }
                    ctx.request_redraw();
                }
                MouseEventKind::Up if mouse.button == Some(MouseButton::Left) && in_bounds => {
                    if let Some(index) = self.item_at_position(mouse.position.y, rect) {
                        if let Some(server) = self.servers.get(index) {
                            self.selected_id = Some(server.id.clone());
                            if let Some(handler) = &self.on_select {
                                handler(server);
                            }
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
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
