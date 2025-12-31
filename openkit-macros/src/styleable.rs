//! Styleable derive macro implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, Fields, Error};

pub fn derive_styleable_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;
    
    // Find the base field
    let base_field = find_base_field(&input)?;
    let base_ident = base_field.ident.as_ref()
        .ok_or_else(|| Error::new_spanned(&input, "Base field must be named"))?;
    
    let expanded = quote! {
        impl #name {
            /// Add a CSS class to this widget.
            ///
            /// # Example
            ///
            /// ```rust,ignore
            /// let widget = MyWidget::new()
            ///     .class("primary")
            ///     .class("large");
            /// ```
            pub fn class(mut self, class: &str) -> Self {
                self.#base_ident.classes.add(class);
                self
            }
            
            /// Add multiple CSS classes to this widget.
            ///
            /// # Example
            ///
            /// ```rust,ignore
            /// let widget = MyWidget::new()
            ///     .classes(&["primary", "large", "rounded"]);
            /// ```
            pub fn classes(mut self, classes: &[&str]) -> Self {
                for class in classes {
                    self.#base_ident.classes.add(*class);
                }
                self
            }
            
            /// Set the element ID for this widget.
            ///
            /// # Example
            ///
            /// ```rust,ignore
            /// let widget = MyWidget::new()
            ///     .id("my-unique-widget");
            /// ```
            pub fn id(mut self, id: &str) -> Self {
                self.#base_ident.element_id = Some(id.to_string());
                self
            }
            
            /// Remove a CSS class from this widget.
            pub fn remove_class(&mut self, class: &str) {
                self.#base_ident.classes.remove(class);
            }
            
            /// Toggle a CSS class on this widget.
            pub fn toggle_class(&mut self, class: &str) {
                self.#base_ident.classes.toggle(class);
            }
            
            /// Check if this widget has a specific CSS class.
            pub fn has_class(&self, class: &str) -> bool {
                self.#base_ident.classes.contains(class)
            }
            
            /// Get all CSS classes on this widget.
            pub fn get_classes(&self) -> &crate::css::ClassList {
                &self.#base_ident.classes
            }
            
            /// Get the element ID of this widget.
            pub fn get_id(&self) -> Option<&str> {
                self.#base_ident.element_id.as_deref()
            }
        }
    };
    
    Ok(expanded)
}

fn find_base_field(input: &DeriveInput) -> Result<&syn::Field, Error> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Err(Error::new_spanned(input, "Styleable can only be derived for structs")),
    };
    
    let named = match fields {
        Fields::Named(named) => named,
        _ => return Err(Error::new_spanned(input, "Styleable requires named fields")),
    };
    
    // Find field with #[base] attribute
    for field in &named.named {
        if field.attrs.iter().any(|a| a.path().is_ident("base")) {
            return Ok(field);
        }
    }
    
    // If no #[base] attribute, look for a field named "base"
    for field in &named.named {
        if let Some(ident) = &field.ident {
            if ident == "base" {
                return Ok(field);
            }
        }
    }
    
    Err(Error::new_spanned(
        input,
        "Styleable requires a field marked with #[base] or named 'base'"
    ))
}
