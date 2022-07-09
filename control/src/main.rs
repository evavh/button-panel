use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::Result;

use control::panel::UsartPanel;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    control::setup_tracing();
    let args = control::Args::parse();

    if args.setup {
        control::panel::setup_udev_access()
            .wrap_err("Could not set up udev rules")?;
        return Ok(());
    }

    let panel = UsartPanel::try_connect(&args.tty)
        .wrap_err("Could not connect to Panel")?;

    control::run(panel, args).await
}
