use std::{num::NonZeroU32, rc::Rc};

use tiny_skia::Pixmap;
use tracing::trace;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

struct Gfx {
    window: Rc<Window>,
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
}

pub trait Renderer {
    /// `width`/`height` are the window's physical size; the returned pixmap must match it.
    fn draw(&mut self, width: u32, height: u32, scale: f32) -> Pixmap;
}

struct App<F>
where
    F: Renderer,
{
    gfx: Option<Gfx>,
    renderer: F,
}

impl<F> App<F>
where
    F: Renderer,
{
    pub fn new(renderer: F) -> Self {
        Self {
            gfx: None,
            renderer,
        }
    }
}

impl<F> ApplicationHandler for App<F>
where
    F: Renderer,
{
    fn resumed(&mut self, el: &ActiveEventLoop) {
        let window = Rc::new(
            el.create_window(Window::default_attributes().with_title("rock"))
                .expect("failed to create window"),
        );
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        window.request_redraw();
        self.gfx = Some(Gfx { window, surface });
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gfx) = self.gfx.as_mut() else { return };

        match event {
            WindowEvent::CloseRequested => el.exit(),

            WindowEvent::KeyboardInput { event, .. }
                if event.state.is_pressed()
                    && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                el.exit();
            }

            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                gfx.window.request_redraw()
            }

            WindowEvent::RedrawRequested => {
                let size = gfx.window.inner_size();
                let (Some(w), Some(h)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                else {
                    return; // minimized: zero-sized
                };
                gfx.surface.resize(w, h).unwrap();

                // 1. draw into a tiny-skia pixmap the same size as the window buffer
                let scale = gfx.window.scale_factor() as f32;
                let pixmap = self.renderer.draw(w.get(), h.get(), scale);

                // 2. copy into the window buffer (softbuffer wants 0x00RRGGBB)
                let mut buffer = gfx.surface.buffer_mut().unwrap();
                for (dst, px) in buffer.iter_mut().zip(pixmap.pixels()) {
                    *dst = (px.red() as u32) << 16 | (px.green() as u32) << 8 | px.blue() as u32;
                }

                trace!("Rendered pixmap to buffer");
                buffer.present().unwrap();
            }

            _ => {}
        }
    }
}

pub trait Run {
    fn run_simple(self);
}

impl<T: Renderer> Run for T {
    fn run_simple(self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait); // sleep until an event arrives
        event_loop.run_app(&mut App::new(self)).unwrap();
    }
}
