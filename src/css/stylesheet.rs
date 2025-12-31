//! CSS stylesheet and rule management.

use crate::css::{
    CssValue, ComputedStyle, Selector, StyleContext, StyleProperty, WidgetState,
};
use crate::css::selector::Specificity;
use std::collections::HashMap;

/// A CSS stylesheet containing rules.
#[derive(Debug, Clone, Default)]
pub struct StyleSheet {
    pub rules: Vec<StyleRule>,
}

impl StyleSheet {
    pub fn new(rules: Vec<StyleRule>) -> Self {
        Self { rules }
    }

    /// Create an empty stylesheet.
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the stylesheet.
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }

    /// Find all matching rules for a widget.
    pub fn find_matching_rules<'a>(
        &'a self,
        widget_type: &str,
        widget_id: Option<&str>,
        classes: &[String],
        state: &WidgetState,
    ) -> Vec<(&'a StyleRule, Specificity)> {
        self.rules
            .iter()
            .filter_map(|rule| {
                if rule.selector.matches(widget_type, widget_id, classes, state) {
                    Some((rule, rule.selector.specificity))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Compute styles for a widget by applying all matching rules in specificity order.
    pub fn compute_style(
        &self,
        widget_type: &str,
        widget_id: Option<&str>,
        classes: &[String],
        state: &WidgetState,
        ctx: &StyleContext,
    ) -> ComputedStyle {
        let mut style = ComputedStyle::default();

        // Find all matching rules
        let mut matches = self.find_matching_rules(widget_type, widget_id, classes, state);

        // Sort by specificity (ascending so higher specificity overrides)
        matches.sort_by(|a, b| a.1.cmp(&b.1));

        // Apply rules in order
        for (rule, _) in matches {
            for (property, value) in &rule.declarations {
                style.apply(property, value, ctx);
            }
        }

        style
    }

    /// Merge another stylesheet into this one.
    pub fn merge(&mut self, other: StyleSheet) {
        self.rules.extend(other.rules);
    }
}

/// A single CSS rule (selector + declarations).
#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: Selector,
    pub declarations: HashMap<StyleProperty, CssValue>,
}

impl StyleRule {
    pub fn new(selector: Selector, declarations: HashMap<StyleProperty, CssValue>) -> Self {
        Self {
            selector,
            declarations,
        }
    }

    /// Create a rule from selector and a builder closure.
    pub fn build<F>(selector: Selector, builder: F) -> Self
    where
        F: FnOnce(&mut HashMap<StyleProperty, CssValue>),
    {
        let mut declarations = HashMap::new();
        builder(&mut declarations);
        Self::new(selector, declarations)
    }
}

/// Builder for creating stylesheets programmatically.
pub struct StyleSheetBuilder {
    rules: Vec<StyleRule>,
}

impl StyleSheetBuilder {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule for a class selector.
    pub fn class(self, name: &str) -> RuleBuilder {
        RuleBuilder {
            builder: self,
            selector: Selector::class(name),
            declarations: HashMap::new(),
        }
    }

    /// Add a rule for a type selector.
    pub fn type_selector(self, name: &str) -> RuleBuilder {
        RuleBuilder {
            builder: self,
            selector: Selector::type_selector(name),
            declarations: HashMap::new(),
        }
    }

    /// Add a pre-built rule.
    pub fn rule(mut self, rule: StyleRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Build the stylesheet.
    pub fn build(self) -> StyleSheet {
        StyleSheet::new(self.rules)
    }
}

impl Default for StyleSheetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for a single rule.
pub struct RuleBuilder {
    builder: StyleSheetBuilder,
    selector: Selector,
    declarations: HashMap<StyleProperty, CssValue>,
}

impl RuleBuilder {
    /// Set a property value.
    pub fn set(mut self, property: StyleProperty, value: CssValue) -> Self {
        self.declarations.insert(property, value);
        self
    }

    /// Set background color.
    pub fn background_color(self, color: crate::geometry::Color) -> Self {
        self.set(StyleProperty::BackgroundColor, CssValue::Color(color))
    }

    /// Set text color.
    pub fn color(self, color: crate::geometry::Color) -> Self {
        self.set(StyleProperty::Color, CssValue::Color(color))
    }

    /// Set padding (all sides).
    pub fn padding(self, px: f32) -> Self {
        self.set(
            StyleProperty::Padding,
            CssValue::Length(crate::css::Length::px(px)),
        )
    }

    /// Set border radius.
    pub fn border_radius(self, px: f32) -> Self {
        self.set(
            StyleProperty::BorderRadius,
            CssValue::Length(crate::css::Length::px(px)),
        )
    }

    /// Set font size.
    pub fn font_size(self, px: f32) -> Self {
        self.set(
            StyleProperty::FontSize,
            CssValue::Length(crate::css::Length::px(px)),
        )
    }

    /// Add a pseudo-class variant.
    pub fn pseudo(mut self, pseudo: crate::css::PseudoClass) -> RuleBuilder {
        // First, finish the current rule
        let rule = StyleRule::new(self.selector.clone(), self.declarations);
        self.builder.rules.push(rule);

        // Start a new rule with the pseudo-class
        RuleBuilder {
            builder: self.builder,
            selector: self.selector.pseudo(pseudo),
            declarations: HashMap::new(),
        }
    }

    /// Finish this rule and continue building the stylesheet.
    pub fn done(mut self) -> StyleSheetBuilder {
        let rule = StyleRule::new(self.selector, self.declarations);
        self.builder.rules.push(rule);
        self.builder
    }

    /// Finish this rule and build the stylesheet.
    pub fn build(self) -> StyleSheet {
        self.done().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Color;
    use crate::theme::ThemeData;

    #[test]
    fn test_stylesheet_builder() {
        let stylesheet = StyleSheetBuilder::new()
            .class("button")
            .background_color(Color::BLUE)
            .padding(8.0)
            .border_radius(4.0)
            .done()
            .build();

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_matching_rules() {
        let stylesheet = StyleSheetBuilder::new()
            .class("primary")
            .background_color(Color::BLUE)
            .done()
            .class("secondary")
            .background_color(Color::from_hex("#666").unwrap())
            .done()
            .build();

        let state = WidgetState::default();
        let matches = stylesheet.find_matching_rules(
            "button",
            None,
            &["primary".to_string()],
            &state,
        );

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_compute_style() {
        let stylesheet = StyleSheetBuilder::new()
            .class("button")
            .background_color(Color::BLUE)
            .padding(8.0)
            .done()
            .build();

        let theme = ThemeData::light();
        let ctx = StyleContext::new(&theme);
        let state = WidgetState::default();

        let style = stylesheet.compute_style(
            "button",
            None,
            &["button".to_string()],
            &state,
            &ctx,
        );

        assert_eq!(style.background_color, Color::BLUE);
    }
}
