
use anyhow::Context;
use clap::Parser;
use piercast_api::ApiServer;
use piercast_core::{Engine, EngineConfig};

#[derive(Debug, Parser)]
#[command(name = "piercastd", about = "Piercast daemon")]
struct Args {
    /// Override data directory
    #[arg(long, env = "PIERCAST_DATA_DIR")]
    data_dir: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let mut cfg = EngineConfig::default();
    if let Some(d) = args.data_dir {
        cfg.data_dir = d;
    }
    let engine = Engine::new(cfg).context("open engine")?;
    tracing::info!(token = %engine.auth_token(), "pairing token ready (pairing.json)");
    ApiServer::new(engine).serve().await?;
    Ok(())
}
