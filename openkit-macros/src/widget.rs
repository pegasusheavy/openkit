//! Widget derive macro implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, Fields, Error, Ident};
use darling::FromDeriveInput;

/// Attributes for the Widget derive macro.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(widget))]
struct WidgetAttrs {
    ident: Ident,
    
    /// The CSS type name for the widget (e.g., "button", "label")
    #[darling(default)]
    type_name: Option<String>,
}

pub fn derive_widget_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;
    
    // Parse attributes
    let attrs = WidgetAttrs::from_derive_input(&input)
        .map_err(|e| Error::new_spanned(&input, e.to_string()))?;
    
    // Get the type name (default to lowercase struct name with hyphens)
    let type_name = attrs.type_name.unwrap_or_else(|| {
        to_kebab_case(&name.to_string())
    });
    
    // Find the base field
    let base_field = find_base_field(&input)?;
    let base_ident = base_field.ident.as_ref()
        .ok_or_else(|| Error::new_spanned(&input, "Base field must be named"))?;
    
    // Generate the implementation
    let expanded = quote! {
        impl #name {
            /// Get the widget's unique ID.
            pub fn widget_id(&self) -> crate::widget::WidgetId {
                self.#base_ident.id
            }
            
            /// Get the widget's CSS type name.
            pub fn widget_type_name(&self) -> &'static str {
                #type_name
            }
            
            /// Get the widget's element ID.
            pub fn widget_element_id(&self) -> Option<&str> {
                self.#base_ident.element_id.as_deref()
            }
            
            /// Get the widget's CSS classes.
            pub fn widget_classes(&self) -> &crate::css::ClassList {
                &self.#base_ident.classes
            }
            
            /// Get the widget's current state.
            pub fn widget_state(&self) -> crate::css::WidgetState {
                self.#base_ident.state
            }
            
            /// Get the widget's bounds.
            pub fn widget_bounds(&self) -> crate::geometry::Rect {
                self.#base_ident.bounds
            }
            
            /// Set the widget's bounds.
            pub fn set_widget_bounds(&mut self, bounds: crate::geometry::Rect) {
                self.#base_ident.bounds = bounds;
            }
        }
        
        // Partial Widget trait implementation
        // Users still need to implement layout, paint, and handle_event
        impl crate::widget::WidgetBase for #name {
            fn id(&self) -> crate::widget::WidgetId {
                self.#base_ident.id
            }
            
            fn type_name(&self) -> &'static str {
                #type_name
            }
            
            fn element_id(&self) -> Option<&str> {
                self.#base_ident.element_id.as_deref()
            }
            
            fn classes(&self) -> &crate::css::ClassList {
                &self.#base_ident.classes
            }
            
            fn state(&self) -> crate::css::WidgetState {
                self.#base_ident.state
            }
            
            fn bounds(&self) -> crate::geometry::Rect {
                self.#base_ident.bounds
            }
            
            fn set_bounds(&mut self, bounds: crate::geometry::Rect) {
                self.#base_ident.bounds = bounds;
            }
        }
    };
    
    Ok(expanded)
}

fn find_base_field(input: &DeriveInput) -> Result<&syn::Field, Error> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Err(Error::new_spanned(input, "Widget can only be derived for structs")),
    };
    
    let named = match fields {
        Fields::Named(named) => named,
        _ => return Err(Error::new_spanned(input, "Widget requires named fields")),
    };
    
    // Find field with #[base] attribute
    for field in &named.named {
        if field.attrs.iter().any(|a| a.path().is_ident("base")) {
            return Ok(field);
        }
    }
    
    // If no #[base] attribute, look for a field named "base" of type WidgetBase
    for field in &named.named {
        if let Some(ident) = &field.ident {
            if ident == "base" {
                return Ok(field);
            }
        }
    }
    
    Err(Error::new_spanned(
        input,
        "Widget requires a field marked with #[base] or named 'base'"
    ))
}

/// Convert PascalCase to kebab-case.
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("Button"), "button");
        assert_eq!(to_kebab_case("TextField"), "text-field");
        assert_eq!(to_kebab_case("MyCustomWidget"), "my-custom-widget");
    }
}
