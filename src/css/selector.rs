//! CSS selector types and matching.

use crate::css::WidgetState;

/// A CSS selector.
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub parts: Vec<SelectorPart>,
    pub specificity: Specificity,
}

impl Selector {
    pub fn new(parts: Vec<SelectorPart>) -> Self {
        let specificity = Self::calculate_specificity(&parts);
        Self { parts, specificity }
    }

    /// Create a type selector (e.g., "button").
    pub fn type_selector(name: &str) -> Self {
        Self::new(vec![SelectorPart::Type(name.to_string())])
    }

    /// Create a class selector (e.g., ".primary").
    pub fn class(name: &str) -> Self {
        Self::new(vec![SelectorPart::Class(name.to_string())])
    }

    /// Create an ID selector (e.g., "#main").
    pub fn id(name: &str) -> Self {
        Self::new(vec![SelectorPart::Id(name.to_string())])
    }

    /// Add a pseudo-class to this selector.
    pub fn pseudo(mut self, pseudo: PseudoClass) -> Self {
        self.parts.push(SelectorPart::PseudoClass(pseudo));
        self.specificity = Self::calculate_specificity(&self.parts);
        self
    }

    /// Calculate specificity (ID, class, type).
    fn calculate_specificity(parts: &[SelectorPart]) -> Specificity {
        let mut ids = 0u32;
        let mut classes = 0u32;
        let mut types = 0u32;

        for part in parts {
            match part {
                SelectorPart::Id(_) => ids += 1,
                SelectorPart::Class(_) | SelectorPart::PseudoClass(_) | SelectorPart::Attribute { .. } => {
                    classes += 1
                }
                SelectorPart::Type(_) | SelectorPart::PseudoElement(_) => types += 1,
                SelectorPart::Universal => {}
                SelectorPart::Combinator(_) => {}
            }
        }

        Specificity(ids, classes, types)
    }

    /// Check if this selector matches a widget.
    pub fn matches(
        &self,
        widget_type: &str,
        widget_id: Option<&str>,
        classes: &[String],
        state: &WidgetState,
    ) -> bool {
        for part in &self.parts {
            match part {
                SelectorPart::Universal => continue,
                SelectorPart::Type(name) => {
                    if name != widget_type {
                        return false;
                    }
                }
                SelectorPart::Class(name) => {
                    if !classes.contains(name) {
                        return false;
                    }
                }
                SelectorPart::Id(name) => {
                    if widget_id != Some(name.as_str()) {
                        return false;
                    }
                }
                SelectorPart::PseudoClass(pseudo) => {
                    if !state.matches(pseudo) {
                        return false;
                    }
                }
                SelectorPart::Attribute { name: _, op: _, value: _ } => {
                    // TODO: Implement attribute matching
                    return false;
                }
                SelectorPart::PseudoElement(_) => {
                    // Pseudo-elements don't affect matching
                }
                SelectorPart::Combinator(_) => {
                    // TODO: Implement combinator matching
                }
            }
        }
        true
    }
}

/// A part of a CSS selector.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorPart {
    /// Universal selector (*)
    Universal,
    /// Type selector (e.g., "button")
    Type(String),
    /// Class selector (e.g., ".primary")
    Class(String),
    /// ID selector (e.g., "#main")
    Id(String),
    /// Attribute selector (e.g., "[type=text]")
    Attribute {
        name: String,
        op: AttributeOp,
        value: Option<String>,
    },
    /// Pseudo-class (e.g., ":hover")
    PseudoClass(PseudoClass),
    /// Pseudo-element (e.g., "::before")
    PseudoElement(String),
    /// Combinator
    Combinator(Combinator),
}

/// Attribute selector operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOp {
    /// [attr] - has attribute
    Exists,
    /// [attr=value] - exact match
    Equals,
    /// [attr~=value] - word in space-separated list
    Contains,
    /// [attr|=value] - starts with value or value-
    DashMatch,
    /// [attr^=value] - starts with
    Prefix,
    /// [attr$=value] - ends with
    Suffix,
    /// [attr*=value] - contains substring
    Substring,
}

/// CSS pseudo-classes.
#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClass {
    Hover,
    Active,
    Focus,
    FocusVisible,
    Disabled,
    Enabled,
    Checked,
    FirstChild,
    LastChild,
    NthChild(usize),
    Not(Box<Selector>),
}

impl PseudoClass {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "hover" => Some(PseudoClass::Hover),
            "active" => Some(PseudoClass::Active),
            "focus" => Some(PseudoClass::Focus),
            "focus-visible" => Some(PseudoClass::FocusVisible),
            "disabled" => Some(PseudoClass::Disabled),
            "enabled" => Some(PseudoClass::Enabled),
            "checked" => Some(PseudoClass::Checked),
            "first-child" => Some(PseudoClass::FirstChild),
            "last-child" => Some(PseudoClass::LastChild),
            _ => None,
        }
    }
}

/// CSS combinators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (space)
    Descendant,
    /// Child combinator (>)
    Child,
    /// Next sibling combinator (+)
    NextSibling,
    /// Subsequent sibling combinator (~)
    SubsequentSibling,
}

/// CSS specificity (a, b, c).
/// a = ID selectors
/// b = class selectors, attribute selectors, pseudo-classes
/// c = type selectors, pseudo-elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Specificity(pub u32, pub u32, pub u32);

impl Specificity {
    pub fn new(ids: u32, classes: u32, types: u32) -> Self {
        Self(ids, classes, types)
    }

    /// Inline styles have highest specificity.
    pub fn inline() -> Self {
        Self(u32::MAX, u32::MAX, u32::MAX)
    }

    /// Convert to a comparable number.
    pub fn to_value(&self) -> u64 {
        (self.0 as u64) << 32 | (self.1 as u64) << 16 | (self.2 as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_matching() {
        let selector = Selector::class("primary");
        let state = WidgetState::default();

        assert!(selector.matches("button", None, &["primary".to_string()], &state));
        assert!(!selector.matches("button", None, &["secondary".to_string()], &state));
    }

    #[test]
    fn test_pseudo_class_matching() {
        let selector = Selector::class("btn").pseudo(PseudoClass::Hover);

        let mut state = WidgetState::default();
        assert!(!selector.matches("button", None, &["btn".to_string()], &state));

        state.hovered = true;
        assert!(selector.matches("button", None, &["btn".to_string()], &state));
    }

    #[test]
    fn test_specificity_ordering() {
        let id = Specificity::new(1, 0, 0);
        let class = Specificity::new(0, 1, 0);
        let type_sel = Specificity::new(0, 0, 1);

        assert!(id > class);
        assert!(class > type_sel);
    }
}
