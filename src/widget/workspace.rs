//! Workspace/virtual desktop switcher widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A workspace/virtual desktop.
#[derive(Debug, Clone)]
pub struct WorkspaceItem {
    pub id: String,
    pub name: Option<String>,
    pub thumbnail: Option<String>,
    pub window_count: usize,
}

impl WorkspaceItem {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), name: None, thumbnail: None, window_count: 0 }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into()); self
    }

    pub fn windows(mut self, count: usize) -> Self {
        self.window_count = count; self
    }
}

/// Workspace switcher widget.
#[allow(clippy::type_complexity)]
pub struct WorkspaceSwitcher {
    base: WidgetBase,
    workspaces: Vec<WorkspaceItem>,
    active_id: Option<String>,
    hovered: Option<String>,
    on_switch: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl WorkspaceSwitcher {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("workspace-switcher"),
            workspaces: Vec::new(),
            active_id: None,
            hovered: None,
            on_switch: None,
        }
    }

    pub fn workspace(mut self, ws: WorkspaceItem) -> Self {
        if self.active_id.is_none() { self.active_id = Some(ws.id.clone()); }
        self.workspaces.push(ws); self
    }

    pub fn active(mut self, id: impl Into<String>) -> Self {
        self.active_id = Some(id.into()); self
    }

    pub fn on_switch<F: Fn(&str) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_switch = Some(Box::new(f)); self
    }

    fn workspace_rect(&self, index: usize) -> Rect {
        let size = 60.0;
        let gap = 8.0;
        Rect::new(
            self.base.bounds.x() + (index as f32) * (size + gap),
            self.base.bounds.y(),
            size, size * 0.6,
        )
    }
}

impl Default for WorkspaceSwitcher { fn default() -> Self { Self::new() } }

impl Widget for WorkspaceSwitcher {
    fn id(&self) -> WidgetId { self.base.id }
    fn type_name(&self) -> &'static str { "workspace-switcher" }
    fn element_id(&self) -> Option<&str> { self.base.element_id.as_deref() }
    fn classes(&self) -> &ClassList { &self.base.classes }
    fn state(&self) -> WidgetState { self.base.state }

    fn intrinsic_size(&self, _ctx: &LayoutContext) -> Size {
        let w = self.workspaces.len() as f32 * 68.0 - 8.0;
        Size::new(w.max(0.0), 44.0)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, _rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let radius = BorderRadius::all(4.0);

        for (i, ws) in self.workspaces.iter().enumerate() {
            let r = self.workspace_rect(i);
            let active = self.active_id.as_ref() == Some(&ws.id);
            let hovered = self.hovered.as_ref() == Some(&ws.id);

            let bg = if active { theme.colors.accent }
                else if hovered { theme.colors.accent.with_alpha(0.3) }
                else { theme.colors.muted };
            painter.fill_rounded_rect(r, bg, radius);

            if active { painter.stroke_rect(r, theme.colors.ring, 2.0); }

            let num = format!("{}", i + 1);
            let fg = if active { theme.colors.accent_foreground } else { theme.colors.foreground };
            painter.draw_text(&num, Point::new(r.x() + r.width()/2.0 - 4.0, r.y() + r.height()/2.0 + 5.0), fg, 14.0);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(m) = event {
            match m.kind {
                MouseEventKind::Move => {
                    let new = (0..self.workspaces.len())
                        .find(|&i| self.workspace_rect(i).contains(m.position))
                        .map(|i| self.workspaces[i].id.clone());
                    if new != self.hovered { self.hovered = new; ctx.request_redraw(); }
                }
                MouseEventKind::Up if m.button == Some(MouseButton::Left) => {
                    if let Some(i) = (0..self.workspaces.len())
                        .find(|&i| self.workspace_rect(i).contains(m.position)) {
                        let id = self.workspaces[i].id.clone();
                        self.active_id = Some(id.clone());
                        if let Some(h) = &self.on_switch { h(&id); }
                        ctx.request_redraw();
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
