//! CSS parsing and style engine for OpenKit.
//!
//! ## Loading Custom CSS
//!
//! OpenKit supports loading custom CSS to override framework styles:
//!
//! ```rust,ignore
//! use openkit::prelude::*;
//! use openkit::css::StyleManager;
//!
//! let mut styles = StyleManager::new();
//! styles.load_file("./custom.css")?;
//! styles.load_css(r#"
//!     .my-button { background-color: #3b82f6; }
//! "#)?;
//!
//! App::new()
//!     .styles(styles)
//!     .run(|| view!());
//! ```

mod loader;
mod parser;
pub mod properties;
mod selector;
mod stylesheet;
mod value;

pub use loader::{StyleManager, StyleBuilder, CssLoadError};
pub use parser::CssParser;
pub use properties::{ComputedStyle, StyleProperty, AlignItems, FlexDirection, JustifyContent};
pub use selector::{Selector, SelectorPart, PseudoClass, Specificity};
pub use stylesheet::{StyleSheet, StyleRule, StyleSheetBuilder, RuleBuilder};
pub use value::{CssValue, Length, LengthUnit};

use crate::theme::ThemeData;
use std::collections::HashMap;
use std::sync::Arc;

/// Style context for resolving styles.
///
/// Contains everything needed to compute styles for a widget:
/// - Theme data for design tokens
/// - Style manager for custom stylesheets
/// - Viewport information for responsive units
#[derive(Debug, Clone)]
pub struct StyleContext<'a> {
    pub theme: &'a ThemeData,
    pub style_manager: Option<Arc<StyleManager>>,
    pub parent_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl<'a> StyleContext<'a> {
    pub fn new(theme: &'a ThemeData) -> Self {
        Self {
            theme,
            style_manager: None,
            parent_font_size: theme.typography.base_size,
            viewport_width: 1920.0,
            viewport_height: 1080.0,
        }
    }

    /// Create a style context with a custom StyleManager.
    pub fn with_styles(theme: &'a ThemeData, styles: Arc<StyleManager>) -> Self {
        Self {
            theme,
            style_manager: Some(styles),
            parent_font_size: theme.typography.base_size,
            viewport_width: 1920.0,
            viewport_height: 1080.0,
        }
    }

    /// Set the viewport size for responsive units.
    pub fn with_viewport(mut self, width: f32, height: f32) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    /// Resolve a CSS variable to its value.
    ///
    /// First checks the StyleManager's custom variables, then falls back to theme.
    pub fn resolve_var(&self, name: &str) -> Option<String> {
        // First check custom CSS variables
        if let Some(sm) = &self.style_manager {
            if let Some(value) = sm.get_variable(name) {
                return Some(value.clone());
            }
        }
        // Fall back to theme
        self.theme.resolve_var(name)
    }

    /// Get the combined stylesheet (custom + default).
    pub fn combined_stylesheet(&self) -> StyleSheet {
        if let Some(sm) = &self.style_manager {
            sm.combined_stylesheet()
        } else {
            StyleSheet::default()
        }
    }

    /// Compute styles for a widget.
    pub fn compute_style(
        &self,
        widget_type: &str,
        widget_id: Option<&str>,
        classes: &[String],
        state: &WidgetState,
    ) -> ComputedStyle {
        let stylesheet = self.combined_stylesheet();
        stylesheet.compute_style(widget_type, widget_id, classes, state, self)
    }

    /// Convert a length to pixels.
    pub fn to_pixels(&self, length: &Length) -> f32 {
        match length.unit {
            LengthUnit::Px => length.value,
            LengthUnit::Rem => length.value * self.theme.typography.base_size,
            LengthUnit::Em => length.value * self.parent_font_size,
            LengthUnit::Percent => length.value / 100.0, // Needs context
            LengthUnit::Vw => length.value * self.viewport_width / 100.0,
            LengthUnit::Vh => length.value * self.viewport_height / 100.0,
            LengthUnit::Vmin => length.value * self.viewport_width.min(self.viewport_height) / 100.0,
            LengthUnit::Vmax => length.value * self.viewport_width.max(self.viewport_height) / 100.0,
        }
    }
}

/// Widget state for pseudo-class matching.
#[derive(Debug, Clone, Copy, Default)]
pub struct WidgetState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub checked: bool,
    pub first_child: bool,
    pub last_child: bool,
    pub nth_child: usize,
}

impl WidgetState {
    pub fn matches(&self, pseudo: &PseudoClass) -> bool {
        match pseudo {
            PseudoClass::Hover => self.hovered,
            PseudoClass::Active => self.pressed,
            PseudoClass::Focus => self.focused,
            PseudoClass::FocusVisible => self.focused, // Simplified
            PseudoClass::Disabled => self.disabled,
            PseudoClass::Enabled => !self.disabled,
            PseudoClass::Checked => self.checked,
            PseudoClass::FirstChild => self.first_child,
            PseudoClass::LastChild => self.last_child,
            PseudoClass::NthChild(n) => self.nth_child == *n,
            PseudoClass::Not(_) => true, // TODO: Implement
        }
    }
}

/// CSS class list for a widget.
#[derive(Debug, Clone, Default)]
pub struct ClassList {
    classes: Vec<String>,
}

impl ClassList {
    pub fn new() -> Self {
        Self { classes: Vec::new() }
    }

    pub fn add(&mut self, class: impl Into<String>) {
        let class = class.into();
        if !self.classes.contains(&class) {
            self.classes.push(class);
        }
    }

    pub fn remove(&mut self, class: &str) {
        self.classes.retain(|c| c != class);
    }

    pub fn toggle(&mut self, class: impl Into<String>) {
        let class = class.into();
        if self.classes.contains(&class) {
            self.remove(&class);
        } else {
            self.classes.push(class);
        }
    }

    pub fn contains(&self, class: &str) -> bool {
        self.classes.contains(&class.to_string())
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.classes.iter()
    }
}

impl From<&str> for ClassList {
    fn from(s: &str) -> Self {
        let classes = s
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        Self { classes }
    }
}

impl From<Vec<String>> for ClassList {
    fn from(classes: Vec<String>) -> Self {
        Self { classes }
    }
}

/// Inline styles for a widget.
#[derive(Debug, Clone, Default)]
pub struct InlineStyle {
    properties: HashMap<String, CssValue>,
}

impl InlineStyle {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn set(&mut self, property: impl Into<String>, value: CssValue) {
        self.properties.insert(property.into(), value);
    }

    pub fn get(&self, property: &str) -> Option<&CssValue> {
        self.properties.get(property)
    }

    pub fn remove(&mut self, property: &str) {
        self.properties.remove(property);
    }
}
