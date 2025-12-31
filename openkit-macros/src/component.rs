//! Component derive macro implementation.

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{DeriveInput, Data, Fields, Error, Ident, ItemFn, Meta, Lit, Token, LitStr};
use syn::parse::{Parse, ParseStream};
use darling::{FromDeriveInput, FromField, FromMeta};

/// Parsed component attribute arguments.
#[derive(Debug, Default)]
pub struct ComponentAttrArgs {
    pub selector: Option<String>,
}

impl Parse for ComponentAttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = ComponentAttrArgs::default();
        
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            
            if ident == "selector" {
                let lit: LitStr = input.parse()?;
                args.selector = Some(lit.value());
            }
            
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(args)
    }
}

/// Attributes for the Component derive macro.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(component))]
struct ComponentAttrs {
    ident: Ident,
    
    /// The component selector (e.g., "my-counter")
    #[darling(default)]
    selector: Option<String>,
}

/// Parsed prop attribute.
#[derive(Debug, Default)]
struct PropAttr {
    default: Option<syn::Expr>,
    optional: bool,
}

pub fn derive_component_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;
    
    // Get the selector
    let selector = get_selector(&input)?;
    
    // Get fields
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Err(Error::new_spanned(&input, "Component can only be derived for structs")),
    };
    
    let named = match fields {
        Fields::Named(named) => named,
        _ => return Err(Error::new_spanned(&input, "Component requires named fields")),
    };
    
    // Categorize fields
    let mut state_fields = Vec::new();
    let mut prop_fields = Vec::new();
    let mut event_fields = Vec::new();
    
    for field in &named.named {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        
        let has_state = field.attrs.iter().any(|a| a.path().is_ident("state"));
        let has_prop = field.attrs.iter().any(|a| a.path().is_ident("prop"));
        let has_event = field.attrs.iter().any(|a| a.path().is_ident("event"));
        
        if has_state {
            state_fields.push((ident, ty));
        } else if has_prop {
            let prop_attr = parse_prop_attr(field)?;
            prop_fields.push((ident, ty, prop_attr));
        } else if has_event {
            event_fields.push((ident, ty));
        }
    }
    
    // Generate state wrapper struct
    let state_struct_name = format_ident!("{}State", name);
    let state_struct = generate_state_struct(&state_struct_name, &state_fields);
    
    // Generate props struct
    let props_struct_name = format_ident!("{}Props", name);
    let props_struct = generate_props_struct(&props_struct_name, &prop_fields);
    
    // Generate events struct
    let events_struct_name = format_ident!("{}Events", name);
    let events_struct = generate_events_struct(&events_struct_name, &event_fields);
    
    // Generate the Component implementation
    let state_field_inits: Vec<_> = state_fields.iter().map(|(ident, _)| {
        quote! { #ident: crate::component::State::new(Default::default()) }
    }).collect();
    
    let prop_field_inits: Vec<_> = prop_fields.iter().map(|(ident, _, attr)| {
        if let Some(ref default) = attr.default {
            quote! { #ident: #default }
        } else if attr.optional {
            quote! { #ident: None }
        } else {
            quote! { #ident: Default::default() }
        }
    }).collect();
    
    let event_field_inits: Vec<_> = event_fields.iter().map(|(ident, _)| {
        quote! { #ident: crate::component::EventEmitter::new() }
    }).collect();
    
    let all_field_inits: Vec<_> = state_field_inits.iter()
        .chain(prop_field_inits.iter())
        .chain(event_field_inits.iter())
        .cloned()
        .collect();
    
    let expanded = quote! {
        #state_struct
        #props_struct
        #events_struct
        
        impl #name {
            /// Create a new component with default values.
            pub fn new() -> Self {
                Self {
                    #(#all_field_inits,)*
                }
            }
            
            /// Get the component selector.
            pub fn selector() -> &'static str {
                #selector
            }
        }
        
        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
    
    Ok(expanded)
}

fn get_selector(input: &DeriveInput) -> Result<String, Error> {
    for attr in &input.attrs {
        if attr.path().is_ident("component") {
            if let Meta::List(list) = &attr.meta {
                let args: ComponentAttrArgs = syn::parse2(list.tokens.clone())?;
                if let Some(selector) = args.selector {
                    return Ok(selector);
                }
            }
        }
    }
    
    // Default to kebab-case of struct name
    Ok(to_kebab_case(&input.ident.to_string()))
}

fn parse_prop_attr(field: &syn::Field) -> Result<PropAttr, Error> {
    let mut attr = PropAttr::default();
    
    for a in &field.attrs {
        if a.path().is_ident("prop") {
            if let Meta::List(list) = &a.meta {
                // Parse prop(default = value) or prop(optional)
                let content = list.tokens.to_string();
                if content.contains("optional") {
                    attr.optional = true;
                }
                // Note: For full default parsing, would need more complex logic
            }
        }
    }
    
    Ok(attr)
}

fn generate_state_struct(name: &Ident, fields: &[(&Ident, &syn::Type)]) -> TokenStream {
    if fields.is_empty() {
        return quote! {
            /// Empty state for this component.
            pub struct #name;
        };
    }
    
    let field_defs: Vec<_> = fields.iter().map(|(ident, ty)| {
        quote! { pub #ident: crate::component::State<#ty> }
    }).collect();
    
    quote! {
        /// State container for the component.
        #[derive(Debug)]
        pub struct #name {
            #(#field_defs,)*
        }
    }
}

fn generate_props_struct(name: &Ident, fields: &[(&Ident, &syn::Type, PropAttr)]) -> TokenStream {
    if fields.is_empty() {
        return quote! {
            /// Empty props for this component.
            #[derive(Debug, Default, Clone)]
            pub struct #name;
        };
    }
    
    let field_defs: Vec<_> = fields.iter().map(|(ident, ty, _)| {
        quote! { pub #ident: #ty }
    }).collect();
    
    quote! {
        /// Properties for the component.
        #[derive(Debug, Clone)]
        pub struct #name {
            #(#field_defs,)*
        }
    }
}

fn generate_events_struct(name: &Ident, fields: &[(&Ident, &syn::Type)]) -> TokenStream {
    if fields.is_empty() {
        return quote! {
            /// Empty events for this component.
            pub struct #name;
        };
    }
    
    let field_defs: Vec<_> = fields.iter().map(|(ident, ty)| {
        quote! { pub #ident: #ty }
    }).collect();
    
    quote! {
        /// Event emitters for the component.
        pub struct #name {
            #(#field_defs,)*
        }
    }
}

pub fn component_attribute_impl(attr: ComponentAttrArgs, item: ItemFn) -> Result<TokenStream, Error> {
    let fn_name = &item.sig.ident;
    let component_name = format_ident!("{}Component", fn_name);
    
    // Get selector from attributes
    let selector = attr.selector.unwrap_or_else(|| to_kebab_case(&fn_name.to_string()));
    
    let expanded = quote! {
        #item
        
        /// Generated component wrapper.
        pub struct #component_name;
        
        impl #component_name {
            pub fn selector() -> &'static str {
                #selector
            }
        }
    };
    
    Ok(expanded)
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
