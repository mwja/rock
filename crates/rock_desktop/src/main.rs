use std::{num::NonZeroU32, rc::Rc};

use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};
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

#[derive(Default)]
struct App {
    gfx: Option<Gfx>,
}

impl ApplicationHandler for App {
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

            WindowEvent::Resized(_) => gfx.window.request_redraw(),

            WindowEvent::RedrawRequested => {
                let size = gfx.window.inner_size();
                let (Some(w), Some(h)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                else {
                    return; // minimized: zero-sized
                };
                gfx.surface.resize(w, h).unwrap();

                // 1. draw into a tiny-skia pixmap
                let pixmap = draw(w.get(), h.get());

                // 2. copy into the window buffer (softbuffer wants 0x00RRGGBB)
                let mut buffer = gfx.surface.buffer_mut().unwrap();
                for (dst, px) in buffer.iter_mut().zip(pixmap.pixels()) {
                    *dst = (px.red() as u32) << 16 | (px.green() as u32) << 8 | px.blue() as u32;
                }
                buffer.present().unwrap();
            }

            _ => {}
        }
    }
}

/// To be replaced by a renderer crate in the future.
fn draw(width: u32, height: u32) -> Pixmap {
    let mut pixmap = Pixmap::new(width, height).unwrap();
    pixmap.fill(Color::WHITE);

    let mut paint = Paint::default();
    paint.set_color_rgba8(220, 60, 60, 255);
    let square = Rect::from_xywh(100.0, 100.0, 200.0, 200.0).unwrap();
    pixmap.fill_rect(square, &paint, Transform::identity(), None);

    pixmap
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait); // sleep until an event arrives
    event_loop.run_app(&mut App::default()).unwrap();
}
