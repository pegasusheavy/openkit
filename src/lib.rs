//! # OpenKit
//!
//! A cross-platform CSS-styled UI framework for Rust.
//!
//! OpenKit provides a consistent, beautiful UI experience across Windows, macOS, and Linux
//! with CSS-powered styling and a Tailwind-inspired design system.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use openkit::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .title("My App")
//!         .theme(Theme::Auto)
//!         .run(|| {
//!             col![16;
//!                 label!("Hello, OpenKit!"),
//!                 button!("Click me", { println!("Clicked!"); }),
//!             ]
//!         });
//! }
//! ```
//!
//! ## Declarative Macros
//!
//! OpenKit provides ergonomic macros for building UIs:
//!
//! ```rust
//! use openkit::prelude::*;
//!
//! // Layout macros
//! let ui = col![16;                          // Column with 16px gap
//!     label!("Welcome!"),
//!     row![8;                                // Row with 8px gap
//!         button!("OK", { /* handler */ }),
//!         button!("Cancel", Secondary),
//!     ],
//! ];
//!
//! // Widget macros
//! let btn = button!("Save", { println!("Saved!"); });
//! let cb = checkbox!("Enable", |checked| println!("{}", checked));
//! let tf = textfield!("Enter name...", |val| println!("{}", val));
//!
//! // Utility macros
//! let classes = class!["btn", "primary", "large"];
//! let styles = style! { padding: 16, color: "blue" };
//! ```

pub mod app;
#[macro_use]
pub mod component;
pub mod css;
pub mod event;
pub mod geometry;
pub mod layout;
#[macro_use]
pub mod macros;
pub mod platform;
pub mod render;
pub mod theme;
pub mod widget;

// Re-export proc macros when the feature is enabled
#[cfg(feature = "macros")]
pub use openkit_macros::{Widget, Component, Styleable, component, prop, state, event};

pub mod prelude {
    //! Convenient re-exports for common types.

    // Core types
    pub use crate::app::App;
    pub use crate::css::{StyleManager, StyleBuilder, CssLoadError};
    pub use crate::event::{Event, MouseButton, MouseEvent, KeyEvent, Key, Modifiers};
    pub use crate::geometry::{Point, Size, Rect, Color, BorderRadius, EdgeInsets};
    pub use crate::layout::{Layout, Alignment, Padding};
    pub use crate::theme::Theme;

    // Widget types
    pub use crate::widget::{Widget, WidgetId};
    pub use crate::widget::button::{Button, ButtonVariant};
    pub use crate::widget::label::Label;
    pub use crate::widget::textfield::TextField;
    pub use crate::widget::checkbox::Checkbox;
    pub use crate::widget::container::{Column, Row};

    // Component system (Angular-like)
    pub use crate::component::{
        // Core types
        State, EventEmitter, Model, Binding,
        ComponentBuilder, BuiltComponent, ComponentContext,
        component,
        // Lifecycle
        Lifecycle, Changes,
        // Directives
        If, For, Switch,
        // Pipes
        Pipe, UppercasePipe, LowercasePipe, CurrencyPipe,
    };

    // Re-export all declarative macros
    pub use crate::{
        // Widget macros
        view, col, row, class, style,
        button, checkbox, textfield, label,
        when, match_widget, for_each, spacer, dbg_widget,
        // Angular-like macros
        define_component, ng_if, ng_for, ng_switch,
        bind, on, model,
    };
    
    // Re-export proc macros when enabled
    #[cfg(feature = "macros")]
    pub use openkit_macros::{Widget, Component, Styleable, component, prop, state, event};
}

/// Re-export of the error types
pub use crate::app::AppError;
