//! Speed graph widget for real-time bandwidth visualization.
//!
//! A line chart widget ideal for showing download/upload speeds over time.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind};
use crate::geometry::{Color, Point, Rect, Size, BorderRadius};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A data series for the graph.
#[derive(Debug, Clone)]
pub struct GraphSeries {
    /// Series identifier
    pub id: String,
    /// Series label
    pub label: String,
    /// Data points (values over time)
    pub data: Vec<f64>,
    /// Line color
    pub color: Color,
    /// Fill area under the line
    pub fill: bool,
    /// Line width
    pub line_width: f32,
}

impl GraphSeries {
    /// Create a new series.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            data: Vec::new(),
            color: Color::from_rgb8(59, 130, 246), // Blue
            fill: true,
            line_width: 2.0,
        }
    }

    /// Set the color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set initial data.
    pub fn data(mut self, data: Vec<f64>) -> Self {
        self.data = data;
        self
    }

    /// Enable/disable fill.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Set line width.
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Push a new data point.
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
    }

    /// Push a new data point, maintaining max capacity.
    pub fn push_bounded(&mut self, value: f64, max_points: usize) {
        self.data.push(value);
        while self.data.len() > max_points {
            self.data.remove(0);
        }
    }
}

/// Graph time scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeScale {
    /// Last 30 seconds
    Seconds30,
    /// Last minute
    #[default]
    Minute1,
    /// Last 5 minutes
    Minutes5,
    /// Last 15 minutes
    Minutes15,
    /// Last hour
    Hour1,
}

impl TimeScale {
    /// Get the number of data points for this scale.
    pub fn points(&self) -> usize {
        match self {
            TimeScale::Seconds30 => 30,
            TimeScale::Minute1 => 60,
            TimeScale::Minutes5 => 300,
            TimeScale::Minutes15 => 900,
            TimeScale::Hour1 => 3600,
        }
    }

    /// Get the label for this scale.
    pub fn label(&self) -> &'static str {
        match self {
            TimeScale::Seconds30 => "30s",
            TimeScale::Minute1 => "1m",
            TimeScale::Minutes5 => "5m",
            TimeScale::Minutes15 => "15m",
            TimeScale::Hour1 => "1h",
        }
    }
}

/// Speed graph widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::widget::speed_graph::*;
/// use openkit::geometry::Color;
///
/// let graph = SpeedGraph::new()
///     .series(
///         GraphSeries::new("download", "Download")
///             .color(Color::from_rgb(34, 197, 94))
///             .fill(true)
///     )
///     .series(
///         GraphSeries::new("upload", "Upload")
///             .color(Color::from_rgb(168, 85, 247))
///             .fill(true)
///     )
///     .time_scale(TimeScale::Minute1)
///     .show_legend(true)
///     .show_grid(true);
///
/// // Update with new data points
/// graph.push("download", 1024.0 * 500.0); // 500 KB/s
/// graph.push("upload", 1024.0 * 100.0);   // 100 KB/s
/// ```
pub struct SpeedGraph {
    base: WidgetBase,
    series: Vec<GraphSeries>,
    time_scale: TimeScale,
    max_value: Option<f64>,
    auto_scale: bool,
    show_legend: bool,
    show_grid: bool,
    show_labels: bool,
    show_current_value: bool,
    grid_lines: usize,
    padding: f32,
    hovered_point: Option<(usize, usize)>, // (series_idx, point_idx)
}

impl SpeedGraph {
    /// Create a new speed graph.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("speed-graph"),
            series: Vec::new(),
            time_scale: TimeScale::default(),
            max_value: None,
            auto_scale: true,
            show_legend: true,
            show_grid: true,
            show_labels: true,
            show_current_value: true,
            grid_lines: 4,
            padding: 8.0,
            hovered_point: None,
        }
    }

    /// Add a series.
    pub fn series(mut self, series: GraphSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Set the time scale.
    pub fn time_scale(mut self, scale: TimeScale) -> Self {
        self.time_scale = scale;
        self
    }

    /// Set a fixed maximum value.
    pub fn max_value(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self.auto_scale = false;
        self
    }

    /// Enable auto-scaling.
    pub fn auto_scale(mut self, auto: bool) -> Self {
        self.auto_scale = auto;
        self
    }

    /// Show/hide legend.
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Show/hide grid.
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Show/hide axis labels.
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Show/hide current value display.
    pub fn show_current_value(mut self, show: bool) -> Self {
        self.show_current_value = show;
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Push a value to a series.
    pub fn push(&mut self, series_id: &str, value: f64) {
        let max_points = self.time_scale.points();
        if let Some(series) = self.series.iter_mut().find(|s| s.id == series_id) {
            series.push_bounded(value, max_points);
        }
    }

    /// Get the current value for a series.
    pub fn current(&self, series_id: &str) -> Option<f64> {
        self.series
            .iter()
            .find(|s| s.id == series_id)
            .and_then(|s| s.data.last().copied())
    }

    /// Calculate the maximum value across all series.
    fn calculate_max(&self) -> f64 {
        if let Some(max) = self.max_value {
            return max;
        }

        let max = self.series
            .iter()
            .flat_map(|s| s.data.iter())
            .fold(0.0f64, |acc, &v| acc.max(v));

        // Round up to a nice number
        if max <= 0.0 {
            1024.0 // 1 KB/s minimum
        } else {
            let magnitude = 10.0f64.powf(max.log10().floor());
            (max / magnitude).ceil() * magnitude
        }
    }

    fn graph_rect(&self) -> Rect {
        let bounds = self.base.bounds;
        let left_margin = if self.show_labels { 60.0 } else { self.padding };
        let right_margin = self.padding;
        let top_margin = if self.show_legend { 30.0 } else { self.padding };
        let bottom_margin = if self.show_labels { 24.0 } else { self.padding };

        Rect::new(
            bounds.x() + left_margin,
            bounds.y() + top_margin,
            bounds.width() - left_margin - right_margin,
            bounds.height() - top_margin - bottom_margin,
        )
    }
}

impl Default for SpeedGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SpeedGraph {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "speed-graph"
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
        Size::new(400.0, 200.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        painter.fill_rounded_rect(rect, theme.colors.card, BorderRadius::all(8.0));

        let graph = self.graph_rect();
        let max_value = self.calculate_max();
        let num_points = self.time_scale.points();

        // Draw grid
        if self.show_grid {
            let grid_color = theme.colors.border.with_alpha(0.3);

            // Horizontal grid lines
            for i in 0..=self.grid_lines {
                let y = graph.y() + graph.height() * (i as f32 / self.grid_lines as f32);
                painter.fill_rect(
                    Rect::new(graph.x(), y, graph.width(), 1.0),
                    grid_color,
                );
            }

            // Vertical grid lines
            let v_lines = 6;
            for i in 0..=v_lines {
                let x = graph.x() + graph.width() * (i as f32 / v_lines as f32);
                painter.fill_rect(
                    Rect::new(x, graph.y(), 1.0, graph.height()),
                    grid_color,
                );
            }
        }

        // Draw Y-axis labels
        if self.show_labels {
            for i in 0..=self.grid_lines {
                let value = max_value * ((self.grid_lines - i) as f64 / self.grid_lines as f64);
                let label = format_speed(value);
                let y = graph.y() + graph.height() * (i as f32 / self.grid_lines as f32);
                painter.draw_text(
                    &label,
                    Point::new(rect.x() + 4.0, y + 4.0),
                    theme.colors.muted_foreground,
                    10.0,
                );
            }

            // Time label
            painter.draw_text(
                self.time_scale.label(),
                Point::new(graph.x() + graph.width() - 20.0, graph.y() + graph.height() + 16.0),
                theme.colors.muted_foreground,
                10.0,
            );
        }

        // Draw series
        for series in &self.series {
            if series.data.is_empty() {
                continue;
            }

            let point_count = series.data.len().min(num_points);
            let data_start = series.data.len().saturating_sub(num_points);

            // Build path points
            let mut points: Vec<Point> = Vec::with_capacity(point_count);
            for (i, &value) in series.data[data_start..].iter().enumerate() {
                let x = graph.x() + graph.width() * (i as f32 / (num_points - 1).max(1) as f32);
                let y = graph.y() + graph.height() * (1.0 - (value / max_value) as f32).clamp(0.0, 1.0);
                points.push(Point::new(x, y));
            }

            // Draw fill
            if series.fill && points.len() >= 2 {
                let fill_color = series.color.with_alpha(0.2);
                // Simplified fill - draw triangles from bottom
                for i in 0..points.len() - 1 {
                    let p1 = points[i];
                    let p2 = points[i + 1];
                    let bottom_y = graph.y() + graph.height();

                    // Draw a quad as two triangles (simplified with rectangles)
                    let min_y = p1.y.min(p2.y);
                    let fill_rect = Rect::new(
                        p1.x,
                        min_y,
                        (p2.x - p1.x).max(1.0),
                        bottom_y - min_y,
                    );
                    painter.fill_rect(fill_rect, fill_color);
                }
            }

            // Draw line
            if points.len() >= 2 {
                for i in 0..points.len() - 1 {
                    let p1 = points[i];
                    let p2 = points[i + 1];

                    // Draw line segment as a thin rectangle (simplified)
                    let dx = p2.x - p1.x;
                    let dy = p2.y - p1.y;
                    let len = (dx * dx + dy * dy).sqrt();
                    if len > 0.0 {
                        // Approximate with a series of small rectangles
                        let steps = (len / 2.0).ceil() as usize;
                        for j in 0..steps {
                            let t = j as f32 / steps as f32;
                            let x = p1.x + dx * t;
                            let y = p1.y + dy * t;
                            painter.fill_rect(
                                Rect::new(x - series.line_width / 2.0, y - series.line_width / 2.0, series.line_width, series.line_width),
                                series.color,
                            );
                        }
                    }
                }
            }

            // Draw data points
            for point in &points {
                painter.fill_rect(
                    Rect::new(point.x - 2.0, point.y - 2.0, 4.0, 4.0),
                    series.color,
                );
            }
        }

        // Draw legend
        if self.show_legend && !self.series.is_empty() {
            let mut legend_x = graph.x();
            for series in &self.series {
                // Color box
                painter.fill_rect(
                    Rect::new(legend_x, rect.y() + 8.0, 12.0, 12.0),
                    series.color,
                );

                // Label
                let label = if self.show_current_value {
                    if let Some(value) = series.data.last() {
                        format!("{}: {}", series.label, format_speed(*value))
                    } else {
                        series.label.clone()
                    }
                } else {
                    series.label.clone()
                };

                painter.draw_text(
                    &label,
                    Point::new(legend_x + 16.0, rect.y() + 18.0),
                    theme.colors.foreground,
                    11.0,
                );

                legend_x += label.len() as f32 * 7.0 + 32.0;
            }
        }

        // Border
        painter.stroke_rect(rect, theme.colors.border, 1.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Move => {
                    // Could implement point hover detection here
                    let _ = ctx;
                }
                MouseEventKind::Leave => {
                    if self.hovered_point.is_some() {
                        self.hovered_point = None;
                        ctx.request_redraw();
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

/// Format a speed value as human-readable string.
fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.1} GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}
