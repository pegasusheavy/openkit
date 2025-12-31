//! Desktop widget with wallpaper and icon grid.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;
use std::path::PathBuf;

/// How to scale/position the wallpaper image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WallpaperMode {
    /// Scale to fill the entire screen, may crop (default)
    #[default]
    Fill,
    /// Scale to fit within screen, may show background color
    Fit,
    /// Stretch to exact screen size, may distort
    Stretch,
    /// Tile the image
    Tile,
    /// Center without scaling
    Center,
    /// Span across multiple monitors (for multi-monitor setups)
    Span,
}

/// Gradient direction for gradient backgrounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientDirection {
    /// Top to bottom (default)
    #[default]
    ToBottom,
    /// Bottom to top
    ToTop,
    /// Left to right
    ToRight,
    /// Right to left
    ToLeft,
    /// Top-left to bottom-right
    ToBottomRight,
    /// Top-right to bottom-left
    ToBottomLeft,
    /// Bottom-left to top-right
    ToTopRight,
    /// Bottom-right to top-left
    ToTopLeft,
    /// Radial gradient from center
    Radial,
}

/// A wallpaper/background configuration.
#[derive(Debug, Clone)]
pub enum Wallpaper {
    /// Solid color background
    Color(Color),
    /// Image background with scaling mode
    Image {
        path: PathBuf,
        mode: WallpaperMode,
        /// Fallback color if image fails to load
        fallback: Color,
    },
    /// Linear or radial gradient
    Gradient {
        start: Color,
        end: Color,
        direction: GradientDirection,
    },
    /// Image with color overlay/tint
    ImageWithOverlay {
        path: PathBuf,
        mode: WallpaperMode,
        overlay: Color,
        fallback: Color,
    },
}

impl Default for Wallpaper {
    fn default() -> Self {
        Wallpaper::Color(Color::rgb(0.1, 0.1, 0.18))
    }
}

impl Wallpaper {
    /// Create a solid color wallpaper.
    pub fn color(color: Color) -> Self {
        Wallpaper::Color(color)
    }

    /// Create an image wallpaper.
    pub fn image(path: impl Into<PathBuf>) -> Self {
        Wallpaper::Image {
            path: path.into(),
            mode: WallpaperMode::default(),
            fallback: Color::rgb(0.1, 0.1, 0.18),
        }
    }

    /// Create an image wallpaper with a specific mode.
    pub fn image_with_mode(path: impl Into<PathBuf>, mode: WallpaperMode) -> Self {
        Wallpaper::Image {
            path: path.into(),
            mode,
            fallback: Color::rgb(0.1, 0.1, 0.18),
        }
    }

    /// Create a gradient wallpaper.
    pub fn gradient(start: Color, end: Color) -> Self {
        Wallpaper::Gradient {
            start,
            end,
            direction: GradientDirection::default(),
        }
    }

    /// Create a gradient with direction.
    pub fn gradient_with_direction(start: Color, end: Color, direction: GradientDirection) -> Self {
        Wallpaper::Gradient {
            start,
            end,
            direction,
        }
    }

    /// Create an image with color overlay.
    pub fn image_with_overlay(path: impl Into<PathBuf>, overlay: Color) -> Self {
        Wallpaper::ImageWithOverlay {
            path: path.into(),
            mode: WallpaperMode::Fill,
            overlay,
            fallback: Color::rgb(0.1, 0.1, 0.18),
        }
    }

    /// Set the scaling mode (for image wallpapers).
    pub fn with_mode(self, mode: WallpaperMode) -> Self {
        match self {
            Wallpaper::Image { path, fallback, .. } => Wallpaper::Image { path, mode, fallback },
            Wallpaper::ImageWithOverlay { path, overlay, fallback, .. } => {
                Wallpaper::ImageWithOverlay { path, mode, overlay, fallback }
            }
            other => other,
        }
    }

    /// Set the fallback color (for image wallpapers).
    pub fn with_fallback(self, fallback: Color) -> Self {
        match self {
            Wallpaper::Image { path, mode, .. } => Wallpaper::Image { path, mode, fallback },
            Wallpaper::ImageWithOverlay { path, mode, overlay, .. } => {
                Wallpaper::ImageWithOverlay { path, mode, overlay, fallback }
            }
            other => other,
        }
    }

    /// Set the gradient direction (for gradient wallpapers).
    pub fn with_direction(self, direction: GradientDirection) -> Self {
        match self {
            Wallpaper::Gradient { start, end, .. } => Wallpaper::Gradient { start, end, direction },
            other => other,
        }
    }
}

/// Desktop icon item.
#[derive(Debug, Clone)]
pub struct DesktopIcon {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Icon (emoji or icon name)
    pub icon: String,
    /// Grid position (column, row)
    pub position: (usize, usize),
    /// Whether this icon is selected
    pub selected: bool,
}

impl DesktopIcon {
    /// Create a new desktop icon.
    pub fn new(id: impl Into<String>, name: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            position: (0, 0),
            selected: false,
        }
    }

    /// Set the grid position.
    pub fn at(mut self, col: usize, row: usize) -> Self {
        self.position = (col, row);
        self
    }
}

/// A desktop widget with wallpaper background and icon grid.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Solid color background
/// let desktop = Desktop::new()
///     .background(Wallpaper::color(Color::from_hex("#1a1a2e").unwrap()));
///
/// // Image wallpaper with fill mode
/// let desktop = Desktop::new()
///     .background(Wallpaper::image("/usr/share/backgrounds/nature.jpg")
///         .with_mode(WallpaperMode::Fill));
///
/// // Gradient background
/// let desktop = Desktop::new()
///     .background(Wallpaper::gradient(
///         Color::from_hex("#1a1a2e").unwrap(),
///         Color::from_hex("#16213e").unwrap(),
///     ).with_direction(GradientDirection::ToBottomRight));
///
/// // Image with dark overlay
/// let desktop = Desktop::new()
///     .background(Wallpaper::image_with_overlay(
///         "/path/to/image.jpg",
///         Color::BLACK.with_alpha(0.3),
///     ));
/// ```
#[allow(clippy::type_complexity)]
pub struct Desktop {
    base: WidgetBase,
    /// Background/wallpaper configuration
    wallpaper: Wallpaper,
    /// Desktop icons
    icons: Vec<DesktopIcon>,
    /// Icon size
    icon_size: f32,
    /// Grid cell size
    cell_size: f32,
    /// Grid padding from edges
    grid_padding: f32,
    /// Currently hovered icon
    hovered_icon: Option<String>,
    /// Last click time for double-click detection
    last_click_time: Option<std::time::Instant>,
    last_click_id: Option<String>,
    /// Callbacks
    on_icon_click: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_icon_double_click: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_right_click: Option<Box<dyn Fn(Point) + Send + Sync>>,
    on_wallpaper_change: Option<Box<dyn Fn(&Wallpaper) + Send + Sync>>,
}

impl Desktop {
    /// Create a new desktop.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("desktop"),
            wallpaper: Wallpaper::default(),
            icons: Vec::new(),
            icon_size: 48.0,
            cell_size: 80.0,
            grid_padding: 16.0,
            hovered_icon: None,
            last_click_time: None,
            last_click_id: None,
            on_icon_click: None,
            on_icon_double_click: None,
            on_right_click: None,
            on_wallpaper_change: None,
        }
    }

    /// Set the background/wallpaper.
    pub fn background(mut self, wallpaper: Wallpaper) -> Self {
        self.wallpaper = wallpaper;
        self
    }

    /// Set a solid color background (convenience method).
    pub fn wallpaper_color(mut self, color: Color) -> Self {
        self.wallpaper = Wallpaper::Color(color);
        self
    }

    /// Set an image wallpaper (convenience method).
    pub fn wallpaper_image(mut self, path: impl Into<PathBuf>) -> Self {
        self.wallpaper = Wallpaper::image(path);
        self
    }

    /// Set an image wallpaper with mode (convenience method).
    pub fn wallpaper_image_with_mode(mut self, path: impl Into<PathBuf>, mode: WallpaperMode) -> Self {
        self.wallpaper = Wallpaper::image_with_mode(path, mode);
        self
    }

    /// Set a gradient background (convenience method).
    pub fn wallpaper_gradient(mut self, start: Color, end: Color, direction: GradientDirection) -> Self {
        self.wallpaper = Wallpaper::gradient_with_direction(start, end, direction);
        self
    }

    /// Add a desktop icon.
    pub fn icon(mut self, icon: DesktopIcon) -> Self {
        self.icons.push(icon);
        self
    }

    /// Set multiple icons.
    pub fn icons(mut self, icons: Vec<DesktopIcon>) -> Self {
        self.icons = icons;
        self
    }

    /// Set the icon size.
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    /// Set the grid cell size.
    pub fn cell_size(mut self, size: f32) -> Self {
        self.cell_size = size;
        self
    }

    /// Set the icon click handler.
    pub fn on_icon_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_icon_click = Some(Box::new(handler));
        self
    }

    /// Set the icon double-click handler.
    pub fn on_icon_double_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_icon_double_click = Some(Box::new(handler));
        self
    }

    /// Set the right-click handler (for context menu).
    pub fn on_right_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(Point) + Send + Sync + 'static,
    {
        self.on_right_click = Some(Box::new(handler));
        self
    }

    /// Set the wallpaper change handler.
    pub fn on_wallpaper_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Wallpaper) + Send + Sync + 'static,
    {
        self.on_wallpaper_change = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the current wallpaper.
    pub fn get_wallpaper(&self) -> &Wallpaper {
        &self.wallpaper
    }

    /// Set the wallpaper at runtime.
    pub fn set_wallpaper(&mut self, wallpaper: Wallpaper) {
        self.wallpaper = wallpaper;
        if let Some(handler) = &self.on_wallpaper_change {
            handler(&self.wallpaper);
        }
    }

    /// Get the rect for an icon at a grid position.
    fn get_icon_rect(&self, col: usize, row: usize) -> Rect {
        let x = self.base.bounds.x() + self.grid_padding + (col as f32) * self.cell_size;
        let y = self.base.bounds.y() + self.grid_padding + (row as f32) * self.cell_size;
        Rect::new(x, y, self.cell_size, self.cell_size)
    }

    /// Find which icon is at a point.
    fn icon_at_point(&self, point: Point) -> Option<&DesktopIcon> {
        for icon in &self.icons {
            let rect = self.get_icon_rect(icon.position.0, icon.position.1);
            if rect.contains(point) {
                return Some(icon);
            }
        }
        None
    }

    /// Select an icon by ID.
    pub fn select_icon(&mut self, id: &str) {
        for icon in &mut self.icons {
            icon.selected = icon.id == id;
        }
    }

    /// Clear selection.
    pub fn clear_selection(&mut self) {
        for icon in &mut self.icons {
            icon.selected = false;
        }
    }

    /// Paint the wallpaper background.
    fn paint_wallpaper(&self, painter: &mut Painter, rect: Rect) {
        match &self.wallpaper {
            Wallpaper::Color(color) => {
                painter.fill_rect(rect, *color);
            }
            Wallpaper::Image { path: _, mode: _, fallback } => {
                // TODO: Implement actual image loading and rendering
                // For now, draw the fallback color
                painter.fill_rect(rect, *fallback);

                // Placeholder: draw a pattern to indicate image area
                self.draw_image_placeholder(painter, rect);
            }
            Wallpaper::Gradient { start, end, direction } => {
                self.paint_gradient(painter, rect, *start, *end, *direction);
            }
            Wallpaper::ImageWithOverlay { path: _, mode: _, overlay, fallback } => {
                // TODO: Implement actual image loading
                painter.fill_rect(rect, *fallback);
                self.draw_image_placeholder(painter, rect);

                // Draw overlay
                painter.fill_rect(rect, *overlay);
            }
        }
    }

    /// Draw a placeholder pattern for images (until image loading is implemented).
    fn draw_image_placeholder(&self, painter: &mut Painter, rect: Rect) {
        // Draw a subtle grid pattern to indicate where image would be
        let grid_size: f32 = 40.0;
        let line_color = Color::WHITE.with_alpha(0.03);

        let mut x = rect.x();
        while x < rect.x() + rect.width() {
            painter.fill_rect(Rect::new(x, rect.y(), 1.0, rect.height()), line_color);
            x += grid_size;
        }

        let mut y = rect.y();
        while y < rect.y() + rect.height() {
            painter.fill_rect(Rect::new(rect.x(), y, rect.width(), 1.0), line_color);
            y += grid_size;
        }
    }

    /// Paint a gradient background.
    fn paint_gradient(&self, painter: &mut Painter, rect: Rect, start: Color, end: Color, direction: GradientDirection) {
        // Simulate gradient with multiple color bands
        let steps = 64;

        match direction {
            GradientDirection::ToBottom | GradientDirection::ToTop => {
                let (from, to) = if direction == GradientDirection::ToBottom {
                    (start, end)
                } else {
                    (end, start)
                };
                let step_height = rect.height() / steps as f32;

                for i in 0..steps {
                    let t = i as f32 / (steps - 1) as f32;
                    let color = Self::lerp_color(from, to, t);
                    let y = rect.y() + (i as f32) * step_height;
                    painter.fill_rect(Rect::new(rect.x(), y, rect.width(), step_height + 1.0), color);
                }
            }
            GradientDirection::ToRight | GradientDirection::ToLeft => {
                let (from, to) = if direction == GradientDirection::ToRight {
                    (start, end)
                } else {
                    (end, start)
                };
                let step_width = rect.width() / steps as f32;

                for i in 0..steps {
                    let t = i as f32 / (steps - 1) as f32;
                    let color = Self::lerp_color(from, to, t);
                    let x = rect.x() + (i as f32) * step_width;
                    painter.fill_rect(Rect::new(x, rect.y(), step_width + 1.0, rect.height()), color);
                }
            }
            GradientDirection::ToBottomRight | GradientDirection::ToTopLeft => {
                let (from, to) = if direction == GradientDirection::ToBottomRight {
                    (start, end)
                } else {
                    (end, start)
                };
                // Diagonal gradient approximation
                for i in 0..steps {
                    let t = i as f32 / (steps - 1) as f32;
                    let color = Self::lerp_color(from, to, t);
                    let offset = t * (rect.width() + rect.height());
                    // Draw diagonal band
                    for j in 0..10 {
                        let y = rect.y() + offset - rect.width() + (j as f32);
                        if y >= rect.y() && y < rect.y() + rect.height() {
                            painter.fill_rect(Rect::new(rect.x(), y, rect.width(), 1.0), color);
                        }
                    }
                }
            }
            GradientDirection::ToBottomLeft | GradientDirection::ToTopRight => {
                let (from, to) = if direction == GradientDirection::ToBottomLeft {
                    (start, end)
                } else {
                    (end, start)
                };
                // Diagonal gradient approximation (other direction)
                for i in 0..steps {
                    let t = i as f32 / (steps - 1) as f32;
                    let color = Self::lerp_color(from, to, t);
                    let offset = t * (rect.width() + rect.height());
                    for j in 0..10 {
                        let y = rect.y() + offset - rect.width() + (j as f32);
                        if y >= rect.y() && y < rect.y() + rect.height() {
                            painter.fill_rect(Rect::new(rect.x(), y, rect.width(), 1.0), color);
                        }
                    }
                }
            }
            GradientDirection::Radial => {
                // Radial gradient approximation with concentric circles
                let center_x = rect.x() + rect.width() / 2.0;
                let center_y = rect.y() + rect.height() / 2.0;
                let max_radius = (rect.width().powi(2) + rect.height().powi(2)).sqrt() / 2.0;

                // Draw from outside in so inner colors overwrite outer
                for i in (0..steps).rev() {
                    let t = i as f32 / (steps - 1) as f32;
                    let color = Self::lerp_color(start, end, t);
                    let radius = max_radius * t;

                    // Approximate circle with filled rect (simplified)
                    let size = radius * 2.0;
                    painter.fill_rect(
                        Rect::new(center_x - radius, center_y - radius, size, size),
                        color,
                    );
                }
            }
        }
    }

    /// Linearly interpolate between two colors.
    fn lerp_color(from: Color, to: Color, t: f32) -> Color {
        Color::rgba(
            from.r + (to.r - from.r) * t,
            from.g + (to.g - from.g) * t,
            from.b + (to.b - from.b) * t,
            from.a + (to.a - from.a) * t,
        )
    }
}

impl Default for Desktop {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Desktop {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "desktop"
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
        // Desktop fills available space
        Size::new(f32::MAX, f32::MAX)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        let size = Size::new(constraints.max_width, constraints.max_height);
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Draw wallpaper background
        self.paint_wallpaper(painter, rect);

        // Draw icons
        for icon in &self.icons {
            let cell_rect = self.get_icon_rect(icon.position.0, icon.position.1);

            // Selection/hover background
            if icon.selected {
                painter.fill_rect(
                    cell_rect,
                    theme.colors.accent.with_alpha(0.3),
                );
            } else if self.hovered_icon.as_ref() == Some(&icon.id) {
                painter.fill_rect(
                    cell_rect,
                    theme.colors.accent.with_alpha(0.15),
                );
            }

            // Icon
            let icon_x = cell_rect.x() + (cell_rect.width() - self.icon_size) / 2.0;
            let icon_y = cell_rect.y() + 8.0;
            painter.draw_text(
                &icon.icon,
                Point::new(icon_x, icon_y + self.icon_size * 0.8),
                Color::WHITE,
                self.icon_size,
            );

            // Label
            let label_y = icon_y + self.icon_size + 8.0;
            let font_size = 11.0;
            let label_x = cell_rect.x() + (cell_rect.width() - icon.name.len() as f32 * font_size * 0.5) / 2.0;
            painter.draw_text(
                &icon.name,
                Point::new(label_x, label_y + font_size),
                Color::WHITE,
                font_size,
            );
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Move => {
                    let icon = self.icon_at_point(mouse.position);
                    let new_hovered = icon.map(|i| i.id.clone());
                    if new_hovered != self.hovered_icon {
                        self.hovered_icon = new_hovered;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                    if let Some(icon) = self.icon_at_point(mouse.position) {
                        let icon_id = icon.id.clone();

                        // Check for double-click
                        let now = std::time::Instant::now();
                        let is_double_click = if let (Some(last_time), Some(last_id)) =
                            (&self.last_click_time, &self.last_click_id)
                        {
                            now.duration_since(*last_time).as_millis() < 500 && last_id == &icon_id
                        } else {
                            false
                        };

                        if is_double_click {
                            if let Some(handler) = &self.on_icon_double_click {
                                handler(&icon_id);
                            }
                            self.last_click_time = None;
                            self.last_click_id = None;
                        } else {
                            self.select_icon(&icon_id);
                            if let Some(handler) = &self.on_icon_click {
                                handler(&icon_id);
                            }
                            self.last_click_time = Some(now);
                            self.last_click_id = Some(icon_id);
                        }
                        ctx.request_redraw();
                        return EventResult::Handled;
                    } else {
                        // Clicked on empty space - clear selection
                        self.clear_selection();
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down if mouse.button == Some(MouseButton::Right) => {
                    if let Some(handler) = &self.on_right_click {
                        handler(mouse.position);
                    }
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
