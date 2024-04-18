use nuage::window;
use pollster::FutureExt as _;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    window::run().block_on()?;
    Ok(())
}
