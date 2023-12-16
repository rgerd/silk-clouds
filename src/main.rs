use pollster::FutureExt as _;
use slosh::window;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    window::run().block_on()?;
    Ok(())
}
