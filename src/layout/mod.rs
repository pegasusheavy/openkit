//! Layout engine for OpenKit.
//!
//! Implements flexbox-style layout.

use crate::css::{AlignItems, ComputedStyle, FlexDirection, JustifyContent};
use crate::geometry::{EdgeInsets, Point, Size};

/// Layout constraints for a widget.
#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }

    pub fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }

    pub fn is_tight(&self) -> bool {
        self.min_width == self.max_width && self.min_height == self.max_height
    }

    pub fn has_bounded_width(&self) -> bool {
        self.max_width.is_finite()
    }

    pub fn has_bounded_height(&self) -> bool {
        self.max_height.is_finite()
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Self::unbounded()
    }
}

/// Layout result for a widget.
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutResult {
    pub size: Size,
}

impl LayoutResult {
    pub fn new(size: Size) -> Self {
        Self { size }
    }

    pub fn with_size(width: f32, height: f32) -> Self {
        Self {
            size: Size::new(width, height),
        }
    }
}

/// Alignment options for layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl From<JustifyContent> for Alignment {
    fn from(jc: JustifyContent) -> Self {
        match jc {
            JustifyContent::FlexStart => Alignment::Start,
            JustifyContent::FlexEnd => Alignment::End,
            JustifyContent::Center => Alignment::Center,
            JustifyContent::SpaceBetween => Alignment::SpaceBetween,
            JustifyContent::SpaceAround => Alignment::SpaceAround,
            JustifyContent::SpaceEvenly => Alignment::SpaceEvenly,
        }
    }
}

impl From<AlignItems> for Alignment {
    fn from(ai: AlignItems) -> Self {
        match ai {
            AlignItems::FlexStart => Alignment::Start,
            AlignItems::FlexEnd => Alignment::End,
            AlignItems::Center => Alignment::Center,
            AlignItems::Stretch => Alignment::Stretch,
            AlignItems::Baseline => Alignment::Start, // Simplified
        }
    }
}

/// Padding wrapper for convenience.
#[derive(Debug, Clone, Copy, Default)]
pub struct Padding(pub EdgeInsets);

impl Padding {
    pub fn all(value: f32) -> Self {
        Padding(EdgeInsets::all(value))
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Padding(EdgeInsets::symmetric(vertical, horizontal))
    }

    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Padding(EdgeInsets::new(top, right, bottom, left))
    }

    pub fn horizontal(&self) -> f32 {
        self.0.horizontal()
    }

    pub fn vertical(&self) -> f32 {
        self.0.vertical()
    }
}

impl From<f32> for Padding {
    fn from(value: f32) -> Self {
        Padding::all(value)
    }
}

impl From<EdgeInsets> for Padding {
    fn from(insets: EdgeInsets) -> Self {
        Padding(insets)
    }
}

/// Layout trait for widgets.
pub trait Layout {
    /// Calculate the intrinsic size of this widget.
    fn intrinsic_size(&self, style: &ComputedStyle) -> Size;

    /// Perform layout and return the final size.
    fn layout(&mut self, constraints: Constraints, style: &ComputedStyle) -> LayoutResult;

    /// Get child positions after layout.
    fn child_positions(&self) -> Vec<Point> {
        Vec::new()
    }
}

/// Flexbox layout calculator.
pub struct FlexLayout {
    pub direction: FlexDirection,
    pub justify: Alignment,
    pub align: Alignment,
    pub gap: f32,
}

impl FlexLayout {
    pub fn new(style: &ComputedStyle) -> Self {
        Self {
            direction: style.flex_direction,
            justify: style.justify_content.into(),
            align: style.align_items.into(),
            gap: style.gap,
        }
    }

    pub fn row() -> Self {
        Self {
            direction: FlexDirection::Row,
            justify: Alignment::Start,
            align: Alignment::Stretch,
            gap: 0.0,
        }
    }

    pub fn column() -> Self {
        Self {
            direction: FlexDirection::Column,
            justify: Alignment::Start,
            align: Alignment::Stretch,
            gap: 0.0,
        }
    }

    /// Check if this is a vertical layout.
    pub fn is_vertical(&self) -> bool {
        matches!(
            self.direction,
            FlexDirection::Column | FlexDirection::ColumnReverse
        )
    }

    /// Check if this is a reversed layout.
    pub fn is_reversed(&self) -> bool {
        matches!(
            self.direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        )
    }

    /// Calculate positions for children.
    pub fn calculate_positions(
        &self,
        container_size: Size,
        child_sizes: &[Size],
        padding: EdgeInsets,
    ) -> Vec<Point> {
        if child_sizes.is_empty() {
            return Vec::new();
        }

        let is_vertical = self.is_vertical();
        let is_reversed = self.is_reversed();

        // Calculate main axis (the direction of the flex layout)
        let (main_start, main_size, cross_start, cross_size) = if is_vertical {
            (
                padding.top,
                container_size.height - padding.vertical(),
                padding.left,
                container_size.width - padding.horizontal(),
            )
        } else {
            (
                padding.left,
                container_size.width - padding.horizontal(),
                padding.top,
                container_size.height - padding.vertical(),
            )
        };

        // Calculate total child size on main axis
        let total_child_main: f32 = child_sizes
            .iter()
            .map(|s| if is_vertical { s.height } else { s.width })
            .sum();
        let total_gaps = self.gap * (child_sizes.len() - 1).max(0) as f32;
        let total_content = total_child_main + total_gaps;
        let free_space = (main_size - total_content).max(0.0);

        // Calculate starting position and spacing based on justify
        let (mut main_pos, spacing) = match self.justify {
            Alignment::Start => (main_start, 0.0),
            Alignment::End => (main_start + free_space, 0.0),
            Alignment::Center => (main_start + free_space / 2.0, 0.0),
            Alignment::SpaceBetween => {
                if child_sizes.len() > 1 {
                    (main_start, free_space / (child_sizes.len() - 1) as f32)
                } else {
                    (main_start, 0.0)
                }
            }
            Alignment::SpaceAround => {
                let s = free_space / child_sizes.len() as f32;
                (main_start + s / 2.0, s)
            }
            Alignment::SpaceEvenly => {
                let s = free_space / (child_sizes.len() + 1) as f32;
                (main_start + s, s)
            }
            Alignment::Stretch => (main_start, 0.0),
        };

        // Build positions
        let mut positions = Vec::with_capacity(child_sizes.len());
        let items: Box<dyn Iterator<Item = &Size>> = if is_reversed {
            Box::new(child_sizes.iter().rev())
        } else {
            Box::new(child_sizes.iter())
        };

        for child_size in items {
            let child_main = if is_vertical { child_size.height } else { child_size.width };
            let child_cross = if is_vertical { child_size.width } else { child_size.height };

            // Calculate cross axis position based on align
            let cross_pos = match self.align {
                Alignment::Start => cross_start,
                Alignment::End => cross_start + cross_size - child_cross,
                Alignment::Center => cross_start + (cross_size - child_cross) / 2.0,
                // SpaceBetween, SpaceAround, SpaceEvenly are handled by the main axis calculation
                Alignment::Stretch | Alignment::SpaceBetween | Alignment::SpaceAround | Alignment::SpaceEvenly => cross_start,
            };

            let point = if is_vertical {
                Point::new(cross_pos, main_pos)
            } else {
                Point::new(main_pos, cross_pos)
            };

            positions.push(point);
            main_pos += child_main + self.gap + spacing;
        }

        if is_reversed {
            positions.reverse();
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraints() {
        let c = Constraints::loose(Size::new(100.0, 50.0));
        let size = c.constrain(Size::new(200.0, 100.0));
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);
    }

    #[test]
    fn test_flex_layout_positions() {
        let layout = FlexLayout::row();
        let positions = layout.calculate_positions(
            Size::new(300.0, 100.0),
            &[Size::new(50.0, 100.0), Size::new(50.0, 100.0)],
            EdgeInsets::ZERO,
        );

        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].x, 0.0);
        assert_eq!(positions[1].x, 50.0);
    }

    #[test]
    fn test_flex_center_justify() {
        let mut layout = FlexLayout::row();
        layout.justify = Alignment::Center;

        let positions = layout.calculate_positions(
            Size::new(200.0, 100.0),
            &[Size::new(50.0, 100.0), Size::new(50.0, 100.0)],
            EdgeInsets::ZERO,
        );

        // 200 - 100 = 100 free space, centered means start at 50
        assert_eq!(positions[0].x, 50.0);
        assert_eq!(positions[1].x, 100.0);
    }
}
