//! Piece map widget for visualizing torrent piece download status.
//!
//! A bitfield visualization widget commonly used in torrent clients to show
//! which pieces have been downloaded.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind};
use crate::geometry::{Color, Point, Rect, Size, BorderRadius};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Piece state in the bitfield.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PieceState {
    /// Piece not yet downloaded
    #[default]
    Missing,
    /// Piece is being downloaded
    Downloading,
    /// Piece has been downloaded and verified
    Complete,
    /// Piece hash verification failed
    Failed,
    /// Piece is being requested
    Requested,
}

/// Piece map display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PieceMapMode {
    /// Dense grid view showing all pieces
    #[default]
    Grid,
    /// Linear bar showing aggregated progress
    Bar,
    /// Compact mode with fewer details
    Compact,
}

/// Piece map widget for visualizing download progress.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::widget::piece_map::*;
///
/// // Create a piece map with 1000 pieces
/// let mut pieces = vec![PieceState::Missing; 1000];
/// pieces[0..500].fill(PieceState::Complete);
/// pieces[500..510].fill(PieceState::Downloading);
///
/// let map = PieceMap::new()
///     .pieces(pieces)
///     .mode(PieceMapMode::Grid)
///     .on_piece_hover(|idx, state| {
///         println!("Piece {}: {:?}", idx, state);
///     });
/// ```
#[allow(clippy::type_complexity)]
pub struct PieceMap {
    base: WidgetBase,
    pieces: Vec<PieceState>,
    mode: PieceMapMode,
    piece_size: f32,
    gap: f32,
    hovered_piece: Option<usize>,
    show_tooltip: bool,
    complete_color: Option<Color>,
    downloading_color: Option<Color>,
    missing_color: Option<Color>,
    failed_color: Option<Color>,
    on_piece_hover: Option<Box<dyn Fn(usize, PieceState) + Send + Sync>>,
}

impl PieceMap {
    /// Create a new piece map.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("piece-map"),
            pieces: Vec::new(),
            mode: PieceMapMode::default(),
            piece_size: 4.0,
            gap: 1.0,
            hovered_piece: None,
            show_tooltip: true,
            complete_color: None,
            downloading_color: None,
            missing_color: None,
            failed_color: None,
            on_piece_hover: None,
        }
    }

    /// Set the pieces.
    pub fn pieces(mut self, pieces: Vec<PieceState>) -> Self {
        self.pieces = pieces;
        self
    }

    /// Set from a bitfield (bytes where each bit represents a piece).
    pub fn from_bitfield(mut self, bitfield: &[u8], total_pieces: usize) -> Self {
        self.pieces = Vec::with_capacity(total_pieces);
        for i in 0..total_pieces {
            let byte_idx = i / 8;
            let bit_idx = 7 - (i % 8);
            let has_piece = if byte_idx < bitfield.len() {
                (bitfield[byte_idx] >> bit_idx) & 1 == 1
            } else {
                false
            };
            self.pieces.push(if has_piece {
                PieceState::Complete
            } else {
                PieceState::Missing
            });
        }
        self
    }

    /// Set the display mode.
    pub fn mode(mut self, mode: PieceMapMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the piece size in pixels.
    pub fn piece_size(mut self, size: f32) -> Self {
        self.piece_size = size;
        self
    }

    /// Set the gap between pieces.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Enable/disable tooltip.
    pub fn show_tooltip(mut self, show: bool) -> Self {
        self.show_tooltip = show;
        self
    }

    /// Set custom colors.
    pub fn complete_color(mut self, color: Color) -> Self {
        self.complete_color = Some(color);
        self
    }

    pub fn downloading_color(mut self, color: Color) -> Self {
        self.downloading_color = Some(color);
        self
    }

    pub fn missing_color(mut self, color: Color) -> Self {
        self.missing_color = Some(color);
        self
    }

    pub fn failed_color(mut self, color: Color) -> Self {
        self.failed_color = Some(color);
        self
    }

    /// Set hover handler.
    pub fn on_piece_hover<F>(mut self, handler: F) -> Self
    where
        F: Fn(usize, PieceState) + Send + Sync + 'static,
    {
        self.on_piece_hover = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Get the completion percentage.
    pub fn completion(&self) -> f32 {
        if self.pieces.is_empty() {
            return 0.0;
        }
        let complete = self.pieces.iter().filter(|p| **p == PieceState::Complete).count();
        complete as f32 / self.pieces.len() as f32
    }

    /// Get statistics.
    pub fn stats(&self) -> PieceStats {
        let mut stats = PieceStats::default();
        for piece in &self.pieces {
            match piece {
                PieceState::Complete => stats.complete += 1,
                PieceState::Downloading => stats.downloading += 1,
                PieceState::Missing => stats.missing += 1,
                PieceState::Failed => stats.failed += 1,
                PieceState::Requested => stats.requested += 1,
            }
        }
        stats.total = self.pieces.len();
        stats
    }

    fn piece_at_point(&self, point: Point) -> Option<usize> {
        let bounds = self.base.bounds;
        if !bounds.contains(point) {
            return None;
        }

        let cell_size = self.piece_size + self.gap;
        let cols = ((bounds.width() - self.gap) / cell_size) as usize;
        if cols == 0 {
            return None;
        }

        let rel_x = point.x - bounds.x();
        let rel_y = point.y - bounds.y();

        let col = (rel_x / cell_size) as usize;
        let row = (rel_y / cell_size) as usize;

        let index = row * cols + col;
        if index < self.pieces.len() {
            Some(index)
        } else {
            None
        }
    }

    fn piece_color(&self, state: PieceState, theme: &crate::theme::ThemeData) -> Color {
        match state {
            PieceState::Complete => self.complete_color.unwrap_or(theme.colors.success),
            PieceState::Downloading => self.downloading_color.unwrap_or(theme.colors.primary),
            PieceState::Missing => self.missing_color.unwrap_or(theme.colors.muted),
            PieceState::Failed => self.failed_color.unwrap_or(theme.colors.destructive),
            PieceState::Requested => self.downloading_color.unwrap_or(theme.colors.warning),
        }
    }
}

/// Piece statistics.
#[derive(Debug, Clone, Copy, Default)]
pub struct PieceStats {
    pub total: usize,
    pub complete: usize,
    pub downloading: usize,
    pub missing: usize,
    pub failed: usize,
    pub requested: usize,
}

impl Default for PieceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for PieceMap {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "piece-map"
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
        match self.mode {
            PieceMapMode::Grid => {
                // Calculate grid dimensions
                let cell_size = self.piece_size + self.gap;
                let cols = 50; // Default columns
                let rows = (self.pieces.len() as f32 / cols as f32).ceil() as usize;
                Size::new(
                    cols as f32 * cell_size + self.gap,
                    (rows as f32 * cell_size + self.gap).min(200.0),
                )
            }
            PieceMapMode::Bar | PieceMapMode::Compact => {
                Size::new(300.0, if self.mode == PieceMapMode::Compact { 8.0 } else { 20.0 })
            }
        }
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        painter.fill_rounded_rect(rect, theme.colors.surface, BorderRadius::all(4.0));

        if self.pieces.is_empty() {
            // Empty state
            painter.draw_text(
                "No piece data",
                Point::new(rect.x() + 8.0, rect.y() + rect.height() / 2.0 + 4.0),
                theme.colors.muted_foreground,
                11.0,
            );
            return;
        }

        match self.mode {
            PieceMapMode::Grid => {
                self.paint_grid(painter, rect, ctx);
            }
            PieceMapMode::Bar => {
                self.paint_bar(painter, rect, ctx);
            }
            PieceMapMode::Compact => {
                self.paint_compact(painter, rect, ctx);
            }
        }

        // Border
        painter.stroke_rect(rect, theme.colors.border, 1.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Move => {
                    let new_hovered = self.piece_at_point(mouse.position);
                    if new_hovered != self.hovered_piece {
                        self.hovered_piece = new_hovered;
                        if let (Some(idx), Some(handler)) = (new_hovered, &self.on_piece_hover) {
                            handler(idx, self.pieces[idx]);
                        }
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Leave => {
                    if self.hovered_piece.is_some() {
                        self.hovered_piece = None;
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

impl PieceMap {
    fn paint_grid(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let cell_size = self.piece_size + self.gap;
        let cols = ((rect.width() - self.gap) / cell_size) as usize;
        if cols == 0 {
            return;
        }

        for (i, &state) in self.pieces.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = rect.x() + self.gap + col as f32 * cell_size;
            let y = rect.y() + self.gap + row as f32 * cell_size;

            if y + self.piece_size > rect.y() + rect.height() {
                break; // Out of view
            }

            let color = self.piece_color(state, theme);
            let piece_rect = Rect::new(x, y, self.piece_size, self.piece_size);

            // Highlight hovered piece
            if self.hovered_piece == Some(i) {
                let highlight = Rect::new(
                    x - 1.0,
                    y - 1.0,
                    self.piece_size + 2.0,
                    self.piece_size + 2.0,
                );
                painter.fill_rect(highlight, theme.colors.accent.with_alpha(0.5));
            }

            painter.fill_rect(piece_rect, color);
        }

        // Tooltip for hovered piece
        if self.show_tooltip {
            if let Some(idx) = self.hovered_piece {
                let state = self.pieces[idx];
                let state_text = match state {
                    PieceState::Complete => "Complete",
                    PieceState::Downloading => "Downloading",
                    PieceState::Missing => "Missing",
                    PieceState::Failed => "Failed",
                    PieceState::Requested => "Requested",
                };
                let tooltip = format!("Piece {}: {}", idx, state_text);
                let tooltip_rect = Rect::new(
                    rect.x() + 4.0,
                    rect.y() + rect.height() - 20.0,
                    tooltip.len() as f32 * 7.0 + 16.0,
                    18.0,
                );
                painter.fill_rounded_rect(tooltip_rect, theme.colors.card, BorderRadius::all(4.0));
                painter.draw_text(
                    &tooltip,
                    Point::new(tooltip_rect.x() + 8.0, tooltip_rect.y() + 13.0),
                    theme.colors.foreground,
                    11.0,
                );
            }
        }
    }

    fn paint_bar(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let padding = 4.0;
        let bar_rect = Rect::new(
            rect.x() + padding,
            rect.y() + padding,
            rect.width() - padding * 2.0,
            rect.height() - padding * 2.0,
        );

        // Background
        painter.fill_rounded_rect(bar_rect, theme.colors.muted, BorderRadius::all(4.0));

        if self.pieces.is_empty() {
            return;
        }

        // Draw segments proportionally
        let segment_width = bar_rect.width() / self.pieces.len() as f32;
        for (i, &state) in self.pieces.iter().enumerate() {
            if state == PieceState::Missing {
                continue;
            }
            let color = self.piece_color(state, theme);
            let seg_x = bar_rect.x() + i as f32 * segment_width;
            painter.fill_rect(
                Rect::new(seg_x, bar_rect.y(), segment_width.max(1.0), bar_rect.height()),
                color,
            );
        }

        // Percentage label
        let pct = self.completion() * 100.0;
        let label = format!("{:.1}%", pct);
        painter.draw_text(
            &label,
            Point::new(bar_rect.x() + bar_rect.width() - 40.0, bar_rect.y() + bar_rect.height() / 2.0 + 4.0),
            theme.colors.foreground,
            11.0,
        );
    }

    fn paint_compact(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Simple progress bar
        painter.fill_rounded_rect(rect, theme.colors.muted, BorderRadius::all(rect.height() / 2.0));

        let fill_width = rect.width() * self.completion();
        painter.fill_rounded_rect(
            Rect::new(rect.x(), rect.y(), fill_width, rect.height()),
            theme.colors.success,
            BorderRadius::all(rect.height() / 2.0),
        );
    }
}
