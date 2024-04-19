use anyhow::Result;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::{cloud_world::CloudWorld, graphics::Graphics};

pub async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("silky clouds")
        .build(&event_loop)?;
    let _ = window.request_inner_size(PhysicalSize::new(800, 800));

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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Space),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {}
            WindowEvent::MouseInput { .. } => {
                // TODO: Input?
            }
            WindowEvent::Resized(physical_size) => {
                gfx.resize(*physical_size);
            }
            WindowEvent::RedrawRequested => {}
            _ => {}
        },
        _ => {}
    })?;

    Ok(())
}
