//! Linux platform detection utilities.
//!
//! OpenKit renders its own chrome - this module only provides detection utilities.
//! No Linux-specific UI libraries are used; all rendering is done via wgpu/tiny-skia.
//! Windowing is handled by winit (supporting both X11 and Wayland).

#![cfg(target_os = "linux")]

use crate::theme::Theme;

/// Display server backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    Unknown,
}

/// Desktop environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Gnome,
    KDE,
    XFCE,
    Cinnamon,
    MATE,
    Pantheon,
    Budgie,
    Deepin,
    Sway,
    Hyprland,
    I3,
    Other(String),
    Unknown,
}

/// Initialize Linux platform.
pub fn init() {
    log::debug!("OpenKit running on Linux");
    log::debug!("Display server: {:?}", detect_display_server());
    log::debug!("Desktop environment: {:?}", detect_desktop_environment());
}

/// Detect the display server (Wayland or X11).
pub fn detect_display_server() -> DisplayServer {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        DisplayServer::Wayland
    } else if std::env::var("DISPLAY").is_ok() {
        DisplayServer::X11
    } else {
        DisplayServer::Unknown
    }
}

/// Check if running on Wayland.
pub fn is_wayland() -> bool {
    detect_display_server() == DisplayServer::Wayland
}

/// Check if running on X11.
pub fn is_x11() -> bool {
    detect_display_server() == DisplayServer::X11
}

/// Detect the desktop environment.
pub fn detect_desktop_environment() -> DesktopEnvironment {
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        let desktop_lower = desktop.to_lowercase();

        if desktop_lower.contains("gnome") {
            DesktopEnvironment::Gnome
        } else if desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
            DesktopEnvironment::KDE
        } else if desktop_lower.contains("xfce") {
            DesktopEnvironment::XFCE
        } else if desktop_lower.contains("cinnamon") {
            DesktopEnvironment::Cinnamon
        } else if desktop_lower.contains("mate") {
            DesktopEnvironment::MATE
        } else if desktop_lower.contains("pantheon") {
            DesktopEnvironment::Pantheon
        } else if desktop_lower.contains("budgie") {
            DesktopEnvironment::Budgie
        } else if desktop_lower.contains("deepin") {
            DesktopEnvironment::Deepin
        } else if desktop_lower.contains("sway") {
            DesktopEnvironment::Sway
        } else if desktop_lower.contains("hyprland") {
            DesktopEnvironment::Hyprland
        } else if desktop_lower.contains("i3") {
            DesktopEnvironment::I3
        } else if !desktop.is_empty() {
            DesktopEnvironment::Other(desktop)
        } else {
            DesktopEnvironment::Unknown
        }
    } else {
        DesktopEnvironment::Unknown
    }
}

/// Detect system theme preference.
///
/// Attempts to read theme from gsettings (GNOME) or kdeglobals (KDE).
/// Falls back to Auto for winit to handle via WindowEvent::ThemeChanged.
pub fn detect_system_theme() -> Theme {
    match detect_desktop_environment() {
        DesktopEnvironment::Gnome | DesktopEnvironment::Pantheon | DesktopEnvironment::Budgie => {
            // Try gsettings (no external library needed, just command)
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "color-scheme"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("prefer-dark") {
                    return Theme::Dark;
                } else if stdout.contains("prefer-light") {
                    return Theme::Light;
                }
            }
        }
        DesktopEnvironment::KDE => {
            // Try reading kdeglobals file
            if let Some(home) = std::env::var("HOME").ok() {
                let kdeglobals = format!("{}/.config/kdeglobals", home);
                if let Ok(content) = std::fs::read_to_string(&kdeglobals) {
                    if content.to_lowercase().contains("dark") {
                        return Theme::Dark;
                    }
                }
            }
        }
        _ => {}
    }

    // Let winit handle it
    Theme::Auto
}

/// Get Linux version info for logging.
pub fn version_info() -> String {
    let de = detect_desktop_environment();
    let ds = detect_display_server();
    format!("Linux ({:?}, {:?})", de, ds)
}
