//! Battery indicator widget
//!
//! Shows battery level and charging status.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Battery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatteryStatus {
    Discharging,
    Charging,
    #[default]
    Full,
    NotPresent,
    Unknown,
}

/// Power profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerProfile {
    PowerSaver,
    #[default]
    Balanced,
    Performance,
}

/// Battery information
#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub level: u8,
    pub status: BatteryStatus,
    pub time_remaining: Option<u64>,
    pub power_profile: PowerProfile,
    pub health: Option<u8>,
    pub battery_saver: bool,
}

/// Battery indicator widget
pub struct BatteryIndicator {
    base: WidgetBase,
    info: BatteryInfo,
    is_open: bool,
    icon_size: f32,
    popup_width: f32,
    low_threshold: u8,
    critical_threshold: u8,
    #[allow(clippy::type_complexity)]
    on_profile_change: Option<Box<dyn Fn(PowerProfile) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_battery_saver_toggle: Option<Box<dyn Fn(bool) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_settings: Option<Box<dyn Fn() + Send + Sync>>,
}

impl BatteryIndicator {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("battery-indicator"),
            info: BatteryInfo {
                level: 100,
                status: BatteryStatus::Full,
                ..Default::default()
            },
            is_open: false,
            icon_size: 24.0,
            popup_width: 360.0,
            low_threshold: 20,
            critical_threshold: 10,
            on_profile_change: None,
            on_battery_saver_toggle: None,
            on_settings: None,
        }
    }

    pub fn info(mut self, info: BatteryInfo) -> Self {
        self.info = info;
        self
    }

    pub fn level(mut self, level: u8) -> Self {
        self.info.level = level.min(100);
        self
    }

    pub fn status(mut self, status: BatteryStatus) -> Self {
        self.info.status = status;
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

    pub fn low_threshold(mut self, threshold: u8) -> Self {
        self.low_threshold = threshold;
        self
    }

    pub fn critical_threshold(mut self, threshold: u8) -> Self {
        self.critical_threshold = threshold;
        self
    }

    pub fn on_profile_change<F>(mut self, f: F) -> Self
    where
        F: Fn(PowerProfile) + Send + Sync + 'static,
    {
        self.on_profile_change = Some(Box::new(f));
        self
    }

    pub fn on_battery_saver_toggle<F>(mut self, f: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_battery_saver_toggle = Some(Box::new(f));
        self
    }

    pub fn on_settings<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_settings = Some(Box::new(f));
        self
    }

    pub fn set_info(&mut self, info: BatteryInfo) {
        self.info = info;
    }

    pub fn set_level(&mut self, level: u8) {
        self.info.level = level.min(100);
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

    pub fn is_low(&self) -> bool {
        self.info.level <= self.low_threshold && self.info.status == BatteryStatus::Discharging
    }

    pub fn is_critical(&self) -> bool {
        self.info.level <= self.critical_threshold && self.info.status == BatteryStatus::Discharging
    }

    pub fn get_icon(&self) -> &'static str {
        match self.info.status {
            BatteryStatus::Charging | BatteryStatus::Full => "🔌",
            BatteryStatus::NotPresent => "⚡",
            BatteryStatus::Unknown => "❓",
            BatteryStatus::Discharging => {
                if self.info.level <= 10 { "🪫" } else { "🔋" }
            }
        }
    }

    pub fn get_color(&self) -> Color {
        if self.is_critical() {
            Color::rgb(0.9, 0.2, 0.2)
        } else if self.is_low() {
            Color::rgb(0.9, 0.6, 0.2)
        } else if self.info.status == BatteryStatus::Charging {
            Color::rgb(0.2, 0.8, 0.4)
        } else {
            Color::WHITE
        }
    }

    pub fn format_time_remaining(&self) -> Option<String> {
        self.info.time_remaining.map(|secs| {
            let hours = secs / 3600;
            let minutes = (secs % 3600) / 60;
            if hours > 0 {
                format!("{}h {}m remaining", hours, minutes)
            } else {
                format!("{}m remaining", minutes)
            }
        })
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

impl Default for BatteryIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for BatteryIndicator {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "battery-indicator"
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
        Size::new(self.icon_size + 24.0, self.icon_size)
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

        // Battery outline
        let battery_width = 20.0;
        let battery_height = 10.0;
        let battery_x = rect.x() + 4.0;
        let battery_y = rect.y() + (rect.height() - battery_height) / 2.0;

        painter.stroke_rect(
            Rect::new(battery_x, battery_y, battery_width, battery_height),
            Color::WHITE,
            1.0,
        );

        // Battery tip
        let tip_rect = Rect::new(battery_x + battery_width, battery_y + 3.0, 2.0, 4.0);
        painter.fill_rect(tip_rect, Color::WHITE);

        // Battery fill
        let fill_width = (battery_width - 4.0) * (self.info.level as f32 / 100.0);
        let fill_rect = Rect::new(battery_x + 2.0, battery_y + 2.0, fill_width, battery_height - 4.0);
        painter.fill_rect(fill_rect, self.get_color());

        // Percentage text
        let percent = format!("{}%", self.info.level);
        painter.draw_text(
            &percent,
            Point::new(rect.x() + 28.0, rect.y() + rect.height() / 2.0 + 4.0),
            Color::WHITE,
            11.0,
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
