use clap::Parser;
use zap::NodeArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = NodeArgs::parse();
    args.run().await?;
    Ok(())
}
