# OpenKit

A cross-platform CSS-styled UI framework for Rust.

OpenKit provides a consistent, beautiful desktop application experience across Windows, macOS, Linux, and FreeBSD with CSS-powered styling and a Tailwind-inspired design system.

## Supported Platforms

| Platform | Backend | GPU API | Status |
|----------|---------|---------|--------|
| Windows 10+ | Win32 | Direct3D 12 / Vulkan | ✅ Fully Supported |
| macOS 10.15+ | Cocoa | Metal | ✅ Fully Supported |
| Linux | X11 / Wayland | Vulkan / OpenGL | ✅ Fully Supported |
| FreeBSD | X11 | Vulkan / OpenGL | ✅ Fully Supported |

## Features

- **Cross-Platform**: Native look and feel on Windows, macOS, Linux, and FreeBSD
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

## Rendering Model

OpenKit renders **its own chrome** - all window decorations, widgets, and UI elements are rendered by OpenKit itself using GPU acceleration. This ensures:

- **Pixel-perfect consistency** across all platforms
- **Full CSS control** over every visual element
- **No platform UI dependencies** - just winit for windowing and wgpu for rendering

```
┌─────────────────────────────────────────────────────────┐
│                    Your Application                      │
├─────────────────────────────────────────────────────────┤
│                     OpenKit Widgets                      │
│  (Button, Label, TextField, Window, Desktop, etc.)      │
├─────────────────────────────────────────────────────────┤
│                    OpenKit Renderer                      │
│           wgpu (GPU) │ tiny-skia (CPU fallback)         │
├─────────────────────────────────────────────────────────┤
│                        winit                             │
│              (Platform window creation)                  │
├────────────┬────────────┬─────────────┬─────────────────┤
│  Windows   │   macOS    │    Linux    │    FreeBSD      │
│  (Win32)   │  (Cocoa)   │ (X11/Wayland)│     (X11)      │
└────────────┴────────────┴─────────────┴─────────────────┘
```

### Platform Detection

Each platform has detection utilities (no external UI libraries required):

- **Windows**: Version detection, theme preference
- **macOS**: Version detection, theme preference
- **Linux**: Display server (X11/Wayland), desktop environment (GNOME, KDE, etc.)
- **FreeBSD**: Desktop environment detection

## Feature Flags

```toml
[dependencies]
openkit = { version = "0.1", features = ["gpu", "macros"] }
```

| Feature | Description | Default |
|---------|-------------|---------|
| `gpu` | GPU-accelerated rendering via wgpu | ✅ |
| `macros` | Declarative UI macros (`col!`, `button!`, etc.) | ✅ |
| `wayland` | Wayland support (Linux) | ✅ |
| `x11` | X11 support (Linux/FreeBSD) | ✅ |
| `hdr` | HDR support (when available) | ❌ |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
