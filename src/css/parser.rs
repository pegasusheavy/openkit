//! CSS parser using the cssparser crate.

use cssparser::{Parser, ParserInput, Token, ParseError};
use crate::css::{
    CssValue, Length, LengthUnit, Selector, SelectorPart, PseudoClass,
    StyleProperty, StyleRule, StyleSheet,
};
use crate::geometry::Color;
use std::collections::HashMap;

/// CSS parser for OpenKit.
pub struct CssParser;

impl CssParser {
    /// Parse a CSS stylesheet.
    pub fn parse_stylesheet(css: &str) -> Result<StyleSheet, CssParseError> {
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);
        let mut rules = Vec::new();

        while !parser.is_exhausted() {
            // Skip whitespace and comments
            let _ = parser.try_parse::<_, _, ParseError<'_, ()>>(|p| {
                p.skip_whitespace();
                Ok(())
            });

            if parser.is_exhausted() {
                break;
            }

            match Self::parse_rule(&mut parser) {
                Ok(rule) => rules.push(rule),
                Err(_) => {
                    // Skip to next rule on error
                    let _ = parser.next();
                }
            }
        }

        Ok(StyleSheet::new(rules))
    }

    /// Parse a single CSS rule.
    fn parse_rule<'i>(parser: &mut Parser<'i, '_>) -> Result<StyleRule, CssParseError> {
        // Parse selector
        let selector = Self::parse_selector(parser)?;

        // Parse declaration block
        let declarations = parser.parse_nested_block(|p| {
            Self::parse_declarations(p)
        }).map_err(|_| CssParseError::InvalidDeclaration)?;

        Ok(StyleRule::new(selector, declarations))
    }

    /// Parse a CSS selector.
    fn parse_selector<'i>(parser: &mut Parser<'i, '_>) -> Result<Selector, CssParseError> {
        let mut parts = Vec::new();

        loop {
            parser.skip_whitespace();

            let token = match parser.next() {
                Ok(t) => t.clone(),
                Err(_) => break,
            };

            match token {
                Token::Ident(name) => {
                    parts.push(SelectorPart::Type(name.to_string()));
                }
                Token::IDHash(name) => {
                    parts.push(SelectorPart::Id(name.to_string()));
                }
                Token::Delim('.') => {
                    // Class selector
                    if let Ok(Token::Ident(name)) = parser.next() {
                        parts.push(SelectorPart::Class(name.to_string()));
                    }
                }
                Token::Delim('*') => {
                    parts.push(SelectorPart::Universal);
                }
                Token::Colon => {
                    // Pseudo-class
                    if let Ok(Token::Ident(name)) = parser.next() {
                        if let Some(pseudo) = PseudoClass::from_name(name) {
                            parts.push(SelectorPart::PseudoClass(pseudo));
                        }
                    }
                }
                Token::CurlyBracketBlock => {
                    // Start of declaration block, we're done with selector
                    break;
                }
                _ => {
                    // Unknown token, might be start of block
                    break;
                }
            }
        }

        if parts.is_empty() {
            return Err(CssParseError::InvalidSelector);
        }

        Ok(Selector::new(parts))
    }

    /// Parse CSS declarations within a block.
    fn parse_declarations<'i>(
        parser: &mut Parser<'i, '_>,
    ) -> Result<HashMap<StyleProperty, CssValue>, ParseError<'i, ()>> {
        let mut declarations = HashMap::new();

        loop {
            parser.skip_whitespace();

            if parser.is_exhausted() {
                break;
            }

            // Parse property name
            let property_name = match parser.next() {
                Ok(Token::Ident(name)) => name.to_string(),
                Ok(Token::Semicolon) => continue,
                _ => continue,
            };

            // Skip colon
            parser.skip_whitespace();
            if parser.expect_colon().is_err() {
                continue;
            }

            // Parse value
            parser.skip_whitespace();
            if let Ok(value) = Self::parse_value(parser) {
                let property = StyleProperty::from_name(&property_name);
                declarations.insert(property, value);
            }

            // Skip semicolon
            let _ = parser.try_parse::<_, _, ParseError<'_, ()>>(|p| {
                p.expect_semicolon()?;
                Ok(())
            });
        }

        Ok(declarations)
    }

    /// Parse a CSS value.
    fn parse_value<'i>(parser: &mut Parser<'i, '_>) -> Result<CssValue, CssParseError> {
        let token = parser.next().map_err(|_| CssParseError::InvalidValue)?;

        match token {
            Token::Ident(name) => {
                let name = name.to_string();

                // Check for color keywords
                if let Some(color) = Self::color_from_keyword(&name) {
                    return Ok(CssValue::Color(color));
                }

                Ok(CssValue::Keyword(name))
            }
            Token::Hash(hash) | Token::IDHash(hash) => {
                // Hex color
                if let Some(color) = Color::from_hex(hash) {
                    Ok(CssValue::Color(color))
                } else {
                    Err(CssParseError::InvalidValue)
                }
            }
            Token::Number { value, .. } => {
                Ok(CssValue::Number(*value))
            }
            Token::Percentage { unit_value, .. } => {
                Ok(CssValue::Percentage(*unit_value * 100.0))
            }
            Token::Dimension { value, unit, .. } => {
                if let Some(length_unit) = LengthUnit::parse(unit) {
                    Ok(CssValue::Length(Length::new(*value, length_unit)))
                } else {
                    Err(CssParseError::InvalidValue)
                }
            }
            Token::QuotedString(s) => {
                Ok(CssValue::String(s.to_string()))
            }
            Token::Function(name) => {
                let name = name.to_string();
                parser.parse_nested_block(|p| {
                    Self::parse_function(&name, p)
                }).map_err(|_| CssParseError::InvalidValue)
            }
            _ => Err(CssParseError::InvalidValue),
        }
    }

    /// Parse a CSS function.
    fn parse_function<'i>(
        name: &str,
        parser: &mut Parser<'i, '_>,
    ) -> Result<CssValue, ParseError<'i, ()>> {
        match name {
            "rgb" | "rgba" => Self::parse_rgb_function(parser),
            "hsl" | "hsla" => Self::parse_hsl_function(parser),
            "var" => Self::parse_var_function(parser),
            "calc" => Ok(CssValue::Keyword("calc(...)".to_string())), // Simplified
            _ => Ok(CssValue::Keyword(format!("{}(...)", name))),
        }
    }

    /// Parse rgb() or rgba() function.
    fn parse_rgb_function<'i>(parser: &mut Parser<'i, '_>) -> Result<CssValue, ParseError<'i, ()>> {
        let mut values = Vec::new();

        loop {
            parser.skip_whitespace();

            match parser.next() {
                Ok(Token::Number { value, .. }) => values.push(*value),
                Ok(Token::Percentage { unit_value, .. }) => values.push(*unit_value * 255.0),
                Ok(Token::Comma) => continue,
                Ok(Token::Delim('/')) => continue, // For rgba with alpha
                Err(_) => break,
                _ => continue,
            }
        }

        match values.len() {
            3 => {
                let color = Color::from_rgb8(
                    values[0] as u8,
                    values[1] as u8,
                    values[2] as u8,
                );
                Ok(CssValue::Color(color))
            }
            4 => {
                let alpha = if values[3] <= 1.0 {
                    (values[3] * 255.0) as u8
                } else {
                    values[3] as u8
                };
                let color = Color::from_rgba8(
                    values[0] as u8,
                    values[1] as u8,
                    values[2] as u8,
                    alpha,
                );
                Ok(CssValue::Color(color))
            }
            _ => Ok(CssValue::Keyword("rgb()".to_string())),
        }
    }

    /// Parse hsl() or hsla() function.
    fn parse_hsl_function<'i>(parser: &mut Parser<'i, '_>) -> Result<CssValue, ParseError<'i, ()>> {
        let mut values = Vec::new();

        loop {
            parser.skip_whitespace();

            match parser.next() {
                Ok(Token::Number { value, .. }) => values.push(*value),
                Ok(Token::Percentage { unit_value, .. }) => values.push(*unit_value * 100.0),
                Ok(Token::Comma) => continue,
                Ok(Token::Delim('/')) => continue,
                Err(_) => break,
                _ => continue,
            }
        }

        if values.len() >= 3 {
            let color = Color::from_hsl(values[0], values[1], values[2]);
            if values.len() >= 4 {
                let alpha = if values[3] <= 1.0 { values[3] } else { values[3] / 100.0 };
                Ok(CssValue::Color(color.with_alpha(alpha)))
            } else {
                Ok(CssValue::Color(color))
            }
        } else {
            Ok(CssValue::Keyword("hsl()".to_string()))
        }
    }

    /// Parse var() function.
    fn parse_var_function<'i>(parser: &mut Parser<'i, '_>) -> Result<CssValue, ParseError<'i, ()>> {
        parser.skip_whitespace();

        let var_name = match parser.next() {
            Ok(Token::Ident(name)) => name.to_string(),
            _ => return Ok(CssValue::Keyword("var()".to_string())),
        };

        // Check for fallback value
        parser.skip_whitespace();
        let fallback = if let Ok(Token::Comma) = parser.next() {
            parser.skip_whitespace();
            CssParser::parse_value(parser).ok().map(Box::new)
        } else {
            None
        };

        Ok(CssValue::Var(var_name, fallback))
    }

    /// Parse inline style string.
    pub fn parse_inline_style(style: &str) -> HashMap<StyleProperty, CssValue> {
        let declarations = HashMap::new();
        let css = format!("x {{ {} }}", style);

        if let Ok(stylesheet) = Self::parse_stylesheet(&css) {
            if let Some(rule) = stylesheet.rules.first() {
                return rule.declarations.clone();
            }
        }

        declarations
    }

    /// Get color from keyword.
    fn color_from_keyword(keyword: &str) -> Option<Color> {
        match keyword.to_lowercase().as_str() {
            "transparent" => Some(Color::TRANSPARENT),
            "black" => Some(Color::BLACK),
            "white" => Some(Color::WHITE),
            "red" => Some(Color::from_rgb8(255, 0, 0)),
            "green" => Some(Color::from_rgb8(0, 128, 0)),
            "blue" => Some(Color::from_rgb8(0, 0, 255)),
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
            _ => None,
        }
    }
}

/// CSS parse error.
#[derive(Debug, Clone)]
pub enum CssParseError {
    InvalidSelector,
    InvalidDeclaration,
    InvalidValue,
    UnexpectedToken,
}

impl std::fmt::Display for CssParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CssParseError::InvalidSelector => write!(f, "Invalid CSS selector"),
            CssParseError::InvalidDeclaration => write!(f, "Invalid CSS declaration"),
            CssParseError::InvalidValue => write!(f, "Invalid CSS value"),
            CssParseError::UnexpectedToken => write!(f, "Unexpected token"),
        }
    }
}

impl std::error::Error for CssParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let css = ".button { background-color: red; padding: 10px; }";
        let stylesheet = CssParser::parse_stylesheet(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_hex_color() {
        let css = ".test { color: #ff0000; }";
        let stylesheet = CssParser::parse_stylesheet(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_inline_style() {
        let style = "color: blue; font-size: 16px;";
        let declarations = CssParser::parse_inline_style(style);

        assert!(declarations.contains_key(&StyleProperty::Color));
        assert!(declarations.contains_key(&StyleProperty::FontSize));
    }
}
