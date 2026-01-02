//! Windows platform detection utilities.
//!
//! OpenKit renders its own chrome - this module only provides detection utilities.
//! No Windows-specific UI libraries are used; all rendering is done via wgpu/tiny-skia.

#![cfg(target_os = "windows")]

use crate::theme::Theme;

/// Initialize Windows platform (no-op, OpenKit handles everything).
pub fn init() {
    log::debug!("OpenKit running on Windows");
}

/// Detect Windows theme preference.
///
/// Note: For accurate theme detection, OpenKit relies on winit's
/// `WindowEvent::ThemeChanged` events. This is a fallback.
pub fn detect_system_theme() -> Theme {
    // We could read the registry here, but winit already handles this
    // via WindowEvent::ThemeChanged. Just return Auto to use winit's detection.
    Theme::Auto
}

/// Check if dark mode is preferred.
pub fn prefers_dark_mode() -> bool {
    // Winit handles this - we just default to checking the theme event
    false
}

/// Get Windows version info for logging.
pub fn version_info() -> String {
    "Windows".to_string()
}
