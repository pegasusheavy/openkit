//! Minimal softbuffer test to verify X11 presentation works.

use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Softbuffer Test")
                        .with_inner_size(winit::dpi::LogicalSize::new(400, 300)),
                )
                .unwrap(),
        );

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
                    let size = window.inner_size();
                    let width = size.width;
                    let height = size.height;

                    if width == 0 || height == 0 {
                        return;
                    }

                    surface
                        .resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap();

                    let mut buffer = surface.buffer_mut().unwrap();

                    // Fill with red
                    for y in 0..height {
                        for x in 0..width {
                            let color = if x < width / 2 {
                                0x00FF0000 // Red
                            } else {
                                0x000000FF // Blue
                            };
                            buffer[(y * width + x) as usize] = color;
                        }
                    }

                    buffer.present().unwrap();
                    println!("Presented {}x{} frame", width, height);
                }
            }
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        window: None,
        surface: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
