//! CSS value types.

use crate::geometry::Color;

/// A CSS value.
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    /// Keyword value (e.g., "auto", "inherit", "none")
    Keyword(String),
    /// Length value
    Length(Length),
    /// Percentage value
    Percentage(f32),
    /// Color value
    Color(Color),
    /// Number value
    Number(f32),
    /// String value
    String(String),
    /// URL value
    Url(String),
    /// CSS variable reference
    Var(String, Option<Box<CssValue>>),
    /// calc() expression
    Calc(CalcExpr),
    /// Multiple values (e.g., for margin shorthand)
    List(Vec<CssValue>),
}

impl CssValue {
    /// Try to get as a length in pixels.
    pub fn as_length(&self) -> Option<Length> {
        match self {
            CssValue::Length(l) => Some(l.clone()),
            CssValue::Number(n) if *n == 0.0 => Some(Length::px(0.0)),
            CssValue::Percentage(p) => Some(Length::percent(*p)),
            _ => None,
        }
    }

    /// Try to get as a color.
    pub fn as_color(&self) -> Option<Color> {
        match self {
            CssValue::Color(c) => Some(*c),
            CssValue::Keyword(k) => color_from_keyword(k),
            _ => None,
        }
    }

    /// Try to get as a number.
    pub fn as_number(&self) -> Option<f32> {
        match self {
            CssValue::Number(n) => Some(*n),
            CssValue::Length(l) if l.unit == LengthUnit::Px => Some(l.value),
            _ => None,
        }
    }

    /// Check if this is "auto".
    pub fn is_auto(&self) -> bool {
        matches!(self, CssValue::Keyword(k) if k == "auto")
    }

    /// Check if this is "inherit".
    pub fn is_inherit(&self) -> bool {
        matches!(self, CssValue::Keyword(k) if k == "inherit")
    }

    /// Check if this is "initial".
    pub fn is_initial(&self) -> bool {
        matches!(self, CssValue::Keyword(k) if k == "initial")
    }

    /// Check if this is "none".
    pub fn is_none(&self) -> bool {
        matches!(self, CssValue::Keyword(k) if k == "none")
    }
}

/// A CSS length value.
#[derive(Debug, Clone, PartialEq)]
pub struct Length {
    pub value: f32,
    pub unit: LengthUnit,
}

impl Length {
    pub fn new(value: f32, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    pub fn px(value: f32) -> Self {
        Self::new(value, LengthUnit::Px)
    }

    pub fn rem(value: f32) -> Self {
        Self::new(value, LengthUnit::Rem)
    }

    pub fn em(value: f32) -> Self {
        Self::new(value, LengthUnit::Em)
    }

    pub fn percent(value: f32) -> Self {
        Self::new(value, LengthUnit::Percent)
    }

    pub fn vw(value: f32) -> Self {
        Self::new(value, LengthUnit::Vw)
    }

    pub fn vh(value: f32) -> Self {
        Self::new(value, LengthUnit::Vh)
    }

    pub fn zero() -> Self {
        Self::px(0.0)
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0.0
    }
}

/// CSS length units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthUnit {
    Px,
    Rem,
    Em,
    Percent,
    Vw,
    Vh,
    Vmin,
    Vmax,
}

impl LengthUnit {
    /// Parse a length unit from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "px" => Some(LengthUnit::Px),
            "rem" => Some(LengthUnit::Rem),
            "em" => Some(LengthUnit::Em),
            "%" => Some(LengthUnit::Percent),
            "vw" => Some(LengthUnit::Vw),
            "vh" => Some(LengthUnit::Vh),
            "vmin" => Some(LengthUnit::Vmin),
            "vmax" => Some(LengthUnit::Vmax),
            _ => None,
        }
    }
}

/// A calc() expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcExpr {
    Value(Length),
    Add(Box<CalcExpr>, Box<CalcExpr>),
    Sub(Box<CalcExpr>, Box<CalcExpr>),
    Mul(Box<CalcExpr>, f32),
    Div(Box<CalcExpr>, f32),
}

impl CalcExpr {
    /// Evaluate the calc expression to a length in pixels.
    pub fn evaluate(&self, base_px: f32, rem_px: f32, em_px: f32) -> f32 {
        match self {
            CalcExpr::Value(length) => match length.unit {
                LengthUnit::Px => length.value,
                LengthUnit::Rem => length.value * rem_px,
                LengthUnit::Em => length.value * em_px,
                LengthUnit::Percent => length.value / 100.0 * base_px,
                LengthUnit::Vw | LengthUnit::Vh | LengthUnit::Vmin | LengthUnit::Vmax => {
                    // Viewport units need window size context
                    length.value
                }
            },
            CalcExpr::Add(a, b) => {
                a.evaluate(base_px, rem_px, em_px) + b.evaluate(base_px, rem_px, em_px)
            }
            CalcExpr::Sub(a, b) => {
                a.evaluate(base_px, rem_px, em_px) - b.evaluate(base_px, rem_px, em_px)
            }
            CalcExpr::Mul(expr, n) => expr.evaluate(base_px, rem_px, em_px) * n,
            CalcExpr::Div(expr, n) => expr.evaluate(base_px, rem_px, em_px) / n,
        }
    }
}

/// Get a color from a CSS keyword.
fn color_from_keyword(keyword: &str) -> Option<Color> {
    match keyword.to_lowercase().as_str() {
        "transparent" => Some(Color::TRANSPARENT),
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::RED),
        "green" => Some(Color::from_rgb8(0, 128, 0)),
        "blue" => Some(Color::BLUE),
        "yellow" => Some(Color::from_rgb8(255, 255, 0)),
        "cyan" | "aqua" => Some(Color::from_rgb8(0, 255, 255)),
        "magenta" | "fuchsia" => Some(Color::from_rgb8(255, 0, 255)),
        "gray" | "grey" => Some(Color::from_rgb8(128, 128, 128)),
        "silver" => Some(Color::from_rgb8(192, 192, 192)),
        "maroon" => Some(Color::from_rgb8(128, 0, 0)),
        "olive" => Some(Color::from_rgb8(128, 128, 0)),
        "lime" => Some(Color::from_rgb8(0, 255, 0)),
        "teal" => Some(Color::from_rgb8(0, 128, 128)),
        "navy" => Some(Color::from_rgb8(0, 0, 128)),
        "purple" => Some(Color::from_rgb8(128, 0, 128)),
        "orange" => Some(Color::from_rgb8(255, 165, 0)),
        "pink" => Some(Color::from_rgb8(255, 192, 203)),
        "brown" => Some(Color::from_rgb8(165, 42, 42)),
        "currentcolor" | "currentColor" => None, // Needs context
        "inherit" => None,
        _ => None,
    }
}

/// Timing function for transitions/animations.
#[derive(Debug, Clone, PartialEq)]
#[derive(Default)]
#[allow(dead_code)]
pub enum TimingFunction {
    Linear,
    #[default]
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
    Steps(u32, StepPosition),
}

impl TimingFunction {
    /// Evaluate the timing function at time t (0.0 to 1.0).
    #[allow(dead_code)]
    pub fn evaluate(&self, t: f32) -> f32 {
        match self {
            TimingFunction::Linear => t,
            TimingFunction::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            TimingFunction::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
            TimingFunction::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
            TimingFunction::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
            TimingFunction::CubicBezier(x1, y1, x2, y2) => cubic_bezier(*x1, *y1, *x2, *y2, t),
            TimingFunction::Steps(steps, position) => {
                let step = (t * *steps as f32).floor() as u32;
                match position {
                    StepPosition::Start => (step + 1) as f32 / *steps as f32,
                    StepPosition::End => step as f32 / *steps as f32,
                }
            }
        }
    }
}


/// Step position for steps() timing function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StepPosition {
    Start,
    End,
}

/// Evaluate a cubic bezier curve at time t.
#[allow(dead_code)]
fn cubic_bezier(_x1: f32, y1: f32, _x2: f32, y2: f32, t: f32) -> f32 {
    // Simple approximation using Newton-Raphson method
    // For a proper implementation, would need to solve for t from x
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let _mt3 = mt2 * mt;

    // Simplified: assume x roughly equals t for typical easing curves
    3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_creation() {
        let l = Length::px(10.0);
        assert_eq!(l.value, 10.0);
        assert_eq!(l.unit, LengthUnit::Px);

        let l = Length::rem(1.5);
        assert_eq!(l.value, 1.5);
        assert_eq!(l.unit, LengthUnit::Rem);
    }

    #[test]
    fn test_css_value_as_color() {
        let v = CssValue::Keyword("red".to_string());
        assert!(v.as_color().is_some());

        let v = CssValue::Color(Color::BLUE);
        assert_eq!(v.as_color(), Some(Color::BLUE));
    }

    #[test]
    fn test_timing_function() {
        let linear = TimingFunction::Linear;
        assert_eq!(linear.evaluate(0.5), 0.5);

        let ease = TimingFunction::Ease;
        let mid = ease.evaluate(0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }
}
