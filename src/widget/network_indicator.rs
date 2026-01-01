//! Network indicator widget
//!
//! Shows network connection status with WiFi/Ethernet details.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Network connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkType {
    Wifi,
    Ethernet,
    Cellular,
    Vpn,
    #[default]
    Disconnected,
}

/// WiFi network info
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_strength: u8,
    pub is_secured: bool,
    pub is_connected: bool,
    pub frequency: Option<String>,
}

/// Network status
#[derive(Debug, Clone, Default)]
pub struct NetworkStatus {
    pub connection_type: NetworkType,
    pub is_connected: bool,
    pub name: Option<String>,
    pub ip_address: Option<String>,
    pub signal_strength: Option<u8>,
    pub upload_speed: Option<u64>,
    pub download_speed: Option<u64>,
}

/// Network indicator widget
pub struct NetworkIndicator {
    base: WidgetBase,
    status: NetworkStatus,
    available_networks: Vec<WifiNetwork>,
    is_open: bool,
    show_networks: bool,
    icon_size: f32,
    popup_width: f32,
    #[allow(clippy::type_complexity)]
    on_network_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_disconnect: Option<Box<dyn Fn() + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_settings: Option<Box<dyn Fn() + Send + Sync>>,
}

impl NetworkIndicator {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("network-indicator"),
            status: NetworkStatus::default(),
            available_networks: Vec::new(),
            is_open: false,
            show_networks: true,
            icon_size: 24.0,
            popup_width: 360.0,
            on_network_select: None,
            on_disconnect: None,
            on_settings: None,
        }
    }

    pub fn status(mut self, status: NetworkStatus) -> Self {
        self.status = status;
        self
    }

    pub fn networks(mut self, networks: Vec<WifiNetwork>) -> Self {
        self.available_networks = networks;
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn popup_width(mut self, width: f32) -> Self {
        self.popup_width = width;
        self
    }

    pub fn show_networks(mut self, show: bool) -> Self {
        self.show_networks = show;
        self
    }

    pub fn on_network_select<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_network_select = Some(Box::new(f));
        self
    }

    pub fn on_disconnect<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_disconnect = Some(Box::new(f));
        self
    }

    pub fn on_settings<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_settings = Some(Box::new(f));
        self
    }

    pub fn set_status(&mut self, status: NetworkStatus) {
        self.status = status;
    }

    pub fn set_networks(&mut self, networks: Vec<WifiNetwork>) {
        self.available_networks = networks;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn get_icon(&self) -> &'static str {
        match self.status.connection_type {
            NetworkType::Wifi => "📶",
            NetworkType::Ethernet => "🔌",
            NetworkType::Cellular => "📱",
            NetworkType::Vpn => "🔒",
            NetworkType::Disconnected => "❌",
        }
    }

    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }
}

impl Default for NetworkIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for NetworkIndicator {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "network-indicator"
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
        Size::new(self.icon_size, self.icon_size)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        // Background on hover
        if self.is_open {
            painter.fill_rounded_rect(rect, Color::rgba(1.0, 1.0, 1.0, 0.1), BorderRadius::all(4.0));
        }

        // Icon
        painter.draw_text(
            self.get_icon(),
            Point::new(rect.x() + 2.0, rect.y() + self.icon_size - 4.0),
            Color::WHITE,
            self.icon_size - 4.0,
        );
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
