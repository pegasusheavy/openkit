//! Declarative macros for ergonomic UI building.
//!
//! # Overview
//!
//! OpenKit provides several macros to make building UIs more pleasant:
//!
//! - [`view!`] - Declarative widget tree syntax (like JSX/SwiftUI)
//! - [`style!`] - Inline CSS-like styling
//! - [`class!`] - Multiple class names
//! - [`col!`] / [`row!`] - Quick layout containers
//! - [`on_click!`] - Event handler shorthand
//!
//! # Examples
//!
//! ```rust,ignore
//! use openkit::prelude::*;
//!
//! // Declarative view syntax
//! let ui = view! {
//!     Column [gap: 16, padding: 24] {
//!         Label("Welcome to OpenKit!")
//!         Row [gap: 8] {
//!             Button("OK") { println!("OK clicked!") }
//!             Button("Cancel").variant(Secondary)
//!         }
//!     }
//! };
//! ```

/// Declarative view macro for building widget trees.
///
/// # Syntax
///
/// ```text
/// view! {
///     WidgetType [prop: value, ...] {
///         children...
///     }
/// }
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let ui = view! {
///     Column [gap: 16, padding: 24] {
///         Label("Hello, World!")
///         Button("Click me") {
///             println!("Clicked!");
///         }
///     }
/// };
/// ```
#[macro_export]
macro_rules! view {
    // Column with properties and children
    (Column [$($prop:ident : $val:expr),* $(,)?] { $($child:tt)* }) => {{
        let mut col = $crate::widget::container::Column::new();
        $(
            col = $crate::__apply_prop!(col, $prop, $val);
        )*
        $(
            col = col.child($crate::view!($child));
        )*
        col
    }};

    // Column with just children
    (Column { $($child:tt)* }) => {{
        let mut col = $crate::widget::container::Column::new();
        $(
            col = col.child($crate::view!($child));
        )*
        col
    }};

    // Row with properties and children
    (Row [$($prop:ident : $val:expr),* $(,)?] { $($child:tt)* }) => {{
        let mut row = $crate::widget::container::Row::new();
        $(
            row = $crate::__apply_prop!(row, $prop, $val);
        )*
        $(
            row = row.child($crate::view!($child));
        )*
        row
    }};

    // Row with just children
    (Row { $($child:tt)* }) => {{
        let mut row = $crate::widget::container::Row::new();
        $(
            row = row.child($crate::view!($child));
        )*
        row
    }};

    // Label with text
    (Label($text:expr)) => {
        $crate::widget::label::Label::new($text)
    };

    // Label with text and class
    (Label($text:expr) . $method:ident ($($arg:expr),*)) => {
        $crate::widget::label::Label::new($text).$method($($arg),*)
    };

    // Button with text and click handler
    (Button($text:expr) { $($body:tt)* }) => {
        $crate::widget::button::Button::new($text)
            .on_click(move || { $($body)* })
    };

    // Button with text only
    (Button($text:expr)) => {
        $crate::widget::button::Button::new($text)
    };

    // Button with method chain
    (Button($text:expr) . $method:ident ($($arg:expr),*) $(. $rest_method:ident ($($rest_arg:expr),*))*) => {
        $crate::widget::button::Button::new($text)
            .$method($($arg),*)
            $(.$rest_method($($rest_arg),*))*
    };

    // Checkbox with no args
    (Checkbox) => {
        $crate::widget::checkbox::Checkbox::new()
    };

    // Checkbox with label
    (Checkbox($label:expr)) => {
        $crate::widget::checkbox::Checkbox::new().label($label)
    };

    // Checkbox with label and handler
    (Checkbox($label:expr) { |$var:ident| $($body:tt)* }) => {
        $crate::widget::checkbox::Checkbox::new()
            .label($label)
            .on_change(move |$var| { $($body)* })
    };

    // TextField
    (TextField) => {
        $crate::widget::textfield::TextField::new()
    };

    // TextField with placeholder
    (TextField($placeholder:expr)) => {
        $crate::widget::textfield::TextField::new().placeholder($placeholder)
    };

    // TextField with placeholder and handler
    (TextField($placeholder:expr) { |$var:ident| $($body:tt)* }) => {
        $crate::widget::textfield::TextField::new()
            .placeholder($placeholder)
            .on_change(move |$var| { $($body)* })
    };

    // Pass through expressions
    ($expr:expr) => {
        $expr
    };
}

/// Internal macro for applying properties to widgets.
#[macro_export]
#[doc(hidden)]
macro_rules! __apply_prop {
    ($widget:expr, gap, $val:expr) => { $widget.gap($val) };
    ($widget:expr, padding, $val:expr) => { $widget.padding($val.into()) };
    ($widget:expr, align, $val:expr) => { $widget.align($val) };
    ($widget:expr, justify, $val:expr) => { $widget.justify($val) };
    ($widget:expr, class, $val:expr) => { $widget.class($val) };
    ($widget:expr, id, $val:expr) => { $widget.id($val) };
    ($widget:expr, $prop:ident, $val:expr) => { $widget.$prop($val) };
}

/// Quick column layout macro.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Simple column with gap
/// let ui = col![16;
///     Label::new("Item 1"),
///     Label::new("Item 2"),
///     Label::new("Item 3"),
/// ];
///
/// // Column without gap
/// let ui = col![
///     Label::new("Item 1"),
///     Label::new("Item 2"),
/// ];
/// ```
#[macro_export]
macro_rules! col {
    // With gap
    ($gap:expr; $($child:expr),* $(,)?) => {{
        $crate::widget::container::Column::new()
            .gap($gap as f32)
            $(.child($child))*
    }};

    // Without gap
    ($($child:expr),* $(,)?) => {{
        $crate::widget::container::Column::new()
            $(.child($child))*
    }};
}

/// Quick row layout macro.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// // Simple row with gap
/// let ui = row![12;
///     Button::new("Save"),
///     Button::new("Cancel"),
/// ];
///
/// // Row without gap
/// let ui = row![
///     Label::new("Name:"),
///     TextField::new(),
/// ];
/// ```
#[macro_export]
macro_rules! row {
    // With gap
    ($gap:expr; $($child:expr),* $(,)?) => {{
        $crate::widget::container::Row::new()
            .gap($gap as f32)
            $(.child($child))*
    }};

    // Without gap
    ($($child:expr),* $(,)?) => {{
        $crate::widget::container::Row::new()
            $(.child($child))*
    }};
}

/// Combine multiple CSS class names.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let classes = class!["btn", "btn-primary", "large"];
/// let button = Button::new("Click").class(&classes);
///
/// // Conditional classes
/// let is_active = true;
/// let classes = class!["btn", if is_active { "active" } else { "" }];
/// ```
#[macro_export]
macro_rules! class {
    ($($class:expr),* $(,)?) => {{
        let classes: Vec<&str> = vec![$($class),*];
        classes.into_iter()
            .filter(|c| !c.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }};
}

/// Inline style macro with CSS-like syntax.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let styles = style! {
///     background_color: "#3b82f6",
///     color: "white",
///     padding: 16,
///     border_radius: 8,
/// };
/// ```
#[macro_export]
macro_rules! style {
    ($($prop:ident : $val:expr),* $(,)?) => {{
        let mut styles = ::std::collections::HashMap::new();
        $(
            styles.insert(
                stringify!($prop).replace("_", "-"),
                format!("{}", $val)
            );
        )*
        styles
    }};
}

/// Create a button with an inline click handler.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let btn = button!("Save", {
///     println!("Saving...");
/// });
///
/// // With variant
/// let btn = button!("Delete", Destructive, {
///     println!("Deleting...");
/// });
/// ```
#[macro_export]
macro_rules! button {
    // Basic button with handler
    ($label:expr, { $($body:tt)* }) => {
        $crate::widget::button::Button::new($label)
            .on_click(move || { $($body)* })
    };

    // Button with variant and handler
    ($label:expr, $variant:ident, { $($body:tt)* }) => {
        $crate::widget::button::Button::new($label)
            .variant($crate::widget::button::ButtonVariant::$variant)
            .on_click(move || { $($body)* })
    };

    // Button without handler
    ($label:expr) => {
        $crate::widget::button::Button::new($label)
    };

    // Button with variant only
    ($label:expr, $variant:ident) => {
        $crate::widget::button::Button::new($label)
            .variant($crate::widget::button::ButtonVariant::$variant)
    };
}

/// Create a labeled checkbox with an inline change handler.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let cb = checkbox!("Enable notifications", |checked| {
///     println!("Notifications: {}", checked);
/// });
///
/// // Pre-checked
/// let cb = checkbox!("Accept terms", true, |checked| {
///     println!("Terms accepted: {}", checked);
/// });
/// ```
#[macro_export]
macro_rules! checkbox {
    // With label and handler
    ($label:expr, |$var:ident| $($body:tt)*) => {
        $crate::widget::checkbox::Checkbox::new()
            .label($label)
            .on_change(move |$var| { $($body)* })
    };

    // With label, initial state, and handler
    ($label:expr, $checked:expr, |$var:ident| $($body:tt)*) => {
        $crate::widget::checkbox::Checkbox::new()
            .label($label)
            .checked($checked)
            .on_change(move |$var| { $($body)* })
    };

    // Just label
    ($label:expr) => {
        $crate::widget::checkbox::Checkbox::new().label($label)
    };
}

/// Create a text field with placeholder and change handler.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let field = textfield!("Enter name...", |value| {
///     println!("Name: {}", value);
/// });
///
/// // With initial value
/// let field = textfield!("Search...", "initial query", |value| {
///     println!("Search: {}", value);
/// });
/// ```
#[macro_export]
macro_rules! textfield {
    // Placeholder and change handler
    ($placeholder:expr, |$var:ident| $($body:tt)*) => {
        $crate::widget::textfield::TextField::new()
            .placeholder($placeholder)
            .on_change(move |$var| { $($body)* })
    };

    // Placeholder, initial value, and change handler
    ($placeholder:expr, $value:expr, |$var:ident| $($body:tt)*) => {
        $crate::widget::textfield::TextField::new()
            .placeholder($placeholder)
            .value($value)
            .on_change(move |$var| { $($body)* })
    };

    // Just placeholder
    ($placeholder:expr) => {
        $crate::widget::textfield::TextField::new().placeholder($placeholder)
    };

    // No args
    () => {
        $crate::widget::textfield::TextField::new()
    };
}

/// Create a label with optional styling.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let lbl = label!("Hello, World!");
/// let lbl = label!("Title", class: "heading");
/// let lbl = label!("Subtitle", class: "subtitle", id: "sub");
/// ```
#[macro_export]
macro_rules! label {
    // Just text
    ($text:expr) => {
        $crate::widget::label::Label::new($text)
    };

    // Text with class
    ($text:expr, class: $class:expr) => {
        $crate::widget::label::Label::new($text).class($class)
    };

    // Text with class and id
    ($text:expr, class: $class:expr, id: $id:expr) => {
        $crate::widget::label::Label::new($text).class($class).id($id)
    };

    // Text with id
    ($text:expr, id: $id:expr) => {
        $crate::widget::label::Label::new($text).id($id)
    };
}

/// Conditional widget rendering.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let show_welcome = true;
///
/// let ui = col![16;
///     when!(show_welcome => Label::new("Welcome!")),
///     Button::new("Continue"),
/// ];
/// ```
#[macro_export]
macro_rules! when {
    ($cond:expr => $widget:expr) => {
        if $cond {
            Some($widget)
        } else {
            None
        }
    };
}

/// Match-based widget selection.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// enum Status { Loading, Error, Ready }
/// let status = Status::Ready;
///
/// let ui = match_widget!(status,
///     Status::Loading => Label::new("Loading..."),
///     Status::Error => Label::new("Error!").class("error"),
///     Status::Ready => Button::new("Start"),
/// );
/// ```
#[macro_export]
macro_rules! match_widget {
    ($val:expr, $($pat:pat => $widget:expr),* $(,)?) => {
        match $val {
            $($pat => Box::new($widget) as Box<dyn $crate::widget::Widget>,)*
        }
    };
}

/// Iterate and create widgets from a collection.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let items = vec!["Apple", "Banana", "Cherry"];
///
/// let ui = col![8;
///     for_each!(items, |item| Label::new(item))
/// ];
/// ```
#[macro_export]
macro_rules! for_each {
    ($iter:expr, |$item:ident| $widget:expr) => {{
        $iter.into_iter().map(|$item| $widget).collect::<Vec<_>>()
    }};

    ($iter:expr, |$idx:ident, $item:ident| $widget:expr) => {{
        $iter.into_iter().enumerate().map(|($idx, $item)| $widget).collect::<Vec<_>>()
    }};
}

/// Create a spacer widget that expands to fill available space.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let ui = row![
///     Label::new("Left"),
///     spacer!(),
///     Label::new("Right"),
/// ];
/// ```
#[macro_export]
macro_rules! spacer {
    () => {
        $crate::widget::label::Label::new("").class("spacer")
    };
    ($size:expr) => {
        $crate::widget::label::Label::new("").class("spacer")
    };
}

/// Debug helper - prints widget tree structure.
///
/// # Examples
///
/// ```rust,ignore
/// use openkit::prelude::*;
///
/// let ui = col![
///     Label::new("Hello"),
///     Button::new("Click"),
/// ];
///
/// dbg_widget!(ui);
/// ```
#[macro_export]
macro_rules! dbg_widget {
    ($widget:expr) => {{
        let widget = $widget;
        eprintln!(
            "[{}:{}] Widget: {}",
            file!(),
            line!(),
            stringify!($widget)
        );
        widget
    }};
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::widget::Widget;

    #[test]
    fn test_class_macro() {
        let classes = class!["btn", "primary"];
        assert_eq!(classes, "btn primary");

        let classes = class!["btn", "", "active"];
        assert_eq!(classes, "btn active");
    }

    #[test]
    fn test_style_macro() {
        let styles = style! {
            background_color: "#fff",
            padding: 16,
        };
        assert_eq!(styles.get("background-color"), Some(&"#fff".to_string()));
        assert_eq!(styles.get("padding"), Some(&"16".to_string()));
    }

    #[test]
    fn test_col_macro() {
        use crate::widget::label::Label;

        let col = col![16;
            Label::new("A"),
            Label::new("B"),
        ];
        assert_eq!(col.children().len(), 2);
    }

    #[test]
    fn test_row_macro() {
        use crate::widget::label::Label;

        let row = row![
            Label::new("X"),
            Label::new("Y"),
        ];
        assert_eq!(row.children().len(), 2);
    }
}
