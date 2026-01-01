//! Calendar popup widget
//!
//! A calendar widget for date selection and display.

use super::{EventContext, LayoutContext, PaintContext, Widget, WidgetBase, WidgetId};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;
use std::collections::HashMap;

/// Calendar event
#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub start_time: Option<(u8, u8)>,
    pub end_time: Option<(u8, u8)>,
    pub color: Color,
    pub all_day: bool,
}

impl CalendarEvent {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            start_time: None,
            end_time: None,
            color: Color::rgb(0.0, 0.47, 0.84),
            all_day: false,
        }
    }

    pub fn time(mut self, start: (u8, u8), end: (u8, u8)) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn all_day(mut self) -> Self {
        self.all_day = true;
        self.start_time = None;
        self.end_time = None;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Calendar popup widget
pub struct Calendar {
    base: WidgetBase,
    year: i32,
    month: u8,
    selected_day: Option<u8>,
    today: (i32, u8, u8),
    events: HashMap<u8, Vec<CalendarEvent>>,
    is_open: bool,
    width: f32,
    show_week_numbers: bool,
    first_day_of_week: u8,
    #[allow(clippy::type_complexity)]
    on_date_select: Option<Box<dyn Fn(i32, u8, u8) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_month_change: Option<Box<dyn Fn(i32, u8) + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_event_click: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl Calendar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("calendar"),
            year: 2025,
            month: 1,
            selected_day: None,
            today: (2025, 1, 1),
            events: HashMap::new(),
            is_open: false,
            width: 320.0,
            show_week_numbers: false,
            first_day_of_week: 0,
            on_date_select: None,
            on_month_change: None,
            on_event_click: None,
        }
    }

    pub fn date(mut self, year: i32, month: u8, day: u8) -> Self {
        self.year = year;
        self.month = month.clamp(1, 12);
        self.selected_day = Some(day.clamp(1, 31));
        self
    }

    pub fn today(mut self, year: i32, month: u8, day: u8) -> Self {
        self.today = (year, month, day);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn show_week_numbers(mut self, show: bool) -> Self {
        self.show_week_numbers = show;
        self
    }

    pub fn first_day_of_week(mut self, day: u8) -> Self {
        self.first_day_of_week = day.min(6);
        self
    }

    pub fn event(mut self, day: u8, event: CalendarEvent) -> Self {
        self.events.entry(day).or_default().push(event);
        self
    }

    pub fn on_date_select<F>(mut self, f: F) -> Self
    where
        F: Fn(i32, u8, u8) + Send + Sync + 'static,
    {
        self.on_date_select = Some(Box::new(f));
        self
    }

    pub fn on_month_change<F>(mut self, f: F) -> Self
    where
        F: Fn(i32, u8) + Send + Sync + 'static,
    {
        self.on_month_change = Some(Box::new(f));
        self
    }

    pub fn on_event_click<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_event_click = Some(Box::new(f));
        self
    }

    pub fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
        if let Some(ref cb) = self.on_month_change {
            cb(self.year, self.month);
        }
    }

    pub fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
        if let Some(ref cb) = self.on_month_change {
            cb(self.year, self.month);
        }
    }

    pub fn go_to_today(&mut self) {
        self.year = self.today.0;
        self.month = self.today.1;
        self.selected_day = Some(self.today.2);
    }

    pub fn select_day(&mut self, day: u8) {
        self.selected_day = Some(day);
        if let Some(ref cb) = self.on_date_select {
            cb(self.year, self.month, day);
        }
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

    pub fn days_in_month(&self) -> u8 {
        match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if self.is_leap_year() { 29 } else { 28 },
            _ => 30,
        }
    }

    pub fn is_leap_year(&self) -> bool {
        (self.year % 4 == 0 && self.year % 100 != 0) || (self.year % 400 == 0)
    }

    pub fn month_name(&self) -> &'static str {
        match self.month {
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "Unknown",
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

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Calendar {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "calendar"
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
        let cell_size = self.width / 7.0;
        let height = 48.0 + 32.0 + (6.0 * cell_size);
        Size::new(self.width, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, _ctx: &PaintContext) {
        let cell_size = rect.width() / 7.0;

        // Background
        painter.fill_rounded_rect(rect, Color::rgba(0.12, 0.12, 0.12, 0.98), BorderRadius::all(8.0));

        // Header
        let header_rect = Rect::new(rect.x(), rect.y(), rect.width(), 48.0);
        painter.fill_rect(header_rect, Color::rgba(1.0, 1.0, 1.0, 0.05));

        // Month/year title
        let title = format!("{} {}", self.month_name(), self.year);
        painter.draw_text(&title, Point::new(rect.x() + 16.0, rect.y() + 30.0), Color::WHITE, 16.0);

        // Navigation arrows
        painter.draw_text("◀", Point::new(rect.max_x() - 64.0, rect.y() + 30.0), Color::WHITE, 16.0);
        painter.draw_text("▶", Point::new(rect.max_x() - 32.0, rect.y() + 30.0), Color::WHITE, 16.0);

        // Weekday headers
        let weekdays = ["S", "M", "T", "W", "T", "F", "S"];
        let weekday_y = rect.y() + 48.0;

        for (i, day) in weekdays.iter().enumerate() {
            let cell_x = rect.x() + (i as f32 * cell_size);
            painter.draw_text(
                day,
                Point::new(cell_x + cell_size / 2.0 - 4.0, weekday_y + 20.0),
                Color::rgba(1.0, 1.0, 1.0, 0.6),
                12.0,
            );
        }

        // Day cells
        let grid_y = weekday_y + 32.0;
        let days = self.days_in_month();

        for day in 1..=days {
            let row = ((day - 1) / 7) as f32;
            let col = ((day - 1) % 7) as f32;

            let cell_rect = Rect::new(
                rect.x() + (col * cell_size),
                grid_y + (row * cell_size),
                cell_size,
                cell_size,
            );

            let is_today = self.year == self.today.0 && self.month == self.today.1 && day == self.today.2;
            let is_selected = self.selected_day == Some(day);

            if is_selected {
                painter.fill_rounded_rect(cell_rect, Color::rgb(0.0, 0.47, 0.84), BorderRadius::all(cell_size / 2.0));
            } else if is_today {
                painter.stroke_rect(cell_rect, Color::rgb(0.0, 0.47, 0.84), 2.0);
            }

            // Day number
            let text_color = Color::WHITE;
            painter.draw_text(
                &day.to_string(),
                Point::new(cell_rect.x() + cell_size / 2.0 - 6.0, cell_rect.y() + cell_size / 2.0 + 5.0),
                text_color,
                14.0,
            );

            // Event indicator
            if let Some(events) = self.events.get(&day) {
                if !events.is_empty() {
                    let dot_rect = Rect::new(
                        cell_rect.x() + cell_size / 2.0 - 2.0,
                        cell_rect.max_y() - 8.0,
                        4.0,
                        4.0,
                    );
                    painter.fill_rounded_rect(dot_rect, events[0].color, BorderRadius::all(2.0));
                }
            }
        }
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
