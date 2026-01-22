//! Button widget.

use super::{Widget, WidgetBase, WidgetId, LayoutContext, PaintContext, EventContext};
use crate::css::{ClassList, WidgetState};
use crate::event::{Event, EventResult, MouseEventKind, MouseButton};
use crate::geometry::{BorderRadius, Color, Point, Rect, Size};
use crate::layout::{Constraints, LayoutResult};
use crate::render::Painter;

/// A clickable button widget.
pub struct Button {
    base: WidgetBase,
    label: String,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    variant: ButtonVariant,
}

/// Button visual variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            base: WidgetBase::new().with_class("button"),
            label: label.into(),
            on_click: None,
            variant: ButtonVariant::Primary,
        }
    }

    /// Set the button label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
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

    /// Set the button variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        // Update class for variant
        let class = match variant {
            ButtonVariant::Primary => "btn-primary",
            ButtonVariant::Secondary => "btn-secondary",
            ButtonVariant::Outline => "btn-outline",
            ButtonVariant::Ghost => "btn-ghost",
            ButtonVariant::Destructive => "btn-destructive",
        };
        self.base.classes.add(class);
        self
    }

    /// Add a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.base.classes.add(class);
        self
    }

    /// Set the element ID.
    pub fn id(mut self, id: &str) -> Self {
        self.base.element_id = Some(id.to_string());
        self
    }

    /// Get the background color based on state and variant.
    fn background_color(&self, theme: &crate::theme::ThemeData) -> Color {
        let base = match self.variant {
            ButtonVariant::Primary => theme.colors.primary,
            ButtonVariant::Secondary => theme.colors.secondary,
            ButtonVariant::Outline => Color::TRANSPARENT,
            ButtonVariant::Ghost => Color::TRANSPARENT,
            ButtonVariant::Destructive => theme.colors.destructive,
        };

        // Ghost and outline get hover backgrounds
        let hover_bg = match self.variant {
            ButtonVariant::Ghost | ButtonVariant::Outline => theme.colors.muted.with_alpha(0.8),
            _ => base.darken(8.0),
        };

        if self.base.state.disabled {
            base.with_alpha(0.5)
        } else if self.base.state.pressed {
            match self.variant {
                ButtonVariant::Ghost | ButtonVariant::Outline => theme.colors.muted,
                _ => base.darken(12.0),
            }
        } else if self.base.state.hovered {
            hover_bg
        } else {
            base
        }
    }

    /// Get the text color based on variant.
    fn text_color(&self, theme: &crate::theme::ThemeData) -> Color {
        match self.variant {
            ButtonVariant::Primary => theme.colors.primary_foreground,
            ButtonVariant::Secondary => theme.colors.secondary_foreground,
            ButtonVariant::Outline => theme.colors.foreground,
            ButtonVariant::Ghost => theme.colors.foreground,
            ButtonVariant::Destructive => theme.colors.destructive_foreground,
        }
    }
}

impl Widget for Button {
    fn id(&self) -> WidgetId {
        self.base.id
    }

    fn type_name(&self) -> &'static str {
        "button"
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
        // Tailwind-style button sizing (px-4 py-2.5 with medium font)
        let font_size = 14.0;
        let char_width = font_size * 0.55; // Slightly tighter character spacing
        let text_width = self.label.len() as f32 * char_width;
        let padding_h = 16.0 * 2.0; // px-4 (16px each side)
        let padding_v = 10.0 * 2.0; // py-2.5 (10px each side)
        Size::new(text_width + padding_h, font_size + padding_v)
    }

    fn layout(&mut self, constraints: Constraints, ctx: &LayoutContext) -> LayoutResult {
        let intrinsic = self.intrinsic_size(ctx);
        let size = constraints.constrain(intrinsic);
        self.base.bounds.size = size;
        LayoutResult::new(size)
    }

    fn paint(&self, painter: &mut Painter, rect: Rect, ctx: &PaintContext) {
        let theme = ctx.style_ctx.theme;

        // Tailwind-style rounded corners (rounded-lg = 8px)
        let radius = BorderRadius::all(8.0);

        // Shadow for depth (subtle shadow for primary/destructive, none for ghost/outline)
        if !self.base.state.pressed && matches!(self.variant, ButtonVariant::Primary | ButtonVariant::Secondary | ButtonVariant::Destructive) {
            // Draw shadow layer (offset down and slightly blurred effect via multiple rects)
            let shadow_color = Color::rgba(0.0, 0.0, 0.0, 0.1);
            let shadow_rect = Rect::new(rect.x(), rect.y() + 1.0, rect.width(), rect.height());
            painter.fill_rounded_rect(shadow_rect, shadow_color, radius);
        }

        // Background
        let bg_color = self.background_color(theme);
        painter.fill_rounded_rect(rect, bg_color, radius);

        // Border for outline variant (Tailwind-style border)
        if self.variant == ButtonVariant::Outline {
            let border_color = if self.base.state.hovered {
                theme.colors.foreground.with_alpha(0.2)
            } else {
                theme.colors.border
            };
            painter.stroke_rounded_rect(rect, border_color, 1.0, radius);
        }

        // Text - centered with proper font sizing
        let text_color = self.text_color(theme);
        let font_size = 14.0;
        let char_width = font_size * 0.55;
        let text_width = self.label.len() as f32 * char_width;
        let text_x = rect.x() + (rect.width() - text_width) / 2.0;
        let text_y = rect.y() + (rect.height() + font_size * 0.7) / 2.0;
        painter.draw_text(&self.label, Point::new(text_x, text_y), text_color, font_size);

        // Focus ring (Tailwind-style ring with offset)
        if self.base.state.focused && ctx.focus_visible {
            let ring_offset = 2.0;
            let ring_rect = Rect::new(
                rect.x() - ring_offset,
                rect.y() - ring_offset,
                rect.width() + ring_offset * 2.0,
                rect.height() + ring_offset * 2.0,
            );
            let ring_radius = BorderRadius::all(8.0 + ring_offset);
            painter.stroke_rounded_rect(ring_rect, theme.colors.ring.with_alpha(0.5), 2.0, ring_radius);
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Event::Mouse(mouse) = event {
            let in_bounds = self.bounds().contains(mouse.position);

            match mouse.kind {
                MouseEventKind::Enter | MouseEventKind::Move => {
                    if in_bounds && !self.base.state.hovered {
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
                MouseEventKind::Down => {
                    if in_bounds && mouse.button == Some(MouseButton::Left) {
                        self.base.state.pressed = true;
                        ctx.request_focus(self.base.id);
                        ctx.request_redraw();
                        return EventResult::Handled;
                    }
                }
                MouseEventKind::Up => {
                    if self.base.state.pressed && mouse.button == Some(MouseButton::Left) {
                        self.base.state.pressed = false;
                        if in_bounds {
                            // Trigger click
                            if let Some(handler) = &self.on_click {
                                handler();
                            }
                        }
                        ctx.request_redraw();
                        return EventResult::Handled;
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
