use std::process::exit;

use nuage::{cloud_world::CloudWorld, graphics::Graphics};

use gif::{Encoder, Frame, Repeat};

fn save_gif(path: &str, frames: &mut Vec<Vec<u8>>, speed: i32, size: u16) -> anyhow::Result<()> {
    let mut image = std::fs::File::create(path)?;
    let mut encoder = Encoder::new(&mut image, size, size, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for mut frame in frames {
        encoder.write_frame(&Frame::from_rgba_speed(size, size, &mut frame, speed))?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut gfx = Graphics::new(800, 800).await;
    let mut cloud_world = CloudWorld::new(&gfx);
    let mut frames: Option<Vec<Vec<u8>>> = Some(Vec::new());

    // Render 20 seconds of 30FPS
    for i in 0..(30 * 20) {
        cloud_world.update();
        match cloud_world.render(&gfx, &mut frames) {
            Ok(_) => {}
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => exit(-1),
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => exit(-1),
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }
    }

    println!("Saving to gif...");
    save_gif("clouds.gif", &mut frames.take().unwrap(), 1, 800).unwrap();
    println!("Saved!");
    Ok(())
}
