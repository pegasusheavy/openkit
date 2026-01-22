//! Tree view widget for hierarchical data display.
//!
//! Ideal for file browsers, directory trees, and nested data structures.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, Key, KeyEventKind};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A tree node.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Node identifier
    pub id: String,
    /// Node label
    pub label: String,
    /// Node icon
    pub icon: Option<String>,
    /// Child nodes
    pub children: Vec<TreeNode>,
    /// Whether this node is expanded
    pub expanded: bool,
    /// Whether this node is selected
    pub selected: bool,
    /// Whether this node is disabled
    pub disabled: bool,
    /// Optional data attached to this node
    pub data: Option<String>,
}

impl TreeNode {
    /// Create a new tree node.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
            selected: false,
            disabled: false,
            data: None,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Add a child node.
    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// Set children.
    pub fn children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }

    /// Set expanded state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set attached data.
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Check if this node has children.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get the default icon based on expansion state.
    pub fn default_icon(&self) -> &'static str {
        if self.has_children() {
            if self.expanded { "📂" } else { "📁" }
        } else {
            "📄"
        }
    }
}

/// Tree view selection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeSelectionMode {
    /// No selection
    None,
    /// Single node selection (default)
    #[default]
    Single,
    /// Multiple node selection
    Multiple,
}

/// Tree view widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::widget::tree_view::*;
///
/// let tree = TreeView::new()
///     .node(
///         TreeNode::new("root", "Documents")
///             .icon("📁")
///             .expanded(true)
///             .child(TreeNode::new("file1", "Report.pdf").icon("📄"))
///             .child(
///                 TreeNode::new("folder1", "Images")
///                     .child(TreeNode::new("img1", "photo.jpg").icon("🖼️"))
///                     .child(TreeNode::new("img2", "screenshot.png").icon("🖼️"))
///             )
///     )
///     .on_select(|id| println!("Selected: {}", id))
///     .on_expand(|id, expanded| println!("{} {}", id, if expanded { "expanded" } else { "collapsed" }));
/// ```
#[allow(clippy::type_complexity)]
pub struct TreeView {
    base: WidgetBase,
    nodes: Vec<TreeNode>,
    item_height: f32,
    indent_size: f32,
    selection_mode: TreeSelectionMode,
    scroll_offset: f32,
    show_lines: bool,
    show_icons: bool,
    hovered_node: Option<String>,
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_expand: Option<Box<dyn Fn(&str, bool) + Send + Sync>>,
    on_activate: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl TreeView {
    /// Create a new tree view.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("tree-view"),
            nodes: Vec::new(),
            item_height: 28.0,
            indent_size: 20.0,
            selection_mode: TreeSelectionMode::default(),
            scroll_offset: 0.0,
            show_lines: false,
            show_icons: true,
            hovered_node: None,
            on_select: None,
            on_expand: None,
            on_activate: None,
        }
    }

    /// Add a root node.
    pub fn node(mut self, node: TreeNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Set all root nodes.
    pub fn nodes(mut self, nodes: Vec<TreeNode>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Set item height.
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// Set indent size.
    pub fn indent_size(mut self, size: f32) -> Self {
        self.indent_size = size;
        self
    }

    /// Set selection mode.
    pub fn selection_mode(mut self, mode: TreeSelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Show/hide connecting lines.
    pub fn show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }

    /// Show/hide icons.
    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    /// Set select handler.
    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Set expand/collapse handler.
    pub fn on_expand<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, bool) + Send + Sync + 'static,
    {
        self.on_expand = Some(Box::new(handler));
        self
    }

    /// Set activate (double-click) handler.
    pub fn on_activate<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_activate = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Select a node by ID.
    pub fn select(&mut self, id: &str) {
        if self.selection_mode == TreeSelectionMode::None {
            return;
        }

        // Clear other selections in single mode
        if self.selection_mode == TreeSelectionMode::Single {
            self.clear_selection_recursive(&mut self.nodes.clone());
        }

        self.select_node_recursive(&mut self.nodes.clone(), id);

        // Now apply changes
        fn apply_selection(nodes: &mut [TreeNode], target_id: &str, single: bool) {
            for node in nodes {
                if single {
                    node.selected = node.id == target_id;
                } else if node.id == target_id {
                    node.selected = true;
                }
                apply_selection(&mut node.children, target_id, single);
            }
        }

        let single = self.selection_mode == TreeSelectionMode::Single;
        apply_selection(&mut self.nodes, id, single);

        if let Some(handler) = &self.on_select {
            handler(id);
        }
    }

    fn clear_selection_recursive(&self, nodes: &mut [TreeNode]) {
        for node in nodes {
            node.selected = false;
            self.clear_selection_recursive(&mut node.children);
        }
    }

    fn select_node_recursive(&self, nodes: &mut [TreeNode], id: &str) {
        for node in nodes {
            if node.id == id {
                node.selected = true;
            }
            self.select_node_recursive(&mut node.children, id);
        }
    }

    /// Toggle expansion of a node.
    pub fn toggle_expand(&mut self, id: &str) {
        fn toggle(nodes: &mut [TreeNode], id: &str) -> Option<bool> {
            for node in nodes {
                if node.id == id {
                    node.expanded = !node.expanded;
                    return Some(node.expanded);
                }
                if let Some(expanded) = toggle(&mut node.children, id) {
                    return Some(expanded);
                }
            }
            None
        }

        if let Some(expanded) = toggle(&mut self.nodes, id) {
            if let Some(handler) = &self.on_expand {
                handler(id, expanded);
            }
        }
    }

    /// Expand all nodes.
    pub fn expand_all(&mut self) {
        fn expand(nodes: &mut [TreeNode]) {
            for node in nodes {
                node.expanded = true;
                expand(&mut node.children);
            }
        }
        expand(&mut self.nodes);
    }

    /// Collapse all nodes.
    pub fn collapse_all(&mut self) {
        fn collapse(nodes: &mut [TreeNode]) {
            for node in nodes {
                node.expanded = false;
                collapse(&mut node.children);
            }
        }
        collapse(&mut self.nodes);
    }

    /// Get selected node IDs.
    pub fn selected(&self) -> Vec<String> {
        fn collect(nodes: &[TreeNode], selected: &mut Vec<String>) {
            for node in nodes {
                if node.selected {
                    selected.push(node.id.clone());
                }
                collect(&node.children, selected);
            }
        }

        let mut result = Vec::new();
        collect(&self.nodes, &mut result);
        result
    }

    /// Count visible nodes.
    fn visible_count(&self) -> usize {
        fn count(nodes: &[TreeNode]) -> usize {
            let mut total = nodes.len();
            for node in nodes {
                if node.expanded {
                    total += count(&node.children);
                }
            }
            total
        }
        count(&self.nodes)
    }

    fn node_at_point(&self, point: Point) -> Option<String> {
        let bounds = self.base.bounds;
        if !bounds.contains(point) {
            return None;
        }

        let relative_y = point.y - bounds.y() + self.scroll_offset;
        let row = (relative_y / self.item_height) as usize;

        fn find_at_index(nodes: &[TreeNode], index: &mut usize, target: usize) -> Option<String> {
            for node in nodes {
                if *index == target {
                    return Some(node.id.clone());
                }
                *index += 1;
                if node.expanded {
                    if let Some(id) = find_at_index(&node.children, index, target) {
                        return Some(id);
                    }
                }
            }
            None
        }

        let mut idx = 0;
        find_at_index(&self.nodes, &mut idx, row)
    }

    fn is_on_expander(&self, point: Point, depth: usize) -> bool {
        let bounds = self.base.bounds;
        let expander_x = bounds.x() + depth as f32 * self.indent_size + 4.0;
        point.x >= expander_x && point.x <= expander_x + 16.0
    }
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TreeView {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "tree-view"
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
        let height = (self.visible_count() as f32 * self.item_height).min(400.0);
        Size::new(250.0, height.max(100.0))
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Background
        painter.fill_rect(rect, theme.colors.background);

        // Draw nodes recursively
        fn draw_nodes(
            painter: &mut Painter,
            nodes: &[TreeNode],
            rect: Rect,
            item_height: f32,
            indent_size: f32,
            depth: usize,
            row: &mut usize,
            scroll_offset: f32,
            show_lines: bool,
            show_icons: bool,
            hovered: &Option<String>,
            ctx: &PaintContext,
        ) {
            let theme = ctx.style_ctx.theme;

            for node in nodes {
                let y = rect.y() + (*row as f32) * item_height - scroll_offset;

                // Skip if not visible
                if y + item_height < rect.y() || y > rect.y() + rect.height() {
                    *row += 1;
                    if node.expanded {
                        skip_count(&node.children, row);
                    }
                    continue;
                }

                let item_rect = Rect::new(rect.x(), y, rect.width(), item_height);
                let x = rect.x() + depth as f32 * indent_size;

                // Background
                let bg_color = if node.selected {
                    theme.colors.accent.with_alpha(0.2)
                } else if hovered.as_ref() == Some(&node.id) {
                    theme.colors.surface_hover
                } else {
                    theme.colors.background
                };
                painter.fill_rect(item_rect, bg_color);

                // Connecting lines
                if show_lines && depth > 0 {
                    let line_x = rect.x() + (depth as f32 - 0.5) * indent_size;
                    painter.fill_rect(
                        Rect::new(line_x, y, 1.0, item_height),
                        theme.colors.border,
                    );
                    painter.fill_rect(
                        Rect::new(line_x, y + item_height / 2.0, indent_size / 2.0, 1.0),
                        theme.colors.border,
                    );
                }

                // Expander
                if node.has_children() {
                    let expander_icon = if node.expanded { "▼" } else { "▶" };
                    painter.draw_text(
                        expander_icon,
                        Point::new(x + 4.0, y + item_height / 2.0 + 4.0),
                        theme.colors.muted_foreground,
                        10.0,
                    );
                }

                let mut content_x = x + 20.0;

                // Icon
                if show_icons {
                    let icon = node.icon.as_deref().unwrap_or_else(|| node.default_icon());
                    painter.draw_text(
                        icon,
                        Point::new(content_x, y + item_height / 2.0 + 4.0),
                        theme.colors.foreground,
                        14.0,
                    );
                    content_x += 20.0;
                }

                // Label
                let text_color = if node.disabled {
                    theme.colors.muted_foreground
                } else if node.selected {
                    theme.colors.foreground
                } else {
                    theme.colors.foreground
                };
                painter.draw_text(
                    &node.label,
                    Point::new(content_x, y + item_height / 2.0 + 4.0),
                    text_color,
                    13.0,
                );

                *row += 1;

                // Draw children if expanded
                if node.expanded {
                    draw_nodes(
                        painter,
                        &node.children,
                        rect,
                        item_height,
                        indent_size,
                        depth + 1,
                        row,
                        scroll_offset,
                        show_lines,
                        show_icons,
                        hovered,
                        ctx,
                    );
                }
            }
        }

        fn skip_count(nodes: &[TreeNode], row: &mut usize) {
            for node in nodes {
                *row += 1;
                if node.expanded {
                    skip_count(&node.children, row);
                }
            }
        }

        let mut row = 0;
        draw_nodes(
            painter,
            &self.nodes,
            rect,
            self.item_height,
            self.indent_size,
            0,
            &mut row,
            self.scroll_offset,
            self.show_lines,
            self.show_icons,
            &self.hovered_node,
            ctx,
        );

        // Border
        painter.stroke_rect(rect, theme.colors.border, 1.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Move => {
                        let new_hovered = self.node_at_point(mouse.position);
                        if new_hovered != self.hovered_node {
                            self.hovered_node = new_hovered;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Leave => {
                        if self.hovered_node.is_some() {
                            self.hovered_node = None;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Down if mouse.button == Some(MouseButton::Left) => {
                        if let Some(id) = self.node_at_point(mouse.position) {
                            // Check if clicking on expander
                            // For simplicity, toggle expand on any click if has children
                            fn find_node<'a>(nodes: &'a [TreeNode], id: &str) -> Option<&'a TreeNode> {
                                for node in nodes {
                                    if node.id == id {
                                        return Some(node);
                                    }
                                    if let Some(found) = find_node(&node.children, id) {
                                        return Some(found);
                                    }
                                }
                                None
                            }

                            if let Some(node) = find_node(&self.nodes, &id) {
                                if node.has_children() && self.is_on_expander(mouse.position, 0) {
                                    self.toggle_expand(&id);
                                } else {
                                    self.select(&id);
                                }
                            } else {
                                self.select(&id);
                            }

                            ctx.request_focus(self.base.id);
                            ctx.request_redraw();
                            return EventResult::Handled;
                        }
                    }
                    _ => {}
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Down && self.base.state.focused => {
                match key.key {
                    Key::Enter => {
                        let selected = self.selected();
                        if let Some(id) = selected.first() {
                            if let Some(handler) = &self.on_activate {
                                handler(id);
                            }
                            return EventResult::Handled;
                        }
                    }
                    Key::Left => {
                        // Collapse selected node or go to parent
                        let selected = self.selected();
                        if let Some(id) = selected.first() {
                            self.toggle_expand(id); // Simplified - should check if expanded
                            ctx.request_redraw();
                        }
                        return EventResult::Handled;
                    }
                    Key::Right => {
                        // Expand selected node
                        let selected = self.selected();
                        if let Some(id) = selected.first() {
                            self.toggle_expand(id);
                            ctx.request_redraw();
                        }
                        return EventResult::Handled;
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
