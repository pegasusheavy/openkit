//! Bookmark bar widget for quick access to bookmarked pages.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A bookmark item.
#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub is_folder: bool,
}

impl Bookmark {
    pub fn new(id: impl Into<String>, title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            url: url.into(),
            favicon: None,
            is_folder: false,
        }
    }

    pub fn folder(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            url: String::new(),
            favicon: None,
            is_folder: true,
        }
    }

    pub fn favicon(mut self, favicon: impl Into<String>) -> Self {
        self.favicon = Some(favicon.into());
        self
    }

    pub fn display_title(&self, max_chars: usize) -> String {
        if self.title.chars().count() > max_chars {
            let mut title: String = self.title.chars().take(max_chars - 1).collect();
            title.push('…');
            title
        } else {
            self.title.clone()
        }
    }
}

/// Bookmark bar widget.
#[allow(clippy::type_complexity)]
pub struct BookmarkBar {
    base: WidgetBase,
    bookmarks: Vec<Bookmark>,
    hovered_id: Option<String>,
    on_click: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl BookmarkBar {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("bookmark-bar"),
            bookmarks: Vec::new(),
            hovered_id: None,
            on_click: None,
        }
    }

    pub fn bookmark(mut self, bookmark: Bookmark) -> Self {
        self.bookmarks.push(bookmark);
        self
    }

    pub fn bookmarks(mut self, bookmarks: Vec<Bookmark>) -> Self {
        self.bookmarks = bookmarks;
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where F: Fn(&str) + Send + Sync + 'static {
        self.on_click = Some(Box::new(handler));
        self
    }

    fn calculate_item_width(&self, bookmark: &Bookmark) -> f32 {
        let icon_width = 20.0;
        let padding = 16.0;
        let char_width = 7.0;
        let title_width = bookmark.display_title(20).chars().count() as f32 * char_width;
        icon_width + title_width + padding
    }
}

impl Default for BookmarkBar {
    fn default() -> Self { Self::new() }
}

impl Widget for BookmarkBar {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "bookmark-bar" }
    fn classes(&self) -> &ClassList { &self.base.classes }
    fn state(&self) -> WidgetState { self.base.state }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        Size::new(400.0, 28.0)
    }

    fn layout(&mut self, constraints: Constraints, _ctx: &LayoutContext) -> LayoutResult {
        LayoutResult::new(Size::new(constraints.max_width.min(1920.0), 28.0))
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = &ctx.style_ctx.theme;

        // Background
        painter.fill_rect(rect, theme.colors.card);
        painter.fill_rect(Rect::new(rect.x(), rect.y() + rect.height() - 1.0, rect.width(), 1.0), theme.colors.border);

        let mut x = rect.x() + 8.0;

        for bookmark in &self.bookmarks {
            let width = self.calculate_item_width(bookmark);
            let item_rect = Rect::new(x, rect.y() + 2.0, width, rect.height() - 4.0);
            let is_hovered = self.hovered_id.as_ref() == Some(&bookmark.id);

            if is_hovered {
                painter.fill_rounded_rect(item_rect, theme.colors.surface_hover, BorderRadius::all(4.0));
            }

            // Icon
            let icon = if bookmark.is_folder { "📁" } else { bookmark.favicon.as_deref().unwrap_or("🔗") };
            painter.draw_text(icon, Point::new(x + 8.0, rect.y() + rect.height() / 2.0 - 7.0), theme.colors.foreground, 12.0);

            // Title
            painter.draw_text(
                &bookmark.display_title(20),
                Point::new(x + 26.0, rect.y() + rect.height() / 2.0 - 6.0),
                theme.colors.foreground,
                12.0,
            );

            x += width + 4.0;
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        let bounds = self.base.bounds;

        if let Event::Mouse(mouse) = event {
            let pos = mouse.position;

            match mouse.kind {
                MouseEventKind::Move => {
                    if bounds.contains(pos) {
                        let mut x = bounds.x() + 8.0;
                        let mut found = None;
                        for bookmark in &self.bookmarks {
                            let width = self.calculate_item_width(bookmark);
                            if pos.x >= x && pos.x < x + width {
                                found = Some(bookmark.id.clone());
                                break;
                            }
                            x += width + 4.0;
                        }
                        if found != self.hovered_id {
                            self.hovered_id = found;
                            ctx.request_redraw();
                        }
                        return EventResult::Handled;
                    } else if self.hovered_id.is_some() {
                        self.hovered_id = None;
                        ctx.request_redraw();
                    }
                }
                MouseEventKind::Down => {
                    if mouse.button == Some(MouseButton::Left) {
                        if let Some(ref id) = self.hovered_id {
                            if let Some(bookmark) = self.bookmarks.iter().find(|b| &b.id == id) {
                                if !bookmark.is_folder {
                                    if let Some(ref handler) = self.on_click {
                                        handler(&bookmark.url);
                                    }
                                }
                            }
                            return EventResult::Handled;
                        }
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
