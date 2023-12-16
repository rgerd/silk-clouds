use anyhow::Result;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::graphics::Graphics;

pub async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;

    let mut gfx = Graphics::new(window).await;

    event_loop.run(move |event, window_target| match event {
        Event::AboutToWait => {
            gfx.window().request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == gfx.window().id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => window_target.exit(),
            WindowEvent::KeyboardInput { .. } | WindowEvent::MouseInput { .. } => {
                gfx.input(event);
            }
            WindowEvent::Resized(physical_size) => {
                gfx.resize(*physical_size);
            }
            WindowEvent::RedrawRequested => {
                gfx.update();
                match gfx.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => gfx.resize(*gfx.size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => window_target.exit(),
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        },
        _ => {}
    })?;

    Ok(())
}
