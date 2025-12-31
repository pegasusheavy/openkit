//! Platform abstraction layer.
//!
//! Provides a unified interface for window management and event handling
//! across Windows, macOS, and Linux using winit.

mod window;

pub use window::{Window, WindowBuilder, WindowConfig};

use crate::event::{Event, KeyEvent, KeyEventKind, Key, Modifiers, MouseButton, MouseEvent, MouseEventKind, WindowEvent};
use crate::geometry::Point;
use crate::theme::Theme;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, WindowEvent as WinitWindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

/// Platform abstraction for running the application.
pub struct Platform {
    event_loop: Option<EventLoop<()>>,
}

impl Platform {
    /// Create a new platform instance.
    pub fn new() -> Result<Self, PlatformError> {
        let event_loop = EventLoop::new().map_err(|e| PlatformError::EventLoopCreation(e.to_string()))?;
        event_loop.set_control_flow(ControlFlow::Wait);

        Ok(Self {
            event_loop: Some(event_loop),
        })
    }

    /// Run the application event loop.
    pub fn run<F>(mut self, handler: F) -> Result<(), PlatformError>
    where
        F: FnMut(&ActiveEventLoop, PlatformEvent) + 'static,
    {
        let event_loop = self.event_loop.take().ok_or(PlatformError::AlreadyRunning)?;

        let mut app = PlatformApp {
            handler: Box::new(handler),
        };

        event_loop
            .run_app(&mut app)
            .map_err(|e| PlatformError::EventLoopRun(e.to_string()))
    }

    /// Detect the system theme preference.
    pub fn detect_theme() -> Theme {
        // This will be called after window creation to get actual theme
        Theme::Auto
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::new().expect("Failed to create platform")
    }
}

/// Platform events.
#[derive(Debug)]
pub enum PlatformEvent {
    /// Event loop resumed (create windows here)
    Resumed,
    /// Window event
    Window { window_id: WindowId, event: Event },
    /// Request to redraw
    RedrawRequested { window_id: WindowId },
    /// About to wait for events
    AboutToWait,
}

/// Platform-specific application handler.
#[allow(clippy::type_complexity)]
struct PlatformApp {
    handler: Box<dyn FnMut(&ActiveEventLoop, PlatformEvent)>,
}

impl ApplicationHandler for PlatformApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        (self.handler)(event_loop, PlatformEvent::Resumed);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WinitWindowEvent,
    ) {
        let platform_event = match event {
            WinitWindowEvent::CloseRequested => Some(Event::Window(WindowEvent::CloseRequested)),
            WinitWindowEvent::Resized(PhysicalSize { width, height }) => {
                Some(Event::Window(WindowEvent::Resized { width, height }))
            }
            WinitWindowEvent::Moved(PhysicalPosition { x, y }) => {
                Some(Event::Window(WindowEvent::Moved { x, y }))
            }
            WinitWindowEvent::Focused(focused) => {
                if focused {
                    Some(Event::Window(WindowEvent::Focused))
                } else {
                    Some(Event::Window(WindowEvent::Unfocused))
                }
            }
            WinitWindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                Some(Event::Window(WindowEvent::ScaleFactorChanged { scale_factor }))
            }
            WinitWindowEvent::ThemeChanged(theme) => {
                let dark = matches!(theme, winit::window::Theme::Dark);
                Some(Event::Window(WindowEvent::ThemeChanged { dark }))
            }
            WinitWindowEvent::CursorMoved { position, .. } => {
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Move,
                    Point::new(position.x as f32, position.y as f32),
                )))
            }
            WinitWindowEvent::CursorEntered { .. } => {
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Enter,
                    Point::ZERO,
                )))
            }
            WinitWindowEvent::CursorLeft { .. } => {
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Leave,
                    Point::ZERO,
                )))
            }
            WinitWindowEvent::MouseInput { state, button, .. } => {
                let kind = match state {
                    ElementState::Pressed => MouseEventKind::Down,
                    ElementState::Released => MouseEventKind::Up,
                };
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    winit::event::MouseButton::Back => MouseButton::Back,
                    winit::event::MouseButton::Forward => MouseButton::Forward,
                    winit::event::MouseButton::Other(id) => MouseButton::Other(id),
                };
                Some(Event::Mouse(
                    MouseEvent::new(kind, Point::ZERO).with_button(button),
                ))
            }
            WinitWindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        (x as i32 * 120, y as i32 * 120)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        (pos.x as i32, pos.y as i32)
                    }
                };
                Some(Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Scroll { delta_x, delta_y },
                    position: Point::ZERO,
                    button: None,
                    modifiers: Modifiers::empty(),
                }))
            }
            WinitWindowEvent::KeyboardInput { event, .. } => {
                let kind = match event.state {
                    ElementState::Pressed => KeyEventKind::Down,
                    ElementState::Released => KeyEventKind::Up,
                };
                let key = convert_key(&event.logical_key);
                let text = event.text.as_ref().map(|t| t.to_string());
                Some(Event::Key(KeyEvent {
                    kind,
                    key,
                    physical_key: None,
                    text,
                    modifiers: Modifiers::empty(), // TODO: Track modifiers
                    is_repeat: event.repeat,
                }))
            }
            WinitWindowEvent::RedrawRequested => {
                (self.handler)(
                    event_loop,
                    PlatformEvent::RedrawRequested { window_id },
                );
                return;
            }
            _ => None,
        };

        if let Some(event) = platform_event {
            (self.handler)(event_loop, PlatformEvent::Window { window_id, event });
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        (self.handler)(event_loop, PlatformEvent::AboutToWait);
    }
}

/// Convert winit key to OpenKit key.
fn convert_key(key: &winit::keyboard::Key) -> Key {
    use winit::keyboard::{Key as WKey, NamedKey};

    match key {
        WKey::Named(named) => match named {
            NamedKey::Enter => Key::Enter,
            NamedKey::Tab => Key::Tab,
            NamedKey::Space => Key::Space,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Delete => Key::Delete,
            NamedKey::Escape => Key::Escape,
            NamedKey::ArrowUp => Key::Up,
            NamedKey::ArrowDown => Key::Down,
            NamedKey::ArrowLeft => Key::Left,
            NamedKey::ArrowRight => Key::Right,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::PageUp => Key::PageUp,
            NamedKey::PageDown => Key::PageDown,
            NamedKey::Insert => Key::Insert,
            NamedKey::F1 => Key::F1,
            NamedKey::F2 => Key::F2,
            NamedKey::F3 => Key::F3,
            NamedKey::F4 => Key::F4,
            NamedKey::F5 => Key::F5,
            NamedKey::F6 => Key::F6,
            NamedKey::F7 => Key::F7,
            NamedKey::F8 => Key::F8,
            NamedKey::F9 => Key::F9,
            NamedKey::F10 => Key::F10,
            NamedKey::F11 => Key::F11,
            NamedKey::F12 => Key::F12,
            NamedKey::Shift => Key::Shift,
            NamedKey::Control => Key::Control,
            NamedKey::Alt => Key::Alt,
            NamedKey::Super => Key::Super,
            NamedKey::CapsLock => Key::CapsLock,
            NamedKey::NumLock => Key::NumLock,
            NamedKey::ScrollLock => Key::ScrollLock,
            NamedKey::PrintScreen => Key::PrintScreen,
            NamedKey::Pause => Key::Pause,
            _ => Key::Unknown,
        },
        WKey::Character(c) => {
            let c = c.to_lowercase().chars().next().unwrap_or(' ');
            match c {
                'a' => Key::A,
                'b' => Key::B,
                'c' => Key::C,
                'd' => Key::D,
                'e' => Key::E,
                'f' => Key::F,
                'g' => Key::G,
                'h' => Key::H,
                'i' => Key::I,
                'j' => Key::J,
                'k' => Key::K,
                'l' => Key::L,
                'm' => Key::M,
                'n' => Key::N,
                'o' => Key::O,
                'p' => Key::P,
                'q' => Key::Q,
                'r' => Key::R,
                's' => Key::S,
                't' => Key::T,
                'u' => Key::U,
                'v' => Key::V,
                'w' => Key::W,
                'x' => Key::X,
                'y' => Key::Y,
                'z' => Key::Z,
                '0' => Key::Num0,
                '1' => Key::Num1,
                '2' => Key::Num2,
                '3' => Key::Num3,
                '4' => Key::Num4,
                '5' => Key::Num5,
                '6' => Key::Num6,
                '7' => Key::Num7,
                '8' => Key::Num8,
                '9' => Key::Num9,
                '-' => Key::Minus,
                '=' => Key::Equal,
                '[' => Key::BracketLeft,
                ']' => Key::BracketRight,
                '\\' => Key::Backslash,
                ';' => Key::Semicolon,
                '\'' => Key::Quote,
                '`' => Key::Grave,
                ',' => Key::Comma,
                '.' => Key::Period,
                '/' => Key::Slash,
                ' ' => Key::Space,
                _ => Key::Unknown,
            }
        }
        _ => Key::Unknown,
    }
}

/// Platform error types.
#[derive(Debug, Clone)]
pub enum PlatformError {
    EventLoopCreation(String),
    EventLoopRun(String),
    WindowCreation(String),
    AlreadyRunning,
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformError::EventLoopCreation(e) => write!(f, "Failed to create event loop: {}", e),
            PlatformError::EventLoopRun(e) => write!(f, "Event loop error: {}", e),
            PlatformError::WindowCreation(e) => write!(f, "Failed to create window: {}", e),
            PlatformError::AlreadyRunning => write!(f, "Event loop is already running"),
        }
    }
}

impl std::error::Error for PlatformError {}
