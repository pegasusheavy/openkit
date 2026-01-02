//! macOS platform detection utilities.
//!
//! OpenKit renders its own chrome - this module only provides detection utilities.
//! No macOS-specific UI libraries are used; all rendering is done via wgpu/tiny-skia.

#![cfg(target_os = "macos")]

use crate::theme::Theme;

/// Initialize macOS platform (no-op, OpenKit handles everything).
pub fn init() {
    log::debug!("OpenKit running on macOS");
}

/// Detect macOS theme preference.
///
/// Note: For accurate theme detection, OpenKit relies on winit's
/// `WindowEvent::ThemeChanged` events. This is a fallback.
pub fn detect_system_theme() -> Theme {
    // Winit handles theme detection via WindowEvent::ThemeChanged
    Theme::Auto
}

/// Check if dark mode is preferred.
pub fn prefers_dark_mode() -> bool {
    // Winit handles this
    false
}

/// Get macOS version info for logging.
pub fn version_info() -> String {
    "macOS".to_string()
}
