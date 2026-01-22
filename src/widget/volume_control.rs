//! Volume control widget
//!
//! A popup volume slider with mute toggle and device selection.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Audio output device
#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: AudioDeviceType,
    pub is_default: bool,
}

/// Audio device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDeviceType {
    Speakers,
    Headphones,
    Bluetooth,
    Hdmi,
    Usb,
    Other,
}

/// Volume control widget
pub struct VolumeControl {
    base: WidgetBase,
    volume: f32,
    is_muted: bool,
    devices: Vec<AudioDevice>,
    current_device: Option<String>,
    is_open: bool,
    show_devices: bool,
    width: f32,
    height: f32,
    accent_color: Color,
    #[allow(clippy::type_complexity)]
    on_volume_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_mute_toggle: Option<Box<dyn Fn(bool) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_device_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl VolumeControl {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("volume-control"),
            volume: 0.5,
            is_muted: false,
            devices: Vec::new(),
            current_device: None,
            is_open: false,
            show_devices: true,
            width: 300.0,
            height: 48.0,
            accent_color: Color::rgb(0.0, 0.47, 0.84),
            on_volume_change: None,
            on_mute_toggle: None,
            on_device_change: None,
        }
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    pub fn muted(mut self, muted: bool) -> Self {
        self.is_muted = muted;
        self
    }

    pub fn devices(mut self, devices: Vec<AudioDevice>) -> Self {
        self.devices = devices;
        self
    }

    pub fn current_device(mut self, device_id: impl Into<String>) -> Self {
        self.current_device = Some(device_id.into());
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    pub fn show_devices(mut self, show: bool) -> Self {
        self.show_devices = show;
        self
    }

    pub fn on_volume_change<F>(mut self, f: F) -> Self
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_volume_change = Some(Box::new(f));
        self
    }

    pub fn on_mute_toggle<F>(mut self, f: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_mute_toggle = Some(Box::new(f));
        self
    }

    pub fn on_device_change<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_device_change = Some(Box::new(f));
        self
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(ref cb) = self.on_volume_change {
            cb(self.volume);
        }
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
        if let Some(ref cb) = self.on_mute_toggle {
            cb(self.is_muted);
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.is_muted = muted;
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
        if self.is_muted {
            "🔇"
        } else if self.volume == 0.0 {
            "🔈"
        } else if self.volume < 0.5 {
            "🔉"
        } else {
            "🔊"
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

impl Default for VolumeControl {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for VolumeControl {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "volume-control"
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
        let mut height = self.height;
        if self.is_open && self.show_devices && !self.devices.is_empty() {
            height += self.devices.len() as f32 * 40.0;
        }
        Size::new(self.width, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        // Background
        painter.fill_rounded_rect(rect, Color::rgba(0.15, 0.15, 0.15, 0.95), BorderRadius::all(8.0));

        // Volume icon
        painter.draw_text(
            self.get_icon(),
            Point::new(rect.x() + 16.0, rect.y() + 30.0),
            Color::WHITE,
            20.0,
        );

        // Volume slider track
        let track_y = rect.y() + 24.0;
        let track_rect = Rect::new(rect.x() + 48.0, track_y - 2.0, rect.width() - 64.0, 4.0);
        painter.fill_rounded_rect(track_rect, Color::rgba(1.0, 1.0, 1.0, 0.2), BorderRadius::all(2.0));

        // Volume slider fill
        let fill_width = (rect.width() - 64.0) * self.volume;
        let fill_rect = Rect::new(rect.x() + 48.0, track_y - 2.0, fill_width, 4.0);
        painter.fill_rounded_rect(fill_rect, self.accent_color, BorderRadius::all(2.0));

        // Slider thumb
        let thumb_x = rect.x() + 48.0 + fill_width - 6.0;
        let thumb_rect = Rect::new(thumb_x, track_y - 6.0, 12.0, 12.0);
        painter.fill_rounded_rect(thumb_rect, Color::WHITE, BorderRadius::all(6.0));

        // Volume percentage
        let percent = format!("{}%", (self.volume * 100.0) as i32);
        painter.draw_text(
            &percent,
            Point::new(rect.max_x() - 48.0, rect.y() + 30.0),
            Color::WHITE,
            14.0,
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
