//! Application entry point and lifecycle management.
//!
//! ## Custom CSS Styling
//!
//! Load custom CSS to override framework styles:
//!
//! ```rust,ignore
//! use openkit::prelude::*;
//! use openkit::css::StyleManager;
//!
//! // Create and configure style manager
//! let mut styles = StyleManager::new();
//! styles.load_file("./styles/custom.css")?;
//! styles.load_css(r#"
//!     .my-button {
//!         background-color: #3b82f6;
//!         border-radius: 8px;
//!     }
//! "#)?;
//! styles.set_variable("--accent", "#f59e0b");
//!
//! App::new()
//!     .title("Styled App")
//!     .styles(styles)
//!     .run(|| {
//!         button!("Custom Styled", class: "my-button")
//!     });
//! ```

use crate::css::{StyleContext, StyleManager};
use crate::event::{Event, WindowEvent};
use crate::geometry::{Rect, Size};
use crate::layout::Constraints;
use crate::platform::{Platform, PlatformEvent, PlatformError, Window, WindowBuilder};
use crate::render::Renderer;
use crate::theme::{Theme, ThemeData};
use crate::widget::{EventContext, LayoutContext, PaintContext, Widget};

use std::sync::Arc;

/// Application builder and runner.
pub struct App {
    title: String,
    size: Size,
    theme: Theme,
    resizable: bool,
    style_manager: Option<StyleManager>,
}

impl App {
    /// Create a new application builder.
    pub fn new() -> Self {
        Self {
            title: "OpenKit".to_string(),
            size: Size::new(800.0, 600.0),
            theme: Theme::Auto,
            resizable: true,
            style_manager: None,
        }
    }

    /// Set the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the initial window size.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Size::new(width, height);
        self
    }

    /// Set the theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set custom styles using a StyleManager.
    ///
    /// The StyleManager allows loading custom CSS to override framework styles.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use openkit::css::StyleManager;
    ///
    /// let mut styles = StyleManager::new();
    /// styles.load_css(r#"
    ///     .primary-btn {
    ///         background: linear-gradient(135deg, #667eea, #764ba2);
    ///     }
    /// "#)?;
    ///
    /// App::new()
    ///     .styles(styles)
    ///     .run(|| { /* ... */ });
    /// ```
    pub fn styles(mut self, manager: StyleManager) -> Self {
        self.style_manager = Some(manager);
        self
    }

    /// Load CSS from a file path.
    ///
    /// This is a convenience method that creates or updates the internal StyleManager.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// App::new()
    ///     .load_css_file("./custom.css")
    ///     .run(|| { /* ... */ });
    /// ```
    pub fn load_css_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        let manager = self.style_manager.get_or_insert_with(StyleManager::new);
        if let Err(e) = manager.load_file(path) {
            log::warn!("Failed to load CSS file: {}", e);
        }
        self
    }

    /// Load CSS from a string.
    ///
    /// This is a convenience method that creates or updates the internal StyleManager.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// App::new()
    ///     .load_css(r#"
    ///         .my-button { background-color: #3b82f6; }
    ///     "#)
    ///     .run(|| { /* ... */ });
    /// ```
    pub fn load_css(mut self, css: &str) -> Self {
        let manager = self.style_manager.get_or_insert_with(StyleManager::new);
        if let Err(e) = manager.load_css(css) {
            log::warn!("Failed to parse CSS: {}", e);
        }
        self
    }

    /// Set a CSS custom property (variable).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// App::new()
    ///     .css_var("--primary-color", "#3b82f6")
    ///     .css_var("--border-radius", "8px")
    ///     .run(|| { /* ... */ });
    /// ```
    pub fn css_var(mut self, name: &str, value: &str) -> Self {
        let manager = self.style_manager.get_or_insert_with(StyleManager::new);
        manager.set_variable(name, value);
        self
    }

    /// Run the application with the given root widget builder.
    pub fn run<F, W>(self, builder: F) -> Result<(), AppError>
    where
        F: FnOnce() -> W + 'static,
        W: Widget + 'static,
    {
        env_logger::init();

        let platform = Platform::new().map_err(AppError::Platform)?;

        let title = self.title.clone();
        let size = self.size;
        let resizable = self.resizable;
        let initial_theme = self.theme;
        let style_manager = self.style_manager.map(Arc::new);

        // State that will be initialized on resume
        let mut state: Option<AppState<W>> = None;
        let mut builder_opt: Option<F> = Some(builder);

        platform.run(move |event_loop, event| {
            match event {
                PlatformEvent::Resumed => {
                    // Create window and renderer
                    let window = WindowBuilder::new()
                        .title(&title)
                        .size(size.width, size.height)
                        .resizable(resizable)
                        .build(event_loop)
                        .expect("Failed to create window");

                    // Determine theme from window or use specified
                    let theme_data = match initial_theme {
                        Theme::Light => ThemeData::light(),
                        Theme::Dark => ThemeData::dark(),
                        Theme::Auto => {
                            if matches!(window.theme(), Theme::Dark) {
                                ThemeData::dark()
                            } else {
                                ThemeData::light()
                            }
                        }
                    };

                    let renderer = Renderer::new(&window);

                    // Build root widget
                    let root = if let Some(b) = builder_opt.take() {
                        b()
                    } else {
                        panic!("Builder already consumed");
                    };

                    state = Some(AppState {
                        window,
                        renderer,
                        root: Box::new(root),
                        theme_data,
                        style_manager: style_manager.clone(),
                        event_ctx: EventContext::new(),
                        needs_layout: true,
                        needs_paint: true,
                    });

                    // Request initial redraw
                    if let Some(s) = &state {
                        s.window.request_redraw();
                    }
                }
                PlatformEvent::Window { window_id, event } => {
                    if let Some(s) = &mut state {
                        if s.window.id() != window_id {
                            return;
                        }

                        match &event {
                            Event::Window(WindowEvent::CloseRequested) => {
                                event_loop.exit();
                            }
                            Event::Window(WindowEvent::Resized { width, height }) => {
                                let size = Size::new(*width as f32, *height as f32);
                                s.renderer.resize(size);
                                s.needs_layout = true;
                                s.needs_paint = true;
                                s.window.request_redraw();
                            }
                            Event::Window(WindowEvent::ThemeChanged { dark }) => {
                                s.theme_data = if *dark {
                                    ThemeData::dark()
                                } else {
                                    ThemeData::light()
                                };
                                s.needs_paint = true;
                                s.window.request_redraw();
                            }
                            Event::Mouse(mouse) => {
                                s.event_ctx.mouse_position = mouse.position;
                            }
                            _ => {}
                        }

                        // Dispatch event to widgets
                        s.root.handle_event(&event, &mut s.event_ctx);

                        if s.event_ctx.should_redraw {
                            s.event_ctx.should_redraw = false;
                            s.needs_paint = true;
                            s.window.request_redraw();
                        }
                    }
                }
                PlatformEvent::RedrawRequested { window_id } => {
                    if let Some(s) = &mut state {
                        if s.window.id() != window_id {
                            return;
                        }

                        // Layout if needed
                        if s.needs_layout {
                            s.needs_layout = false;
                            let size = s.window.size();
                            let style_ctx = if let Some(sm) = &s.style_manager {
                                StyleContext::with_styles(&s.theme_data, sm.clone())
                                    .with_viewport(size.width, size.height)
                            } else {
                                StyleContext::new(&s.theme_data)
                                    .with_viewport(size.width, size.height)
                            };
                            let layout_ctx = LayoutContext::new(&style_ctx);
                            let constraints = Constraints::tight(size);
                            s.root.layout(constraints, &layout_ctx);
                            s.root.set_bounds(Rect::from_origin_size(
                                crate::geometry::Point::ZERO,
                                size,
                            ));
                        }

                        // Paint if needed
                        if s.needs_paint {
                            s.needs_paint = false;
                            let size = s.window.size();
                            let style_ctx = if let Some(sm) = &s.style_manager {
                                StyleContext::with_styles(&s.theme_data, sm.clone())
                                    .with_viewport(size.width, size.height)
                            } else {
                                StyleContext::new(&s.theme_data)
                                    .with_viewport(size.width, size.height)
                            };
                            let paint_ctx = PaintContext::new(&style_ctx);

                            s.renderer.begin_frame(s.theme_data.colors.background);

                            let mut painter = s.renderer.painter();
                            let root_rect = Rect::from_origin_size(
                                crate::geometry::Point::ZERO,
                                size,
                            );
                            s.root.paint(&mut painter, root_rect, &paint_ctx);

                            let commands = painter.finish();
                            s.renderer.draw(&commands);

                            s.renderer.end_frame();
                        }
                    }
                }
                PlatformEvent::AboutToWait => {
                    // Nothing to do
                }
            }
        }).map_err(AppError::Platform)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal application state.
struct AppState<W: Widget> {
    window: Window,
    renderer: Renderer,
    root: Box<W>,
    theme_data: ThemeData,
    style_manager: Option<Arc<StyleManager>>,
    event_ctx: EventContext,
    needs_layout: bool,
    needs_paint: bool,
}

/// Application errors.
#[derive(Debug)]
pub enum AppError {
    Platform(PlatformError),
    Window(String),
    Render(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Platform(e) => write!(f, "Platform error: {}", e),
            AppError::Window(e) => write!(f, "Window error: {}", e),
            AppError::Render(e) => write!(f, "Render error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}
