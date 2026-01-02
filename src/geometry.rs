//! Geometry primitives for OpenKit.
//!
//! # Performance
//!
//! All geometry operations are optimized for speed with:
//! - `#[inline]` hints on hot paths
//! - `const fn` where possible
//! - Copy semantics for small structs
//! - SIMD-friendly data layouts

use std::ops::{Add, Sub, Mul};

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn distance(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Squared distance (faster than distance, use for comparisons).
    #[inline(always)]
    pub fn distance_squared(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Linear interpolation between two points.
    #[inline]
    pub fn lerp(&self, other: &Point, t: f32) -> Point {
        Point::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

impl Add for Point {
    type Output = Point;

    #[inline(always)]
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Point;

    #[inline(always)]
    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

/// A 2D size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size { width: 0.0, height: 0.0 };

    #[inline(always)]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[inline(always)]
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    #[inline]
    pub fn contains(&self, other: &Size) -> bool {
        self.width >= other.width && self.height >= other.height
    }

    /// Component-wise maximum.
    #[inline]
    pub fn max(&self, other: Size) -> Size {
        Size::new(
            self.width.max(other.width),
            self.height.max(other.height),
        )
    }

    /// Component-wise minimum.
    #[inline]
    pub fn min(&self, other: Size) -> Size {
        Size::new(
            self.width.min(other.width),
            self.height.min(other.height),
        )
    }

    /// Clamp size to constraints.
    #[inline]
    pub fn clamp(&self, min: Size, max: Size) -> Size {
        Size::new(
            self.width.clamp(min.width, max.width),
            self.height.clamp(min.height, max.height),
        )
    }
}

impl Mul<f32> for Size {
    type Output = Size;

    #[inline(always)]
    fn mul(self, scale: f32) -> Size {
        Size::new(self.width * scale, self.height * scale)
    }
}

/// A 2D rectangle defined by origin and size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    #[inline(always)]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    #[inline(always)]
    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.origin.x
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.origin.y
    }

    #[inline(always)]
    pub fn width(&self) -> f32 {
        self.size.width
    }

    #[inline(always)]
    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn min_x(&self) -> f32 {
        self.origin.x
    }

    pub fn min_y(&self) -> f32 {
        self.origin.y
    }

    #[inline(always)]
    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    #[inline(always)]
    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    /// Check if a point is inside this rectangle.
    #[inline]
    pub fn contains(&self, point: Point) -> bool {
        // Optimized: inline the min/max calls to avoid method overhead
        let x = self.origin.x;
        let y = self.origin.y;
        let max_x = x + self.size.width;
        let max_y = y + self.size.height;
        point.x >= x && point.x <= max_x && point.y >= y && point.y <= max_y
    }

    /// Check if this rectangle intersects with another.
    #[inline]
    pub fn intersects(&self, other: &Rect) -> bool {
        // Optimized: inline calculations for better performance
        let ax = self.origin.x;
        let ay = self.origin.y;
        let ax2 = ax + self.size.width;
        let ay2 = ay + self.size.height;
        
        let bx = other.origin.x;
        let by = other.origin.y;
        let bx2 = bx + other.size.width;
        let by2 = by + other.size.height;
        
        ax < bx2 && ax2 > bx && ay < by2 && ay2 > by
    }

    pub fn inset(&self, amount: f32) -> Rect {
        Rect::new(
            self.origin.x + amount,
            self.origin.y + amount,
            (self.size.width - 2.0 * amount).max(0.0),
            (self.size.height - 2.0 * amount).max(0.0),
        )
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(
            self.origin.x + dx,
            self.origin.y + dy,
            self.size.width,
            self.size.height,
        )
    }
}

/// A color in RGBA format.
///
/// Colors are stored as f32 values in the range 0.0-1.0.
/// All operations are optimized with inline hints for performance.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);

    #[inline(always)]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[inline(always)]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from 8-bit RGB values (0-255).
    #[inline]
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        // Multiply by reciprocal for speed (compiler may optimize this)
        const INV_255: f32 = 1.0 / 255.0;
        Self::rgb(r as f32 * INV_255, g as f32 * INV_255, b as f32 * INV_255)
    }

    /// Create a color from 8-bit RGBA values (0-255).
    #[inline]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        const INV_255: f32 = 1.0 / 255.0;
        Self::rgba(
            r as f32 * INV_255,
            g as f32 * INV_255,
            b as f32 * INV_255,
            a as f32 * INV_255,
        )
    }

    /// Create a color from a hex string (e.g., "#ff0000" or "ff0000").
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::from_rgb8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            3 => {
                // Short form: #rgb -> #rrggbb
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self::from_rgb8(r, g, b))
            }
            _ => None,
        }
    }

    /// Create a color from HSL values.
    /// h: 0-360, s: 0-100, l: 0-100
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let s = s / 100.0;
        let l = l / 100.0;
        let h = h / 360.0;

        if s == 0.0 {
            return Self::rgb(l, l, l);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 1.0 / 2.0 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        }

        Self::rgb(
            hue_to_rgb(p, q, h + 1.0 / 3.0),
            hue_to_rgb(p, q, h),
            hue_to_rgb(p, q, h - 1.0 / 3.0),
        )
    }

    /// Convert to 8-bit RGBA values.
    #[inline]
    pub fn to_rgba8(&self) -> [u8; 4] {
        // Fast path: avoid clamp for well-formed colors
        [
            (self.r * 255.0 + 0.5) as u8,
            (self.g * 255.0 + 0.5) as u8,
            (self.b * 255.0 + 0.5) as u8,
            (self.a * 255.0 + 0.5) as u8,
        ]
    }

    /// Convert to f32 RGBA values (0.0-1.0).
    #[inline(always)]
    pub fn to_rgba_f32(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Blend this color with another using alpha compositing.
    #[inline]
    pub fn blend(&self, other: &Color) -> Color {
        let a = other.a + self.a * (1.0 - other.a);
        if a == 0.0 {
            return Color::TRANSPARENT;
        }
        Color::rgba(
            (other.r * other.a + self.r * self.a * (1.0 - other.a)) / a,
            (other.g * other.a + self.g * self.a * (1.0 - other.a)) / a,
            (other.b * other.a + self.b * self.a * (1.0 - other.a)) / a,
            a,
        )
    }

    /// Darken the color by a percentage (0-100).
    pub fn darken(&self, amount: f32) -> Color {
        let factor = 1.0 - (amount / 100.0);
        Color::rgba(
            (self.r * factor).clamp(0.0, 1.0),
            (self.g * factor).clamp(0.0, 1.0),
            (self.b * factor).clamp(0.0, 1.0),
            self.a,
        )
    }

    /// Lighten the color by a percentage (0-100).
    pub fn lighten(&self, amount: f32) -> Color {
        let factor = amount / 100.0;
        Color::rgba(
            (self.r + (1.0 - self.r) * factor).clamp(0.0, 1.0),
            (self.g + (1.0 - self.g) * factor).clamp(0.0, 1.0),
            (self.b + (1.0 - self.b) * factor).clamp(0.0, 1.0),
            self.a,
        )
    }

    /// Set the alpha value.
    pub fn with_alpha(&self, alpha: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, alpha)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Border radius for rounded rectangles.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl BorderRadius {
    pub const ZERO: BorderRadius = BorderRadius::all(0.0);

    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }

    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }
}

impl From<f32> for BorderRadius {
    fn from(radius: f32) -> Self {
        Self::all(radius)
    }
}

/// Edge insets (padding, margin, border).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const ZERO: EdgeInsets = EdgeInsets::all(0.0);

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl From<f32> for EdgeInsets {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

impl From<(f32, f32)> for EdgeInsets {
    fn from((vertical, horizontal): (f32, f32)) -> Self {
        Self::symmetric(vertical, horizontal)
    }
}

impl From<(f32, f32, f32, f32)> for EdgeInsets {
    fn from((top, right, bottom, left): (f32, f32, f32, f32)) -> Self {
        Self::new(top, right, bottom, left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#ff0000").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);

        let color = Color::from_hex("00ff00").unwrap();
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 0.0);

        let color = Color::from_hex("#fff").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 1.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains(Point::new(50.0, 30.0)));
        assert!(!rect.contains(Point::new(5.0, 30.0)));
        assert!(!rect.contains(Point::new(50.0, 70.0)));
    }

    #[test]
    fn test_color_from_hsl() {
        // Red
        let color = Color::from_hsl(0.0, 100.0, 50.0);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!(color.g.abs() < 0.01);
        assert!(color.b.abs() < 0.01);

        // Green
        let color = Color::from_hsl(120.0, 100.0, 50.0);
        assert!(color.r.abs() < 0.01);
        assert!((color.g - 1.0).abs() < 0.01);
        assert!(color.b.abs() < 0.01);
    }
}
