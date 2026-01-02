//! # OpenKit
//!
//! A cross-platform CSS-styled UI framework for Rust.
//!
//! OpenKit provides a consistent, beautiful UI experience across Windows, macOS, Linux,
//! and FreeBSD with CSS-powered styling and a Tailwind-inspired design system.
//!
//! ## Supported Platforms
//!
//! - **Windows**: Windows 10 and later
//! - **macOS**: macOS 10.15 (Catalina) and later
//! - **Linux**: X11 and Wayland (GNOME, KDE, Sway, Hyprland, etc.)
//! - **FreeBSD**: X11 with common desktop environments
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use openkit::prelude::*;
//!
//! App::new()
//!     .title("My App")
//!     .theme(Theme::Auto)
//!     .run(|| {
//!         col![16;
//!             label!("Hello, OpenKit!"),
//!             button!("Click me", { println!("Clicked!"); }),
//!         ]
//!     });
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
    pub use crate::widget::avatar::{Avatar, AvatarSize, AvatarShape};
    pub use crate::widget::bar::{Bar, BarPosition, BarVariant};
    pub use crate::widget::button::{Button, ButtonVariant};
    pub use crate::widget::card::{Card, CardVariant};
    pub use crate::widget::checkbox::Checkbox;
    pub use crate::widget::clock::{Clock, ClockFormat, DateFormat};
    pub use crate::widget::container::{Column, Row};
    pub use crate::widget::context_menu::{ContextMenu, MenuItem};
    pub use crate::widget::desktop::{Desktop, DesktopIcon, Wallpaper, WallpaperMode, GradientDirection};
    pub use crate::widget::dropdown::{Dropdown, DropdownOption};
    pub use crate::widget::icon_button::{IconButton, IconButtonSize, IconButtonVariant};
    pub use crate::widget::label::Label;
    pub use crate::widget::list_view::{ListView, ListItem, SelectionMode};
    pub use crate::widget::notification::{Notification, NotificationUrgency};
    pub use crate::widget::password_field::PasswordField;
    pub use crate::widget::progress::{Progress, ProgressVariant, ProgressSize};
    pub use crate::widget::scroll_view::{ScrollView, ScrollBarVisibility};
    pub use crate::widget::separator::{Separator, SeparatorOrientation};
    pub use crate::widget::slider::{Slider, SliderOrientation};
    pub use crate::widget::spacer::Spacer;
    pub use crate::widget::spinner::{Spinner, SpinnerSize};
    pub use crate::widget::switch::{ToggleSwitch, ToggleSwitchSize};
    pub use crate::widget::system_tray::{SystemTray, TrayIcon};
    pub use crate::widget::tabs::{Tabs, Tab, TabPosition, TabVariant};
    pub use crate::widget::textfield::TextField;
    pub use crate::widget::tooltip::{Tooltip, TooltipPosition};
    pub use crate::widget::window::{Window, WindowVariant, WindowControlsStyle};
    pub use crate::widget::workspace::{WorkspaceSwitcher, WorkspaceItem};

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

    // Platform utilities
    pub use crate::platform::{init as platform_init, platform_name, is_desktop};
}

/// Re-export of the error types
pub use crate::app::AppError;
