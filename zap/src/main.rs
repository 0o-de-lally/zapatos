use clap::Parser;
use zap::ZapArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = ZapArgs::parse();
    // Initialize logger if needed
    // ...
    args.run().await?;
    Ok(())
}
