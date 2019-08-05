#![deny(clippy::all)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::single_match)]

use rgx::core::*;
use rgx::kit;
use rgx::kit::shape2d::{Batch, Fill, Shape, Stroke};

use cgmath::prelude::*;
use cgmath::Vector2;

use wgpu::winit::{
    ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
};

fn main() {
    env_logger::init();

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();

    ///////////////////////////////////////////////////////////////////////////
    // Setup renderer
    ///////////////////////////////////////////////////////////////////////////

    let mut r = Renderer::new(&window);

    let mut win = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let mut pip: kit::shape2d::Pipeline =
        r.pipeline(win.width as u32, win.height as u32, Blending::default());
    let mut running = true;

    let mut chain = r.swap_chain(win.width as u32, win.height as u32);

    ///////////////////////////////////////////////////////////////////////////
    // Render loop
    ///////////////////////////////////////////////////////////////////////////

    let rad = 64.;
    let mut rows: u32;
    let mut cols: u32;

    // Cursor position.
    let (mut mx, mut my) = (0., 0.);

    while running {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => match key {
                        VirtualKeyCode::Escape => {
                            running = false;
                        }
                        _ => {}
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        mx = position.x;
                        my = position.y;
                    }
                    WindowEvent::CloseRequested => {
                        running = false;
                    }
                    WindowEvent::Resized(size) => {
                        win = size.to_physical(window.get_hidpi_factor());

                        let (w, h) = (win.width as u32, win.height as u32);

                        pip.resize(w, h);
                        chain = r.swap_chain(w, h);
                    }
                    _ => {}
                }
            }
        });

        rows = (win.height as f32 / (rad * 2.)) as u32;
        cols = (win.width as f32 / (rad * 2.)) as u32;

        ///////////////////////////////////////////////////////////////////////////
        // Prepare shape view
        ///////////////////////////////////////////////////////////////////////////

        let mut batch = Batch::new();
        let cursor = Vector2::new((mx / win.width) as f32, 1. - (my / win.height) as f32);

        for i in 0..rows {
            let y = i as f32 * rad * 2.;

            for j in 0..cols {
                let x = j as f32 * rad * 2.;

                // Color
                let c1 = i as f32 / rows as f32;
                let c2 = j as f32 / cols as f32;

                let rpos = Vector2::new(i as f32 / rows as f32, j as f32 / cols as f32);
                let delta = Vector2::distance(rpos, cursor);
                let width = 1. + delta * (rad / 1.5);

                batch.add(Shape::Circle(
                    Vector2::new(x + rad, y + rad),
                    rad,
                    32,
                    Stroke::new(width, Rgba::new(0.5, c2, c1, 0.75)),
                    Fill::Empty(),
                ));
            }
        }

        let buffer = batch.finish(&r);

        ///////////////////////////////////////////////////////////////////////////
        // Create frame
        ///////////////////////////////////////////////////////////////////////////

        let out = chain.next();
        let mut frame = r.frame();

        ///////////////////////////////////////////////////////////////////////////
        // Draw frame
        ///////////////////////////////////////////////////////////////////////////

        {
            let pass = &mut frame.pass(PassOp::Clear(Rgba::TRANSPARENT), &out);

            pass.set_pipeline(&pip);
            pass.set_vertex_buffer(&buffer);
            pass.draw_buffer(0..buffer.size, 0..1);
        }
        r.submit(frame);
    }
}
