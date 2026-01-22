//! Download item widget for showing download progress.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Download state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadState {
    #[default]
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Download item data.
#[derive(Debug, Clone)]
pub struct DownloadData {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub state: DownloadState,
    pub progress: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bps: u64,
}

impl DownloadData {
    pub fn new(id: impl Into<String>, filename: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            filename: filename.into(),
            url: url.into(),
            state: DownloadState::Pending,
            progress: 0.0,
            downloaded_bytes: 0,
            total_bytes: None,
            speed_bps: 0,
        }
    }

    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB { format!("{:.1} GB", bytes as f64 / GB as f64) }
        else if bytes >= MB { format!("{:.1} MB", bytes as f64 / MB as f64) }
        else if bytes >= KB { format!("{:.1} KB", bytes as f64 / KB as f64) }
        else { format!("{} B", bytes) }
    }

    pub fn progress_text(&self) -> String {
        match self.state {
            DownloadState::Completed => Self::format_size(self.downloaded_bytes),
            DownloadState::Failed => "Failed".to_string(),
            DownloadState::Cancelled => "Cancelled".to_string(),
            DownloadState::Paused => format!("{} - Paused", Self::format_size(self.downloaded_bytes)),
            DownloadState::Downloading => {
                if let Some(total) = self.total_bytes {
                    format!("{} of {}", Self::format_size(self.downloaded_bytes), Self::format_size(total))
                } else {
                    Self::format_size(self.downloaded_bytes)
                }
            }
            DownloadState::Pending => "Starting...".to_string(),
        }
    }

    pub fn file_icon(&self) -> &'static str {
        let ext = self.filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "pdf" => "📄",
            "doc" | "docx" => "📝",
            "xls" | "xlsx" => "📊",
            "mp3" | "wav" | "flac" => "🎵",
            "mp4" | "mkv" | "avi" => "🎬",
            "jpg" | "jpeg" | "png" | "gif" => "🖼️",
            "zip" | "rar" | "7z" => "📦",
            "exe" | "msi" | "dmg" => "⚙️",
            _ => "📁",
        }
    }
}

/// Download item widget.
#[allow(clippy::type_complexity)]
pub struct DownloadItem {
    base: WidgetBase,
    data: DownloadData,
    is_hovered: bool,
    on_open: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_cancel: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl DownloadItem {
    pub fn new(data: DownloadData) -> Self {
        Self {
            base: WidgetBase::new().with_class("download-item"),
            data,
            is_hovered: false,
            on_open: None,
            on_cancel: None,
        }
    }

    pub fn on_open<F>(mut self, handler: F) -> Self
    where F: Fn(&str) + Send + Sync + 'static {
        self.on_open = Some(Box::new(handler));
        self
    }

    pub fn on_cancel<F>(mut self, handler: F) -> Self
    where F: Fn(&str) + Send + Sync + 'static {
        self.on_cancel = Some(Box::new(handler));
        self
    }
}

impl Widget for DownloadItem {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "download-item" }
    fn classes(&self) -> &ClassList { &self.base.classes }
    fn state(&self) -> WidgetState { self.base.state }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(320.0, 72.0)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        LayoutResult::new(Size::new(constraints.max_width.clamp(280.0, 400.0), 72.0))
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = &ctx.style_ctx.theme;

        let bg_color = if self.is_hovered { theme.colors.surface_hover } else { theme.colors.card };
        painter.fill_rounded_rect(rect, bg_color, BorderRadius::all(8.0));

        // File icon
        painter.draw_text(self.data.file_icon(), Point::new(rect.x() + 12.0, rect.y() + rect.height() / 2.0 - 12.0), theme.colors.foreground, 24.0);

        // Filename
        let text_x = rect.x() + 48.0;
        let filename = if self.data.filename.chars().count() > 30 {
            let mut s: String = self.data.filename.chars().take(27).collect();
            s.push_str("...");
            s
        } else {
            self.data.filename.clone()
        };
        painter.draw_text(&filename, Point::new(text_x, rect.y() + 14.0), theme.colors.foreground, 13.0);

        // Progress text
        painter.draw_text(&self.data.progress_text(), Point::new(text_x, rect.y() + 32.0), theme.colors.muted_foreground, 11.0);

        // Progress bar
        if self.data.state == DownloadState::Downloading || self.data.state == DownloadState::Paused {
            let bar_y = rect.y() + 50.0;
            let bar_width = rect.width() - text_x + rect.x() - 60.0;

            painter.fill_rounded_rect(Rect::new(text_x, bar_y, bar_width, 4.0), theme.colors.border, BorderRadius::all(2.0));

            let progress_width = bar_width * self.data.progress;
            let progress_color = if self.data.state == DownloadState::Paused { theme.colors.warning } else { theme.colors.accent };
            painter.fill_rounded_rect(Rect::new(text_x, bar_y, progress_width, 4.0), progress_color, BorderRadius::all(2.0));
        }

        // Cancel button
        if self.is_hovered && self.data.state == DownloadState::Downloading {
            let btn_x = rect.x() + rect.width() - 36.0;
            painter.draw_text("✕", Point::new(btn_x, rect.y() + rect.height() / 2.0 - 8.0), theme.colors.muted_foreground, 16.0);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let bounds = self.base.bounds;

        if let Event::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Move => {
                    let was_hovered = self.is_hovered;
                    self.is_hovered = bounds.contains(mouse.position);
                    if was_hovered != self.is_hovered {
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down => {
                    if mouse.button == Some(MouseButton::Left) && self.is_hovered
                        && self.data.state == DownloadState::Completed {
                            if let Some(ref handler) = self.on_open {
                                handler(&self.data.id);
                            }
                            return EventResult::Handled;
                        }
                }
                _ => {}
            }
        }

        EventResult::Ignored
    }

    fn bounds(&self) -> Rect { self.base.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.base.bounds = bounds; }
}
