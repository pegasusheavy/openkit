//! Procedural macros for the OpenKit UI framework.
//!
//! This crate provides derive macros and attribute macros for building
//! OpenKit applications with less boilerplate.
//!
//! # Macros
//!
//! - [`Widget`] - Derive macro for implementing the Widget trait
//! - [`Component`] - Derive macro for creating Angular-like components
//! - [`Styleable`] - Derive macro for CSS styling support
//! - [`#[component]`] - Attribute macro for component definitions
//! - [`#[prop]`] - Attribute macro for component properties
//! - [`#[state]`] - Attribute macro for component state
//! - [`#[event]`] - Attribute macro for event emitters
//!
//! # Examples
//!
//! ```rust,ignore
//! use openkit::prelude::*;
//! use openkit_macros::{Widget, Component, Styleable};
//!
//! #[derive(Widget, Styleable)]
//! struct MyWidget {
//!     #[base]
//!     base: WidgetBase,
//!     label: String,
//! }
//!
//! #[derive(Component)]
//! #[component(selector = "my-counter")]
//! struct CounterComponent {
//!     #[state]
//!     count: i32,
//!
//!     #[prop(default = 1)]
//!     step: i32,
//!
//!     #[event]
//!     on_change: EventEmitter<i32>,
//! }
//! ```

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use proc_macro_error::proc_macro_error;

mod widget;
mod component;
mod styleable;

/// Derive macro for implementing the Widget trait.
///
/// This macro generates the boilerplate implementation of the `Widget` trait
/// for a struct that contains a `WidgetBase` field.
///
/// # Requirements
///
/// - The struct must have a field marked with `#[base]` of type `WidgetBase`
/// - The struct must have a `type_name` attribute specifying the widget type
///
/// # Example
///
/// ```rust,ignore
/// use openkit_macros::Widget;
///
/// #[derive(Widget)]
/// #[widget(type_name = "my-widget")]
/// struct MyWidget {
///     #[base]
///     base: WidgetBase,
///
///     label: String,
///     value: i32,
/// }
///
/// // The macro generates:
/// // - id() -> WidgetId
/// // - type_name() -> &'static str
/// // - element_id() -> Option<&str>
/// // - classes() -> &ClassList
/// // - state() -> WidgetState
/// // - bounds() -> Rect
/// // - set_bounds(&mut self, Rect)
/// ```
#[proc_macro_derive(Widget, attributes(widget, base))]
#[proc_macro_error]
pub fn derive_widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    widget::derive_widget_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derive macro for creating Angular-like components.
///
/// This macro generates the component infrastructure including state management,
/// property binding, event emission, and lifecycle hooks.
///
/// # Attributes
///
/// - `#[component(selector = "...")]` - Set the component selector
/// - `#[state]` - Mark a field as reactive state
/// - `#[prop]` - Mark a field as a component property
/// - `#[prop(default = value)]` - Property with default value
/// - `#[event]` - Mark a field as an event emitter
///
/// # Example
///
/// ```rust,ignore
/// use openkit_macros::Component;
///
/// #[derive(Component)]
/// #[component(selector = "counter")]
/// struct Counter {
///     #[state]
///     count: i32,
///
///     #[prop(default = 1)]
///     step: i32,
///
///     #[event]
///     on_change: EventEmitter<i32>,
/// }
///
/// impl Counter {
///     fn increment(&mut self) {
///         self.count += self.step;
///         self.on_change.emit(self.count);
///     }
/// }
/// ```
#[proc_macro_derive(Component, attributes(component, state, prop, event))]
#[proc_macro_error]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    component::derive_component_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derive macro for CSS styling support.
///
/// This macro generates the `class()` and `id()` builder methods
/// for adding CSS classes and element IDs to widgets.
///
/// # Requirements
///
/// - The struct must have a field marked with `#[base]` of type `WidgetBase`
///
/// # Example
///
/// ```rust,ignore
/// use openkit_macros::Styleable;
///
/// #[derive(Styleable)]
/// struct MyWidget {
///     #[base]
///     base: WidgetBase,
///     // other fields...
/// }
///
/// // The macro generates:
/// impl MyWidget {
///     pub fn class(mut self, class: &str) -> Self { ... }
///     pub fn id(mut self, id: &str) -> Self { ... }
///     pub fn classes(&self) -> &ClassList { ... }
/// }
/// ```
#[proc_macro_derive(Styleable, attributes(base))]
#[proc_macro_error]
pub fn derive_styleable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    styleable::derive_styleable_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Attribute macro for defining components with inline render function.
///
/// This is an alternative to the derive macro that allows defining
/// the component and its render function in one place.
///
/// # Example
///
/// ```rust,ignore
/// use openkit_macros::component;
///
/// #[component(selector = "greeting")]
/// fn Greeting(
///     #[prop] name: String,
///     #[prop(default = "Hello")] greeting: String,
/// ) -> impl Widget {
///     label!(format!("{}, {}!", greeting, name))
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as component::ComponentAttrArgs);
    let item = parse_macro_input!(item as syn::ItemFn);
    component::component_attribute_impl(attr, item)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Attribute macro for marking component properties.
///
/// Properties are values passed from parent to child components.
///
/// # Attributes
///
/// - `#[prop]` - Required property
/// - `#[prop(default = value)]` - Property with default value
/// - `#[prop(optional)]` - Optional property (wrapped in Option)
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Component)]
/// struct MyComponent {
///     #[prop]
///     required_value: String,
///
///     #[prop(default = 42)]
///     with_default: i32,
///
///     #[prop(optional)]
///     maybe_value: Option<String>,
/// }
/// ```
#[proc_macro_attribute]
pub fn prop(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This is handled by the Component derive macro
    // The attribute just marks the field
    item
}

/// Attribute macro for marking component state.
///
/// State fields are reactive - changes trigger re-renders.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Component)]
/// struct Counter {
///     #[state]
///     count: i32,  // Changes to count trigger re-render
/// }
/// ```
#[proc_macro_attribute]
pub fn state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This is handled by the Component derive macro
    item
}

/// Attribute macro for marking event emitters.
///
/// Events allow components to communicate with their parents.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Component)]
/// struct Button {
///     #[event]
///     on_click: EventEmitter<()>,
///
///     #[event]
///     on_value_change: EventEmitter<String>,
/// }
/// ```
#[proc_macro_attribute]
pub fn event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This is handled by the Component derive macro
    item
}

/// Helper function to find a field with a specific attribute.
#[allow(dead_code)]
fn find_field_with_attr<'a>(fields: &'a syn::Fields, attr_name: &str) -> Option<&'a syn::Field> {
    match fields {
        syn::Fields::Named(named) => {
            named.named.iter().find(|f| {
                f.attrs.iter().any(|a| a.path().is_ident(attr_name))
            })
        }
        _ => None,
    }
}

/// Helper function to get all fields with a specific attribute.
#[allow(dead_code)]
fn get_fields_with_attr<'a>(fields: &'a syn::Fields, attr_name: &str) -> Vec<&'a syn::Field> {
    match fields {
        syn::Fields::Named(named) => {
            named.named.iter().filter(|f| {
                f.attrs.iter().any(|a| a.path().is_ident(attr_name))
            }).collect()
        }
        _ => Vec::new(),
    }
}
