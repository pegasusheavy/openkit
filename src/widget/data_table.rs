//! Data table widget for displaying tabular data with sortable columns.
//!
//! A flexible table widget ideal for torrent clients, file browsers, and data-heavy applications.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton, Key, KeyEventKind};
use crate::geometry::{Point, Rect, Size, BorderRadius};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// Column definition for the data table.
#[derive(Debug, Clone)]
pub struct TableColumn {
    /// Column identifier
    pub id: String,
    /// Column header text
    pub header: String,
    /// Column width (fixed or flex)
    pub width: ColumnWidth,
    /// Whether this column is sortable
    pub sortable: bool,
    /// Text alignment
    pub align: ColumnAlign,
}

impl TableColumn {
    /// Create a new column.
    pub fn new(id: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width: ColumnWidth::Flex(1.0),
            sortable: true,
            align: ColumnAlign::Left,
        }
    }

    /// Set fixed width.
    pub fn fixed_width(mut self, width: f32) -> Self {
        self.width = ColumnWidth::Fixed(width);
        self
    }

    /// Set flex width.
    pub fn flex(mut self, flex: f32) -> Self {
        self.width = ColumnWidth::Flex(flex);
        self
    }

    /// Set minimum width.
    pub fn min_width(mut self, min: f32) -> Self {
        self.width = ColumnWidth::MinMax { min, max: f32::INFINITY };
        self
    }

    /// Set sortable.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Set alignment.
    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }
}

/// Column width specification.
#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    /// Fixed width in pixels
    Fixed(f32),
    /// Flexible width (proportion of remaining space)
    Flex(f32),
    /// Minimum/maximum bounds
    MinMax { min: f32, max: f32 },
}

/// Column text alignment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColumnAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    }
}

/// A single cell value.
#[derive(Debug, Clone)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Progress(f32),
    Badge { text: String, variant: BadgeVariant },
    Icon(String),
    Custom(Box<dyn CellRenderer>),
}

impl CellValue {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    pub fn number(n: f64) -> Self {
        Self::Number(n)
    }

    pub fn progress(p: f32) -> Self {
        Self::Progress(p.clamp(0.0, 1.0))
    }

    pub fn badge(text: impl Into<String>, variant: BadgeVariant) -> Self {
        Self::Badge { text: text.into(), variant }
    }
}

/// Badge variant for status cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

/// Custom cell renderer trait.
pub trait CellRenderer: std::fmt::Debug + Send + Sync {
    fn render(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext);
    fn clone_box(&self) -> Box<dyn CellRenderer>;
}

impl Clone for Box<dyn CellRenderer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// A row in the data table.
#[derive(Debug, Clone)]
pub struct TableRow {
    /// Row identifier
    pub id: String,
    /// Cell values indexed by column ID
    pub cells: std::collections::HashMap<String, CellValue>,
    /// Whether this row is selected
    pub selected: bool,
    /// Whether this row is disabled
    pub disabled: bool,
    /// Optional row icon
    pub icon: Option<String>,
}

impl TableRow {
    /// Create a new row.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            cells: std::collections::HashMap::new(),
            selected: false,
            disabled: false,
            icon: None,
        }
    }

    /// Add a cell value.
    pub fn cell(mut self, column_id: impl Into<String>, value: CellValue) -> Self {
        self.cells.insert(column_id.into(), value);
        self
    }

    /// Set row icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Selection mode for the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableSelectionMode {
    /// No selection
    None,
    /// Single row selection (default)
    #[default]
    Single,
    /// Multiple row selection
    Multiple,
}

/// Data table widget.
///
/// # Example
///
/// ```rust,ignore
/// use openkit::widget::data_table::*;
///
/// let table = DataTable::new()
///     .column(TableColumn::new("name", "Name").flex(2.0))
///     .column(TableColumn::new("size", "Size").fixed_width(100.0).align(ColumnAlign::Right))
///     .column(TableColumn::new("progress", "Progress").fixed_width(150.0))
///     .column(TableColumn::new("status", "Status").fixed_width(100.0))
///     .row(
///         TableRow::new("torrent1")
///             .icon("📥")
///             .cell("name", CellValue::text("Ubuntu 24.04 LTS"))
///             .cell("size", CellValue::text("4.2 GB"))
///             .cell("progress", CellValue::progress(0.75))
///             .cell("status", CellValue::badge("Downloading", BadgeVariant::Primary))
///     )
///     .on_select(|id| println!("Selected: {}", id))
///     .on_sort(|col, dir| println!("Sort: {} {:?}", col, dir));
/// ```
#[allow(clippy::type_complexity)]
pub struct DataTable {
    base: WidgetBase,
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    row_height: f32,
    header_height: f32,
    selection_mode: TableSelectionMode,
    sort_column: Option<String>,
    sort_direction: SortDirection,
    scroll_offset: f32,
    hovered_row: Option<usize>,
    hovered_header: Option<usize>,
    show_row_numbers: bool,
    zebra_stripes: bool,
    computed_widths: Vec<f32>,
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_activate: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_sort: Option<Box<dyn Fn(&str, SortDirection) + Send + Sync>>,
    on_context_menu: Option<Box<dyn Fn(&str, Point) + Send + Sync>>,
}

impl DataTable {
    /// Create a new data table.
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new().with_class("data-table"),
            columns: Vec::new(),
            rows: Vec::new(),
            row_height: 40.0,
            header_height: 36.0,
            selection_mode: TableSelectionMode::default(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            scroll_offset: 0.0,
            hovered_row: None,
            hovered_header: None,
            show_row_numbers: false,
            zebra_stripes: true,
            computed_widths: Vec::new(),
            on_select: None,
            on_activate: None,
            on_sort: None,
            on_context_menu: None,
        }
    }

    /// Add a column.
    pub fn column(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Set all columns.
    pub fn columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns = columns;
        self
    }

    /// Add a row.
    pub fn row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Set all rows.
    pub fn rows(mut self, rows: Vec<TableRow>) -> Self {
        self.rows = rows;
        self
    }

    /// Set the row height.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Set the header height.
    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    /// Set selection mode.
    pub fn selection_mode(mut self, mode: TableSelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Enable zebra stripes.
    pub fn zebra_stripes(mut self, enable: bool) -> Self {
        self.zebra_stripes = enable;
        self
    }

    /// Show row numbers.
    pub fn show_row_numbers(mut self, show: bool) -> Self {
        self.show_row_numbers = show;
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

    /// Set activate handler (double-click).
    pub fn on_activate<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_activate = Some(Box::new(handler));
        self
    }

    /// Set sort handler.
    pub fn on_sort<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, SortDirection) + Send + Sync + 'static,
    {
        self.on_sort = Some(Box::new(handler));
        self
    }

    /// Set context menu handler.
    pub fn on_context_menu<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, Point) + Send + Sync + 'static,
    {
        self.on_context_menu = Some(Box::new(handler));
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Select a row by ID.
    pub fn select(&mut self, id: &str) {
        if self.selection_mode == TableSelectionMode::None {
            return;
        }

        if self.selection_mode == TableSelectionMode::Single {
            for row in &mut self.rows {
                row.selected = false;
            }
        }

        if let Some(row) = self.rows.iter_mut().find(|r| r.id == id) {
            if !row.disabled {
                row.selected = true;
                if let Some(handler) = &self.on_select {
                    handler(id);
                }
            }
        }
    }

    /// Get selected row IDs.
    pub fn selected(&self) -> Vec<&str> {
        self.rows.iter()
            .filter(|r| r.selected)
            .map(|r| r.id.as_str())
            .collect()
    }

    /// Compute column widths based on available space.
    fn compute_widths(&mut self, available_width: f32) {
        let row_num_width = if self.show_row_numbers { 40.0 } else { 0.0 };
        let usable_width = available_width - row_num_width;

        let mut fixed_total: f32 = 0.0;
        let mut flex_total: f32 = 0.0;

        for col in &self.columns {
            match col.width {
                ColumnWidth::Fixed(w) => fixed_total += w,
                ColumnWidth::Flex(f) => flex_total += f,
                ColumnWidth::MinMax { min, .. } => fixed_total += min,
            }
        }

        let flex_space = (usable_width - fixed_total).max(0.0);

        self.computed_widths = self.columns.iter().map(|col| {
            match col.width {
                ColumnWidth::Fixed(w) => w,
                ColumnWidth::Flex(f) => {
                    if flex_total > 0.0 {
                        (f / flex_total) * flex_space
                    } else {
                        0.0
                    }
                }
                ColumnWidth::MinMax { min, max } => {
                    let flex = flex_space / self.columns.len() as f32;
                    flex.clamp(min, max)
                }
            }
        }).collect();
    }

    fn row_at_point(&self, point: Point) -> Option<usize> {
        let bounds = self.base.bounds;
        if !bounds.contains(point) {
            return None;
        }

        let relative_y = point.y - bounds.y() - self.header_height + self.scroll_offset;
        if relative_y < 0.0 {
            return None;
        }

        let index = (relative_y / self.row_height) as usize;
        if index < self.rows.len() {
            Some(index)
        } else {
            None
        }
    }

    fn header_at_point(&self, point: Point) -> Option<usize> {
        let bounds = self.base.bounds;
        if point.y < bounds.y() || point.y > bounds.y() + self.header_height {
            return None;
        }

        let row_num_width = if self.show_row_numbers { 40.0 } else { 0.0 };
        let relative_x = point.x - bounds.x() - row_num_width;
        if relative_x < 0.0 {
            return None;
        }

        let mut x = 0.0;
        for (i, width) in self.computed_widths.iter().enumerate() {
            if relative_x >= x && relative_x < x + width {
                return Some(i);
            }
            x += width;
        }
        None
    }
}

impl Default for DataTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DataTable {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "data-table"
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
        let height = self.header_height + (self.rows.len() as f32 * self.row_height).min(400.0);
        Size::new(400.0, height)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let size = constraints.constrain(self.intrinsic_size(ctx));
        self.base.bounds.size = size;
        self.compute_widths(size.width);
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;
        let row_num_width = if self.show_row_numbers { 40.0 } else { 0.0 };

        // Background
        painter.fill_rect(rect, theme.colors.background);

        // Header
        let header_rect = Rect::new(rect.x(), rect.y(), rect.width(), self.header_height);
        painter.fill_rect(header_rect, theme.colors.surface);

        // Draw header cells
        let mut x = rect.x() + row_num_width;
        for (i, col) in self.columns.iter().enumerate() {
            let width = self.computed_widths.get(i).copied().unwrap_or(100.0);
            let header_cell = Rect::new(x, rect.y(), width, self.header_height);

            // Hover highlight
            if self.hovered_header == Some(i) && col.sortable {
                painter.fill_rect(header_cell, theme.colors.accent.with_alpha(0.1));
            }

            // Header text
            let text_x = match col.align {
                ColumnAlign::Left => x + 12.0,
                ColumnAlign::Center => x + width / 2.0 - 30.0,
                ColumnAlign::Right => x + width - 60.0,
            };
            painter.draw_text(
                &col.header,
                Point::new(text_x, rect.y() + self.header_height / 2.0 + 4.0),
                theme.colors.foreground,
                12.0,
            );

            // Sort indicator
            if self.sort_column.as_ref() == Some(&col.id) {
                let sort_x = x + width - 20.0;
                painter.draw_text(
                    self.sort_direction.icon(),
                    Point::new(sort_x, rect.y() + self.header_height / 2.0 + 4.0),
                    theme.colors.accent,
                    10.0,
                );
            }

            x += width;
        }

        // Header border
        let border_y = rect.y() + self.header_height - 1.0;
        painter.fill_rect(
            Rect::new(rect.x(), border_y, rect.width(), 1.0),
            theme.colors.border,
        );

        // Draw rows
        let content_rect = Rect::new(
            rect.x(),
            rect.y() + self.header_height,
            rect.width(),
            rect.height() - self.header_height,
        );

        let first_visible = (self.scroll_offset / self.row_height) as usize;
        let visible_count = (content_rect.height() / self.row_height).ceil() as usize + 1;
        let last_visible = (first_visible + visible_count).min(self.rows.len());

        for i in first_visible..last_visible {
            let row = &self.rows[i];
            let row_y = content_rect.y() + (i as f32) * self.row_height - self.scroll_offset;
            let row_rect = Rect::new(rect.x(), row_y, rect.width(), self.row_height);

            if row_y + self.row_height < content_rect.y() || row_y > content_rect.y() + content_rect.height() {
                continue;
            }

            // Row background
            let bg_color = if row.selected {
                theme.colors.accent.with_alpha(0.2)
            } else if self.hovered_row == Some(i) && !row.disabled {
                theme.colors.surface_hover
            } else if self.zebra_stripes && i % 2 == 1 {
                theme.colors.surface.with_alpha(0.5)
            } else {
                theme.colors.background
            };
            painter.fill_rect(row_rect, bg_color);

            // Row number
            if self.show_row_numbers {
                let num_text = format!("{}", i + 1);
                painter.draw_text(
                    &num_text,
                    Point::new(rect.x() + 12.0, row_y + self.row_height / 2.0 + 4.0),
                    theme.colors.muted_foreground,
                    11.0,
                );
            }

            // Row icon
            let mut cell_x = rect.x() + row_num_width;
            if let Some(ref icon) = row.icon {
                painter.draw_text(
                    icon,
                    Point::new(cell_x + 8.0, row_y + self.row_height / 2.0 + 4.0),
                    theme.colors.foreground,
                    16.0,
                );
            }

            // Draw cells
            for (j, col) in self.columns.iter().enumerate() {
                let width = self.computed_widths.get(j).copied().unwrap_or(100.0);
                let cell_rect = Rect::new(cell_x, row_y, width, self.row_height);

                if let Some(value) = row.cells.get(&col.id) {
                    let text_color = if row.disabled {
                        theme.colors.muted_foreground
                    } else {
                        theme.colors.foreground
                    };

                    let content_x = match col.align {
                        ColumnAlign::Left => cell_x + 12.0,
                        ColumnAlign::Center => cell_x + width / 2.0,
                        ColumnAlign::Right => cell_x + width - 12.0,
                    };

                    match value {
                        CellValue::Text(text) => {
                            let truncated = if text.chars().count() > 40 {
                                let mut s: String = text.chars().take(37).collect();
                                s.push_str("...");
                                s
                            } else {
                                text.clone()
                            };
                            painter.draw_text(
                                &truncated,
                                Point::new(content_x, row_y + self.row_height / 2.0 + 4.0),
                                text_color,
                                13.0,
                            );
                        }
                        CellValue::Number(n) => {
                            painter.draw_text(
                                &format!("{:.2}", n),
                                Point::new(content_x, row_y + self.row_height / 2.0 + 4.0),
                                text_color,
                                13.0,
                            );
                        }
                        CellValue::Progress(p) => {
                            let bar_width = width - 24.0;
                            let bar_rect = Rect::new(
                                cell_x + 12.0,
                                row_y + self.row_height / 2.0 - 3.0,
                                bar_width,
                                6.0,
                            );
                            painter.fill_rounded_rect(bar_rect, theme.colors.muted, BorderRadius::all(3.0));
                            painter.fill_rounded_rect(
                                Rect::new(bar_rect.x(), bar_rect.y(), bar_rect.width() * p, bar_rect.height()),
                                theme.colors.primary,
                                BorderRadius::all(3.0),
                            );
                        }
                        CellValue::Badge { text, variant } => {
                            let badge_color = match variant {
                                BadgeVariant::Default => theme.colors.muted,
                                BadgeVariant::Primary => theme.colors.primary,
                                BadgeVariant::Success => theme.colors.success,
                                BadgeVariant::Warning => theme.colors.warning,
                                BadgeVariant::Danger => theme.colors.destructive,
                                BadgeVariant::Info => theme.colors.accent,
                            };
                            let badge_rect = Rect::new(
                                content_x,
                                row_y + self.row_height / 2.0 - 10.0,
                                text.len() as f32 * 7.0 + 16.0,
                                20.0,
                            );
                            painter.fill_rounded_rect(badge_rect, badge_color.with_alpha(0.2), BorderRadius::all(4.0));
                            painter.draw_text(
                                text,
                                Point::new(badge_rect.x() + 8.0, badge_rect.y() + 14.0),
                                badge_color,
                                11.0,
                            );
                        }
                        CellValue::Icon(icon) => {
                            painter.draw_text(
                                icon,
                                Point::new(content_x, row_y + self.row_height / 2.0 + 4.0),
                                text_color,
                                16.0,
                            );
                        }
                        CellValue::Custom(renderer) => {
                            renderer.render(painter, cell_rect, ctx);
                        }
                    }
                }

                cell_x += width;
            }

            // Row border
            if i < self.rows.len() - 1 {
                painter.fill_rect(
                    Rect::new(rect.x(), row_y + self.row_height - 1.0, rect.width(), 1.0),
                    theme.colors.border.with_alpha(0.3),
                );
            }
        }

        // Border
        painter.stroke_rect(rect, theme.colors.border, 1.0);
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Move => {
                        let new_row = self.row_at_point(mouse.position);
                        let new_header = self.header_at_point(mouse.position);

                        if new_row != self.hovered_row || new_header != self.hovered_header {
                            self.hovered_row = new_row;
                            self.hovered_header = new_header;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Leave => {
                        if self.hovered_row.is_some() || self.hovered_header.is_some() {
                            self.hovered_row = None;
                            self.hovered_header = None;
                            ctx.request_redraw();
                        }
                    }
                    MouseEventKind::Down => {
                        if mouse.button == Some(MouseButton::Left) {
                            // Header click (sort)
                            if let Some(col_idx) = self.header_at_point(mouse.position) {
                                if self.columns[col_idx].sortable {
                                    let col_id = self.columns[col_idx].id.clone();
                                    if self.sort_column.as_ref() == Some(&col_id) {
                                        self.sort_direction = self.sort_direction.toggle();
                                    } else {
                                        self.sort_column = Some(col_id.clone());
                                        self.sort_direction = SortDirection::Ascending;
                                    }
                                    if let Some(handler) = &self.on_sort {
                                        handler(&col_id, self.sort_direction);
                                    }
                                    ctx.request_redraw();
                                    return EventResult::Handled;
                                }
                            }

                            // Row click (select)
                            if let Some(row_idx) = self.row_at_point(mouse.position) {
                                let id = self.rows[row_idx].id.clone();
                                self.select(&id);
                                ctx.request_focus(self.base.id);
                                ctx.request_redraw();
                                return EventResult::Handled;
                            }
                        } else if mouse.button == Some(MouseButton::Right) {
                            // Context menu
                            if let Some(row_idx) = self.row_at_point(mouse.position) {
                                let id = self.rows[row_idx].id.clone();
                                self.select(&id);
                                if let Some(handler) = &self.on_context_menu {
                                    handler(&id, mouse.position);
                                }
                                ctx.request_redraw();
                                return EventResult::Handled;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Down && self.base.state.focused => {
                match key.key {
                    Key::Enter => {
                        if let Some(row) = self.rows.iter().find(|r| r.selected) {
                            let id = row.id.clone();
                            if let Some(handler) = &self.on_activate {
                                handler(&id);
                            }
                            return EventResult::Handled;
                        }
                    }
                    Key::Up => {
                        if let Some(idx) = self.rows.iter().position(|r| r.selected) {
                            if idx > 0 {
                                let id = self.rows[idx - 1].id.clone();
                                self.select(&id);
                                ctx.request_redraw();
                            }
                        }
                        return EventResult::Handled;
                    }
                    Key::Down => {
                        if let Some(idx) = self.rows.iter().position(|r| r.selected) {
                            if idx + 1 < self.rows.len() {
                                let id = self.rows[idx + 1].id.clone();
                                self.select(&id);
                                ctx.request_redraw();
                            }
                        } else if !self.rows.is_empty() {
                            let id = self.rows[0].id.clone();
                            self.select(&id);
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
