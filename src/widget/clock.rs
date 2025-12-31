//! Clock widget for displaying time and date.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Clock format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockFormat {
    /// 12-hour format (e.g., "3:45 PM")
    TwelveHour,
    /// 24-hour format (e.g., "15:45") - default
    #[default]
    TwentyFourHour,
}

/// Date format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateFormat {
    /// Short date (e.g., "Jan 15")
    Short,
    /// Long date (e.g., "January 15, 2024") - default
    #[default]
    Long,
    /// Day of week (e.g., "Monday")
    DayOfWeek,
    /// Full (e.g., "Monday, January 15, 2024")
    Full,
}

/// A clock widget displaying time and optionally date.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Simple time display
/// let clock = Clock::new()
///     .format(ClockFormat::TwelveHour)
///     .show_seconds(true);
///
/// // With date
/// let clock_with_date = Clock::new()
///     .show_date(true)
///     .date_format(DateFormat::Short);
/// ```
pub struct Clock {
    base: WidgetBase,
    format: ClockFormat,
    show_seconds: bool,
    show_date: bool,
    date_format: DateFormat,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    // Cached time string (would be updated by timer in real implementation)
    cached_time: String,
    cached_date: String,
}

impl Clock {
    /// Create a new clock.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("clock"),
            format: ClockFormat::default(),
            show_seconds: false,
            show_date: false,
            date_format: DateFormat::default(),
            on_click: None,
            cached_time: "12:00".to_string(),
            cached_date: "January 1".to_string(),
        }
    }

    /// Set the time format.
    pub fn format(mut self, format: ClockFormat) -> Self {
        self.format = format;
        self
    }

    /// Set whether to show seconds.
    pub fn show_seconds(mut self, show: bool) -> Self {
        self.show_seconds = show;
        self
    }

    /// Set whether to show the date.
    pub fn show_date(mut self, show: bool) -> Self {
        self.show_date = show;
        self
    }

    /// Set the date format.
    pub fn date_format(mut self, format: DateFormat) -> Self {
        self.date_format = format;
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

    /// Update the displayed time (call periodically).
    pub fn update(&mut self) {
        // In a real implementation, this would get the current system time
        // For now, we'll use placeholder values

        // Get current time from system
        let now = std::time::SystemTime::now();
        if let Ok(duration) = now.duration_since(std::time::UNIX_EPOCH) {
            let secs = duration.as_secs();
            let hours = ((secs / 3600) % 24) as u32;
            let minutes = ((secs / 60) % 60) as u32;
            let seconds = (secs % 60) as u32;

            self.cached_time = match self.format {
                ClockFormat::TwelveHour => {
                    let (h, ampm) = if hours == 0 {
                        (12, "AM")
                    } else if hours < 12 {
                        (hours, "AM")
                    } else if hours == 12 {
                        (12, "PM")
                    } else {
                        (hours - 12, "PM")
                    };

                    if self.show_seconds {
                        format!("{}:{:02}:{:02} {}", h, minutes, seconds, ampm)
                    } else {
                        format!("{}:{:02} {}", h, minutes, ampm)
                    }
                }
                ClockFormat::TwentyFourHour => {
                    if self.show_seconds {
                        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                    } else {
                        format!("{:02}:{:02}", hours, minutes)
                    }
                }
            };

            // Simplified date - in real implementation would use proper date formatting
            let days = secs / 86400;
            let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                             "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            let day_names = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
            let day_of_week = day_names[(days % 7) as usize];
            let month = month_names[((days / 30) % 12) as usize];
            let day_of_month = (days % 30) + 1;

            self.cached_date = match self.date_format {
                DateFormat::Short => format!("{} {}", month, day_of_month),
                DateFormat::Long => format!("{} {}, 2024", month, day_of_month),
                DateFormat::DayOfWeek => day_of_week.to_string(),
                DateFormat::Full => format!("{}, {} {}, 2024", day_of_week, month, day_of_month),
            };
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Clock {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "clock"
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
        let time_width = self.cached_time.len() as f32 * 8.0;
        let date_width = if self.show_date { self.cached_date.len() as f32 * 7.0 } else { 0.0 };
        let width = time_width.max(date_width) + 16.0;
        let height = if self.show_date { 40.0 } else { 24.0 };
        Size::new(width, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        self.update(); // Update time on layout
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Hover background
        if self.base.state.hovered && self.on_click.is_some() {
            painter.fill_rect(rect, theme.colors.accent.with_alpha(0.1));
        }

        let center_x = rect.x() + rect.width() / 2.0;

        if self.show_date {
            // Time
            let time_width = self.cached_time.len() as f32 * 8.0;
            painter.draw_text(
                &self.cached_time,
                Point::new(center_x - time_width / 2.0, rect.y() + 18.0),
                theme.colors.foreground,
                14.0,
            );

            // Date
            let date_width = self.cached_date.len() as f32 * 6.0;
            painter.draw_text(
                &self.cached_date,
                Point::new(center_x - date_width / 2.0, rect.y() + 34.0),
                theme.colors.muted_foreground,
                11.0,
            );
        } else {
            // Time only, centered
            let time_width = self.cached_time.len() as f32 * 8.0;
            painter.draw_text(
                &self.cached_time,
                Point::new(center_x - time_width / 2.0, rect.y() + rect.height() * 0.7),
                theme.colors.foreground,
                14.0,
            );
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            let in_bounds = self.bounds().contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Move | MouseEventKind::Enter => {
                    if in_bounds && !self.base.state.hovered && self.on_click.is_some() {
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
