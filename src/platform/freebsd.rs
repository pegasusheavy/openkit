//! FreeBSD platform detection utilities.
//!
//! OpenKit renders its own chrome - this module only provides detection utilities.
//! No FreeBSD-specific UI libraries are used; all rendering is done via wgpu/tiny-skia.
//! Windowing is handled by winit (X11 on FreeBSD).

#![cfg(target_os = "freebsd")]

use crate::theme::Theme;

/// Desktop environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Gnome,
    KDE,
    XFCE,
    MATE,
    Other(String),
    Unknown,
}

/// Initialize FreeBSD platform.
pub fn init() {
    log::debug!("OpenKit running on FreeBSD");
    log::debug!("Desktop environment: {:?}", detect_desktop_environment());
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
        } else if desktop_lower.contains("mate") {
            DesktopEnvironment::MATE
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
pub fn detect_system_theme() -> Theme {
    match detect_desktop_environment() {
        DesktopEnvironment::Gnome => {
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

    Theme::Auto
}

/// Get FreeBSD version info for logging.
pub fn version_info() -> String {
    let de = detect_desktop_environment();
    format!("FreeBSD ({:?})", de)
}
