# OpenKit

A cross-platform CSS-styled UI framework for Rust.

OpenKit provides a consistent, beautiful desktop application experience across Windows, macOS, and Linux with CSS-powered styling and a Tailwind-inspired design system.

## Features

- **Cross-Platform**: Native look and feel on Windows, macOS, and Linux
- **CSS Styling**: Style your UI with familiar CSS syntax
- **GPU Accelerated**: High-performance rendering with wgpu (with CPU fallback)
- **Rich Widget Set**: 30+ widgets for building complete desktop applications
- **Tailwind-Inspired**: Built-in design system with dark/light themes
- **Declarative Macros**: Ergonomic UI building with `col!`, `row!`, `button!`, etc.
- **Angular-Like Components**: Component system with props, state, and lifecycle hooks

## Quick Start

Add OpenKit to your `Cargo.toml`:

```toml
[dependencies]
openkit = "0.1"
```

Create a simple application:

```rust
use openkit::prelude::*;

fn main() {
    App::new()
        .title("My App")
        .theme(Theme::Auto)
        .run(|| {
            col![16;
                label!("Hello, OpenKit!"),
                button!("Click me", { println!("Clicked!"); }),
            ]
        });
}
```

## Widgets

OpenKit includes a comprehensive widget set:

### Layout
- `Column`, `Row` - Flex containers
- `Card` - Content container with styling
- `ScrollView` - Scrollable container
- `Tabs` - Tabbed interface
- `Spacer`, `Separator` - Layout helpers

### Input
- `Button`, `IconButton` - Clickable buttons
- `TextField`, `PasswordField` - Text input
- `Checkbox`, `ToggleSwitch` - Boolean input
- `Dropdown`, `Slider` - Selection controls

### Display
- `Label` - Text display
- `Avatar` - User profile images
- `Progress`, `Spinner` - Loading indicators
- `Notification`, `Tooltip` - Information display

### Desktop Shell
- `Window` - Decorated windows with OS-native controls
- `Bar` - Taskbar/panel container
- `Desktop` - Desktop with wallpaper and icons
- `ContextMenu` - Right-click menus
- `SystemTray`, `Clock` - System indicators
- `WorkspaceSwitcher` - Virtual desktop switching

## Styling

Style widgets with CSS:

```rust
use openkit::prelude::*;

fn main() {
    App::new()
        .load_css(r#"
            .my-button {
                background: linear-gradient(to-right, #667eea, #764ba2);
                border-radius: 8px;
                padding: 12px 24px;
            }
            .my-button:hover {
                transform: scale(1.05);
            }
        "#)
        .run(|| {
            button!("Styled Button").class("my-button")
        });
}
```

## Theming

OpenKit includes a Tailwind-inspired theme system:

```rust
use openkit::prelude::*;

fn main() {
    App::new()
        .theme(Theme::Dark)  // or Theme::Light, Theme::Auto
        .run(|| {
            // Widgets automatically use theme colors
            col![16;
                label!("Themed UI"),
                button!("Primary", Primary),
                button!("Secondary", Secondary),
                button!("Destructive", Destructive),
            ]
        });
}
```

## Desktop Wallpapers

Create desktop environments with customizable backgrounds:

```rust
use openkit::prelude::*;

let desktop = Desktop::new()
    .background(Wallpaper::image("/path/to/wallpaper.jpg")
        .with_mode(WallpaperMode::Fill))
    .icon(DesktopIcon::new("home", "Home", "🏠").at(0, 0))
    .icon(DesktopIcon::new("files", "Files", "📁").at(0, 1));
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
