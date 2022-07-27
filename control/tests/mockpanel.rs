use color_eyre::eyre::Context;
use color_eyre::Result;

use control::panel;

#[tokio::test]
async fn main() -> Result<()> {
    color_eyre::install()?;
    control::setup_tracing();

    let args = control::Args {
        ip: "192.168.1.101:6600".to_owned(),
        ..control::Args::default()
    };

    let panel =
        panel::Mock::try_connect().wrap_err("Could not connect to Panel")?;

    control::run(panel, args).await
}
