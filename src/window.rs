use anyhow::Result;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::{cloud_world::CloudWorld, graphics::Graphics};

pub async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("silky clouds")
        .build(&event_loop)?;
    let _ = window.request_inner_size(PhysicalSize::new(1200, 1200));

    let mut gfx = Graphics::new(window).await;
    let mut cloud_world = CloudWorld::new(&gfx);

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
                // TODO: Input?
            }
            WindowEvent::Resized(physical_size) => {
                gfx.resize(*physical_size);
            }
            WindowEvent::RedrawRequested => {
                cloud_world.update();
                match cloud_world.render(&gfx) {
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
