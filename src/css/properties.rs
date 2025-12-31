//! CSS property definitions and computed styles.

use crate::css::{CssValue, StyleContext};
use crate::geometry::{BorderRadius, Color, EdgeInsets};

/// A style property name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StyleProperty {
    // Layout
    Display,
    Position,
    Top,
    Right,
    Bottom,
    Left,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,

    // Flexbox
    FlexDirection,
    FlexWrap,
    JustifyContent,
    AlignItems,
    AlignSelf,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    Gap,
    RowGap,
    ColumnGap,

    // Spacing
    Margin,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    Padding,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,

    // Background
    BackgroundColor,
    BackgroundImage,
    BackgroundSize,
    BackgroundPosition,
    BackgroundRepeat,

    // Border
    BorderWidth,
    BorderTopWidth,
    BorderRightWidth,
    BorderBottomWidth,
    BorderLeftWidth,
    BorderColor,
    BorderTopColor,
    BorderRightColor,
    BorderBottomColor,
    BorderLeftColor,
    BorderStyle,
    BorderRadius,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomRightRadius,
    BorderBottomLeftRadius,

    // Typography
    Color,
    FontFamily,
    FontSize,
    FontWeight,
    FontStyle,
    LineHeight,
    TextAlign,
    TextDecoration,
    TextTransform,
    WhiteSpace,
    LetterSpacing,
    WordSpacing,

    // Effects
    Opacity,
    BoxShadow,
    Cursor,
    Outline,
    OutlineWidth,
    OutlineColor,
    OutlineStyle,
    OutlineOffset,

    // Transitions
    Transition,
    TransitionProperty,
    TransitionDuration,
    TransitionTimingFunction,
    TransitionDelay,

    // Transform
    Transform,
    TransformOrigin,

    // Overflow
    Overflow,
    OverflowX,
    OverflowY,

    // Other
    ZIndex,
    Visibility,
    PointerEvents,

    // Custom property
    Custom(String),
}

impl StyleProperty {
    pub fn from_name(name: &str) -> Self {
        match name {
            "display" => StyleProperty::Display,
            "position" => StyleProperty::Position,
            "top" => StyleProperty::Top,
            "right" => StyleProperty::Right,
            "bottom" => StyleProperty::Bottom,
            "left" => StyleProperty::Left,
            "width" => StyleProperty::Width,
            "height" => StyleProperty::Height,
            "min-width" => StyleProperty::MinWidth,
            "min-height" => StyleProperty::MinHeight,
            "max-width" => StyleProperty::MaxWidth,
            "max-height" => StyleProperty::MaxHeight,

            "flex-direction" => StyleProperty::FlexDirection,
            "flex-wrap" => StyleProperty::FlexWrap,
            "justify-content" => StyleProperty::JustifyContent,
            "align-items" => StyleProperty::AlignItems,
            "align-self" => StyleProperty::AlignSelf,
            "flex-grow" => StyleProperty::FlexGrow,
            "flex-shrink" => StyleProperty::FlexShrink,
            "flex-basis" => StyleProperty::FlexBasis,
            "gap" => StyleProperty::Gap,
            "row-gap" => StyleProperty::RowGap,
            "column-gap" => StyleProperty::ColumnGap,

            "margin" => StyleProperty::Margin,
            "margin-top" => StyleProperty::MarginTop,
            "margin-right" => StyleProperty::MarginRight,
            "margin-bottom" => StyleProperty::MarginBottom,
            "margin-left" => StyleProperty::MarginLeft,
            "padding" => StyleProperty::Padding,
            "padding-top" => StyleProperty::PaddingTop,
            "padding-right" => StyleProperty::PaddingRight,
            "padding-bottom" => StyleProperty::PaddingBottom,
            "padding-left" => StyleProperty::PaddingLeft,

            "background-color" | "background" => StyleProperty::BackgroundColor,
            "background-image" => StyleProperty::BackgroundImage,
            "background-size" => StyleProperty::BackgroundSize,
            "background-position" => StyleProperty::BackgroundPosition,
            "background-repeat" => StyleProperty::BackgroundRepeat,

            "border-width" => StyleProperty::BorderWidth,
            "border-top-width" => StyleProperty::BorderTopWidth,
            "border-right-width" => StyleProperty::BorderRightWidth,
            "border-bottom-width" => StyleProperty::BorderBottomWidth,
            "border-left-width" => StyleProperty::BorderLeftWidth,
            "border-color" => StyleProperty::BorderColor,
            "border-top-color" => StyleProperty::BorderTopColor,
            "border-right-color" => StyleProperty::BorderRightColor,
            "border-bottom-color" => StyleProperty::BorderBottomColor,
            "border-left-color" => StyleProperty::BorderLeftColor,
            "border-style" => StyleProperty::BorderStyle,
            "border-radius" => StyleProperty::BorderRadius,
            "border-top-left-radius" => StyleProperty::BorderTopLeftRadius,
            "border-top-right-radius" => StyleProperty::BorderTopRightRadius,
            "border-bottom-right-radius" => StyleProperty::BorderBottomRightRadius,
            "border-bottom-left-radius" => StyleProperty::BorderBottomLeftRadius,

            "color" => StyleProperty::Color,
            "font-family" => StyleProperty::FontFamily,
            "font-size" => StyleProperty::FontSize,
            "font-weight" => StyleProperty::FontWeight,
            "font-style" => StyleProperty::FontStyle,
            "line-height" => StyleProperty::LineHeight,
            "text-align" => StyleProperty::TextAlign,
            "text-decoration" => StyleProperty::TextDecoration,
            "text-transform" => StyleProperty::TextTransform,
            "white-space" => StyleProperty::WhiteSpace,
            "letter-spacing" => StyleProperty::LetterSpacing,
            "word-spacing" => StyleProperty::WordSpacing,

            "opacity" => StyleProperty::Opacity,
            "box-shadow" => StyleProperty::BoxShadow,
            "cursor" => StyleProperty::Cursor,
            "outline" => StyleProperty::Outline,
            "outline-width" => StyleProperty::OutlineWidth,
            "outline-color" => StyleProperty::OutlineColor,
            "outline-style" => StyleProperty::OutlineStyle,
            "outline-offset" => StyleProperty::OutlineOffset,

            "transition" => StyleProperty::Transition,
            "transition-property" => StyleProperty::TransitionProperty,
            "transition-duration" => StyleProperty::TransitionDuration,
            "transition-timing-function" => StyleProperty::TransitionTimingFunction,
            "transition-delay" => StyleProperty::TransitionDelay,

            "transform" => StyleProperty::Transform,
            "transform-origin" => StyleProperty::TransformOrigin,

            "overflow" => StyleProperty::Overflow,
            "overflow-x" => StyleProperty::OverflowX,
            "overflow-y" => StyleProperty::OverflowY,

            "z-index" => StyleProperty::ZIndex,
            "visibility" => StyleProperty::Visibility,
            "pointer-events" => StyleProperty::PointerEvents,

            name if name.starts_with("--") => StyleProperty::Custom(name.to_string()),
            _ => StyleProperty::Custom(name.to_string()),
        }
    }

    /// Check if this property is inherited by default.
    pub fn is_inherited(&self) -> bool {
        matches!(
            self,
            StyleProperty::Color
                | StyleProperty::FontFamily
                | StyleProperty::FontSize
                | StyleProperty::FontWeight
                | StyleProperty::FontStyle
                | StyleProperty::LineHeight
                | StyleProperty::TextAlign
                | StyleProperty::WhiteSpace
                | StyleProperty::LetterSpacing
                | StyleProperty::WordSpacing
                | StyleProperty::Cursor
                | StyleProperty::Visibility
        )
    }
}

/// Computed style values for a widget.
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    // Layout
    pub display: Display,
    pub position: Position,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,

    // Flexbox
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_self: AlignSelf,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Option<f32>,
    pub gap: f32,
    pub row_gap: f32,
    pub column_gap: f32,

    // Spacing
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,

    // Background
    pub background_color: Color,

    // Border
    pub border_width: EdgeInsets,
    pub border_color: Color,
    pub border_radius: BorderRadius,

    // Typography
    pub color: Color,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub line_height: f32,
    pub text_align: TextAlign,

    // Effects
    pub opacity: f32,
    pub cursor: Cursor,

    // Outline
    pub outline_width: f32,
    pub outline_color: Color,
    pub outline_offset: f32,

    // Overflow
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,

    // Other
    pub z_index: i32,
    pub visibility: Visibility,
    pub pointer_events: PointerEvents,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Flex,
            position: Position::Relative,
            top: None,
            right: None,
            bottom: None,
            left: None,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,

            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_self: AlignSelf::Auto,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
            gap: 0.0,
            row_gap: 0.0,
            column_gap: 0.0,

            margin: EdgeInsets::ZERO,
            padding: EdgeInsets::ZERO,

            background_color: Color::TRANSPARENT,

            border_width: EdgeInsets::ZERO,
            border_color: Color::TRANSPARENT,
            border_radius: BorderRadius::ZERO,

            color: Color::BLACK,
            font_family: "Inter".to_string(),
            font_size: 16.0,
            font_weight: 400,
            line_height: 1.5,
            text_align: TextAlign::Left,

            opacity: 1.0,
            cursor: Cursor::Default,

            outline_width: 0.0,
            outline_color: Color::TRANSPARENT,
            outline_offset: 0.0,

            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,

            z_index: 0,
            visibility: Visibility::Visible,
            pointer_events: PointerEvents::Auto,
        }
    }
}

impl ComputedStyle {
    /// Apply a property value.
    pub fn apply(&mut self, property: &StyleProperty, value: &CssValue, ctx: &StyleContext) {
        match property {
            StyleProperty::Display => {
                if let CssValue::Keyword(k) = value {
                    self.display = Display::from_keyword(k);
                }
            }
            StyleProperty::Position => {
                if let CssValue::Keyword(k) = value {
                    self.position = Position::from_keyword(k);
                }
            }
            StyleProperty::Width => {
                if let Some(len) = value.as_length() {
                    self.width = Some(ctx.to_pixels(&len));
                } else if value.is_auto() {
                    self.width = None;
                }
            }
            StyleProperty::Height => {
                if let Some(len) = value.as_length() {
                    self.height = Some(ctx.to_pixels(&len));
                } else if value.is_auto() {
                    self.height = None;
                }
            }
            StyleProperty::BackgroundColor => {
                if let Some(color) = value.as_color() {
                    self.background_color = color;
                }
            }
            StyleProperty::Color => {
                if let Some(color) = value.as_color() {
                    self.color = color;
                }
            }
            StyleProperty::FontSize => {
                if let Some(len) = value.as_length() {
                    self.font_size = ctx.to_pixels(&len);
                }
            }
            StyleProperty::FontWeight => {
                if let CssValue::Number(n) = value {
                    self.font_weight = *n as u16;
                } else if let CssValue::Keyword(k) = value {
                    self.font_weight = match k.as_str() {
                        "normal" => 400,
                        "bold" => 700,
                        "lighter" => 300,
                        "bolder" => 700,
                        _ => 400,
                    };
                }
            }
            StyleProperty::Padding => {
                if let CssValue::List(values) = value {
                    self.padding = parse_edge_insets(values, ctx);
                } else if let Some(len) = value.as_length() {
                    let px = ctx.to_pixels(&len);
                    self.padding = EdgeInsets::all(px);
                }
            }
            StyleProperty::PaddingTop => {
                if let Some(len) = value.as_length() {
                    self.padding.top = ctx.to_pixels(&len);
                }
            }
            StyleProperty::PaddingRight => {
                if let Some(len) = value.as_length() {
                    self.padding.right = ctx.to_pixels(&len);
                }
            }
            StyleProperty::PaddingBottom => {
                if let Some(len) = value.as_length() {
                    self.padding.bottom = ctx.to_pixels(&len);
                }
            }
            StyleProperty::PaddingLeft => {
                if let Some(len) = value.as_length() {
                    self.padding.left = ctx.to_pixels(&len);
                }
            }
            StyleProperty::Margin => {
                if let CssValue::List(values) = value {
                    self.margin = parse_edge_insets(values, ctx);
                } else if let Some(len) = value.as_length() {
                    let px = ctx.to_pixels(&len);
                    self.margin = EdgeInsets::all(px);
                }
            }
            StyleProperty::BorderRadius => {
                if let Some(len) = value.as_length() {
                    let px = ctx.to_pixels(&len);
                    self.border_radius = BorderRadius::all(px);
                }
            }
            StyleProperty::BorderColor => {
                if let Some(color) = value.as_color() {
                    self.border_color = color;
                }
            }
            StyleProperty::BorderWidth => {
                if let Some(len) = value.as_length() {
                    let px = ctx.to_pixels(&len);
                    self.border_width = EdgeInsets::all(px);
                }
            }
            StyleProperty::Opacity => {
                if let CssValue::Number(n) = value {
                    self.opacity = n.clamp(0.0, 1.0);
                }
            }
            StyleProperty::Gap => {
                if let Some(len) = value.as_length() {
                    self.gap = ctx.to_pixels(&len);
                    self.row_gap = self.gap;
                    self.column_gap = self.gap;
                }
            }
            StyleProperty::FlexDirection => {
                if let CssValue::Keyword(k) = value {
                    self.flex_direction = FlexDirection::from_keyword(k);
                }
            }
            StyleProperty::JustifyContent => {
                if let CssValue::Keyword(k) = value {
                    self.justify_content = JustifyContent::from_keyword(k);
                }
            }
            StyleProperty::AlignItems => {
                if let CssValue::Keyword(k) = value {
                    self.align_items = AlignItems::from_keyword(k);
                }
            }
            StyleProperty::Cursor => {
                if let CssValue::Keyword(k) = value {
                    self.cursor = Cursor::from_keyword(k);
                }
            }
            StyleProperty::TextAlign => {
                if let CssValue::Keyword(k) = value {
                    self.text_align = TextAlign::from_keyword(k);
                }
            }
            StyleProperty::OutlineWidth => {
                if let Some(len) = value.as_length() {
                    self.outline_width = ctx.to_pixels(&len);
                }
            }
            StyleProperty::OutlineColor => {
                if let Some(color) = value.as_color() {
                    self.outline_color = color;
                }
            }
            StyleProperty::OutlineOffset => {
                if let Some(len) = value.as_length() {
                    self.outline_offset = ctx.to_pixels(&len);
                }
            }
            _ => {
                // Other properties not yet implemented
            }
        }
    }
}

/// Parse edge insets from a list of CSS values.
fn parse_edge_insets(values: &[CssValue], ctx: &StyleContext) -> EdgeInsets {
    let pixels: Vec<f32> = values
        .iter()
        .filter_map(|v| v.as_length().map(|l| ctx.to_pixels(&l)))
        .collect();

    match pixels.len() {
        1 => EdgeInsets::all(pixels[0]),
        2 => EdgeInsets::symmetric(pixels[0], pixels[1]),
        3 => EdgeInsets::new(pixels[0], pixels[1], pixels[2], pixels[1]),
        4 => EdgeInsets::new(pixels[0], pixels[1], pixels[2], pixels[3]),
        _ => EdgeInsets::ZERO,
    }
}

// Enums for style values

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    None,
    Block,
    #[default]
    Flex,
    Grid,
    Inline,
    InlineBlock,
    InlineFlex,
}

impl Display {
    pub fn from_keyword(s: &str) -> Self {
        match s {
            "none" => Display::None,
            "block" => Display::Block,
            "flex" => Display::Flex,
            "grid" => Display::Grid,
            "inline" => Display::Inline,
            "inline-block" => Display::InlineBlock,
            "inline-flex" => Display::InlineFlex,
            _ => Display::Flex,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    Static,
    #[default]
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl Position {
    pub fn from_keyword(s: &str) -> Self {
        match s {
            "static" => Position::Static,
            "relative" => Position::Relative,
            "absolute" => Position::Absolute,
            "fixed" => Position::Fixed,
            "sticky" => Position::Sticky,
            _ => Position::Relative,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirection {
    pub fn from_keyword(s: &str) -> Self {
        match s {
            "row" => FlexDirection::Row,
            "row-reverse" => FlexDirection::RowReverse,
            "column" => FlexDirection::Column,
            "column-reverse" => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl JustifyContent {
    pub fn from_keyword(s: &str) -> Self {
        match s {
            "flex-start" | "start" => JustifyContent::FlexStart,
            "flex-end" | "end" => JustifyContent::FlexEnd,
            "center" => JustifyContent::Center,
            "space-between" => JustifyContent::SpaceBetween,
            "space-around" => JustifyContent::SpaceAround,
            "space-evenly" => JustifyContent::SpaceEvenly,
            _ => JustifyContent::FlexStart,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    #[default]
    Stretch,
    Baseline,
}

impl AlignItems {
    pub fn from_keyword(s: &str) -> Self {
        match s {
            "flex-start" | "start" => AlignItems::FlexStart,
            "flex-end" | "end" => AlignItems::FlexEnd,
            "center" => AlignItems::Center,
            "stretch" => AlignItems::Stretch,
            "baseline" => AlignItems::Baseline,
            _ => AlignItems::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignSelf {
    #[default]
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
    Justify,
}

impl TextAlign {
    pub fn from_keyword(s: &str) -> Self {
        match s {
            "left" => TextAlign::Left,
            "right" => TextAlign::Right,
            "center" => TextAlign::Center,
            "justify" => TextAlign::Justify,
            _ => TextAlign::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerEvents {
    #[default]
    Auto,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Default,
    Pointer,
    Text,
    Wait,
    NotAllowed,
    Grab,
    Grabbing,
    Move,
    ResizeNS,
    ResizeEW,
    ResizeNESW,
    ResizeNWSE,
    Crosshair,
    Help,
    None,
}

impl Cursor {
    pub fn from_keyword(s: &str) -> Self {
        match s {
            "default" => Cursor::Default,
            "pointer" => Cursor::Pointer,
            "text" => Cursor::Text,
            "wait" => Cursor::Wait,
            "not-allowed" => Cursor::NotAllowed,
            "grab" => Cursor::Grab,
            "grabbing" => Cursor::Grabbing,
            "move" => Cursor::Move,
            "ns-resize" => Cursor::ResizeNS,
            "ew-resize" => Cursor::ResizeEW,
            "nesw-resize" => Cursor::ResizeNESW,
            "nwse-resize" => Cursor::ResizeNWSE,
            "crosshair" => Cursor::Crosshair,
            "help" => Cursor::Help,
            "none" => Cursor::None,
            _ => Cursor::Default,
        }
    }
}
