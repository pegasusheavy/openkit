//! Start Button widget
//!
//! The iconic start button that opens the start menu.
//! Supports various styles including Windows logo and custom icons.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseButton, MouseEvent, MouseEventKind};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Start button icon style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StartButtonStyle {
    /// Windows-style four-square logo
    #[default]
    WindowsLogo,
    /// Simple hamburger menu icon
    HamburgerMenu,
    /// Grid of dots (app grid style)
    AppGrid,
    /// Custom icon (set via icon path)
    CustomIcon,
    /// Text label (e.g., "Start")
    TextLabel,
    /// Circle with logo
    CircleLogo,
}

/// Animation state for the start button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StartButtonState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Active,
}

/// Start Button widget - the main entry point to the start menu
pub struct StartButton {
    base: WidgetBase,
    style: StartButtonStyle,
    state: StartButtonState,
    is_menu_open: bool,

    // Sizing
    size: f32,
    padding: f32,

    // Colors
    icon_color: Color,
    hover_color: Color,
    active_color: Color,
    background_color: Color,

    // Custom content
    icon_path: Option<String>,
    label: String,

    // Animation
    hover_scale: f32,
    press_scale: f32,
    animate_on_hover: bool,
    glow_enabled: bool,

    // Callbacks
    #[allow(clippy::type_complexity)]
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_menu_toggle: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl StartButton {
    /// Create a new start button with default Windows-style logo
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("start-button"),
            style: StartButtonStyle::WindowsLogo,
            state: StartButtonState::Normal,
            is_menu_open: false,
            size: 48.0,
            padding: 12.0,
            icon_color: Color::WHITE,
            hover_color: Color::rgba(1.0, 1.0, 1.0, 0.1),
            active_color: Color::rgba(1.0, 1.0, 1.0, 0.2),
            background_color: Color::TRANSPARENT,
            icon_path: None,
            label: "Start".to_string(),
            hover_scale: 1.05,
            press_scale: 0.95,
            animate_on_hover: true,
            glow_enabled: false,
            on_click: None,
            on_menu_toggle: None,
        }
    }

    /// Set the button style
    pub fn style(mut self, style: StartButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the button size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set internal padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set the icon color
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = color;
        self
    }

    /// Set the hover background color
    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }

    /// Set the active/pressed background color
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// Set the background color
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Set a custom icon path
    pub fn icon(mut self, path: impl Into<String>) -> Self {
        self.icon_path = Some(path.into());
        self.style = StartButtonStyle::CustomIcon;
        self
    }

    /// Set the text label (for TextLabel style)
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Enable or disable hover animation
    pub fn animate_hover(mut self, enabled: bool) -> Self {
        self.animate_on_hover = enabled;
        self
    }

    /// Set hover scale factor
    pub fn hover_scale(mut self, scale: f32) -> Self {
        self.hover_scale = scale;
        self
    }

    /// Enable glow effect
    pub fn glow(mut self, enabled: bool) -> Self {
        self.glow_enabled = enabled;
        self
    }

    /// Set click callback
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Set menu toggle callback
    pub fn on_menu_toggle<F>(mut self, f: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_menu_toggle = Some(Box::new(f));
        self
    }

    /// Toggle the menu state
    pub fn toggle_menu(&mut self) {
        self.is_menu_open = !self.is_menu_open;
        self.state = if self.is_menu_open {
            StartButtonState::Active
        } else {
            StartButtonState::Normal
        };
        if let Some(ref cb) = self.on_menu_toggle {
            cb(self.is_menu_open);
        }
    }

    /// Set menu open state
    pub fn set_menu_open(&mut self, open: bool) {
        self.is_menu_open = open;
        self.state = if open {
            StartButtonState::Active
        } else {
            StartButtonState::Normal
        };
    }

    /// Check if menu is open
    pub fn is_menu_open(&self) -> bool {
        self.is_menu_open
    }

    /// Add a CSS class
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Set element ID
    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }

    /// Draw the Windows-style four-square logo
    fn draw_windows_logo(&self, painter: &mut Painter, center: Point, size: f32) {
        let square_size = size * 0.4;
        let gap = size * 0.1;
        let half_gap = gap / 2.0;

        // Top-left square
        let tl = Rect::new(
            center.x - square_size - half_gap,
            center.y - square_size - half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(tl, self.icon_color);

        // Top-right square
        let tr = Rect::new(
            center.x + half_gap,
            center.y - square_size - half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(tr, self.icon_color);

        // Bottom-left square
        let bl = Rect::new(
            center.x - square_size - half_gap,
            center.y + half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(bl, self.icon_color);

        // Bottom-right square
        let br = Rect::new(
            center.x + half_gap,
            center.y + half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(br, self.icon_color);
    }

    /// Draw hamburger menu icon
    fn draw_hamburger(&self, painter: &mut Painter, center: Point, size: f32) {
        let bar_height = size * 0.12;
        let bar_width = size * 0.7;
        let spacing = size * 0.25;

        for i in -1..=1 {
            let y = center.y + (i as f32) * spacing - bar_height / 2.0;
            let bar = Rect::new(
                center.x - bar_width / 2.0,
                y,
                bar_width,
                bar_height,
            );
            painter.fill_rounded_rect(bar, self.icon_color, BorderRadius::all(bar_height / 2.0));
        }
    }

    /// Draw app grid icon (3x3 dots)
    fn draw_app_grid(&self, painter: &mut Painter, center: Point, size: f32) {
        let dot_size = size * 0.15;
        let spacing = size * 0.3;

        for row in -1..=1 {
            for col in -1..=1 {
                let x = center.x + (col as f32) * spacing - dot_size / 2.0;
                let y = center.y + (row as f32) * spacing - dot_size / 2.0;
                let dot = Rect::new(x, y, dot_size, dot_size);
                painter.fill_rounded_rect(dot, self.icon_color, BorderRadius::all(dot_size / 2.0));
            }
        }
    }

    /// Draw circle logo
    fn draw_circle_logo(&self, painter: &mut Painter, center: Point, size: f32) {
        // Outer circle
        let outer = Rect::new(
            center.x - size / 2.0,
            center.y - size / 2.0,
            size,
            size,
        );
        painter.fill_rounded_rect(outer, self.icon_color, BorderRadius::all(size / 2.0));

        // Inner logo (scaled down)
        let inner_size = size * 0.6;
        let inner_center = center;

        // Draw mini windows logo inside
        let square_size = inner_size * 0.35;
        let gap = inner_size * 0.08;
        let half_gap = gap / 2.0;

        let bg_color = Color::rgba(0.1, 0.1, 0.1, 1.0);

        let tl = Rect::new(
            inner_center.x - square_size - half_gap,
            inner_center.y - square_size - half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(tl, bg_color);

        let tr = Rect::new(
            inner_center.x + half_gap,
            inner_center.y - square_size - half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(tr, bg_color);

        let bl = Rect::new(
            inner_center.x - square_size - half_gap,
            inner_center.y + half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(bl, bg_color);

        let br = Rect::new(
            inner_center.x + half_gap,
            inner_center.y + half_gap,
            square_size,
            square_size,
        );
        painter.fill_rect(br, bg_color);
    }
}

impl Default for StartButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for StartButton {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "start-button"
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
        Size::new(self.size, self.size)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        // Calculate scale based on state
        let scale = match self.state {
            StartButtonState::Normal => 1.0,
            StartButtonState::Hovered if self.animate_on_hover => self.hover_scale,
            StartButtonState::Pressed => self.press_scale,
            StartButtonState::Active => 1.0,
            _ => 1.0,
        };

        // Scaled rect
        let scaled_size = rect.width() * scale;
        let offset = (rect.width() - scaled_size) / 2.0;
        let scaled_rect = Rect::new(
            rect.x() + offset,
            rect.y() + offset,
            scaled_size,
            scaled_size,
        );

        // Glow effect
        if self.glow_enabled && (self.state == StartButtonState::Hovered || self.state == StartButtonState::Active) {
            let glow_color = Color::rgba(
                self.icon_color.r,
                self.icon_color.g,
                self.icon_color.b,
                0.3,
            );
            let glow_rect = Rect::new(
                scaled_rect.x() - 4.0,
                scaled_rect.y() - 4.0,
                scaled_rect.width() + 8.0,
                scaled_rect.height() + 8.0,
            );
            painter.fill_rounded_rect(glow_rect, glow_color, BorderRadius::all(8.0));
        }

        // Background
        let bg_color = match self.state {
            StartButtonState::Normal => self.background_color,
            StartButtonState::Hovered => self.hover_color,
            StartButtonState::Pressed | StartButtonState::Active => self.active_color,
        };
        painter.fill_rounded_rect(scaled_rect, bg_color, BorderRadius::all(6.0));

        // Icon/content
        let center = Point::new(
            scaled_rect.x() + scaled_rect.width() / 2.0,
            scaled_rect.y() + scaled_rect.height() / 2.0,
        );
        let icon_size = scaled_rect.width() - self.padding * 2.0;

        match self.style {
            StartButtonStyle::WindowsLogo => {
                self.draw_windows_logo(painter, center, icon_size);
            }
            StartButtonStyle::HamburgerMenu => {
                self.draw_hamburger(painter, center, icon_size);
            }
            StartButtonStyle::AppGrid => {
                self.draw_app_grid(painter, center, icon_size);
            }
            StartButtonStyle::CircleLogo => {
                self.draw_circle_logo(painter, center, icon_size);
            }
            StartButtonStyle::TextLabel => {
                painter.draw_text(
                    &self.label,
                    Point::new(scaled_rect.x() + 8.0, center.y + 5.0),
                    self.icon_color,
                    14.0,
                );
            }
            StartButtonStyle::CustomIcon => {
                // For custom icons, draw a placeholder (actual icon loading would be separate)
                if self.icon_path.is_some() {
                    let icon_rect = Rect::new(
                        center.x - icon_size / 2.0,
                        center.y - icon_size / 2.0,
                        icon_size,
                        icon_size,
                    );
                    painter.fill_rounded_rect(icon_rect, self.icon_color, BorderRadius::all(4.0));
                }
            }
        }

        // Active indicator (when menu is open)
        if self.is_menu_open {
            let indicator = Rect::new(
                scaled_rect.x(),
                scaled_rect.max_y() - 3.0,
                scaled_rect.width(),
                3.0,
            );
            painter.fill_rect(indicator, self.icon_color);
        }
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(MouseEvent { kind, position, button, .. }) => {
                let in_bounds = self.base.bounds.contains(*position);
                let is_left_button = *button == Some(MouseButton::Left);

                match kind {
                    MouseEventKind::Move => {
                        if in_bounds && self.state != StartButtonState::Active {
                            self.state = StartButtonState::Hovered;
                            return EventResult::Handled;
                        } else if !in_bounds && self.state == StartButtonState::Hovered {
                            self.state = StartButtonState::Normal;
                            return EventResult::Handled;
                        }
                    }
                    MouseEventKind::Down if in_bounds && is_left_button => {
                        self.state = StartButtonState::Pressed;
                        return EventResult::Handled;
                    }
                    MouseEventKind::Up if is_left_button => {
                        if in_bounds && self.state == StartButtonState::Pressed {
                            self.toggle_menu();
                            if let Some(ref cb) = self.on_click {
                                cb();
                            }
                            return EventResult::Handled;
                        }
                        if !self.is_menu_open {
                            self.state = if in_bounds {
                                StartButtonState::Hovered
                            } else {
                                StartButtonState::Normal
                            };
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
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
