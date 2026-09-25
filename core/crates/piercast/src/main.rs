use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use piercast_core::{paths, Engine, EngineConfig};
use piercast_schema::StartMode;

#[derive(Debug, Parser)]
#[command(name = "piercast", about = "Piercast CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List registered apps
    Apps,
    /// Upsert from piercast.yml
    Upsert { path: PathBuf },
    Start {
        id: String,
        #[arg(long, default_value = "development")]
        mode: String,
        #[arg(long)]
        force_install: bool,
    },
    Stop { id: String },
    Restart { id: String },
    Kill { id: String },
    Open { id: String },
    Logs {
        id: String,
        #[arg(long, default_value_t = 200)]
        tail: usize,
    },
    Expose {
        id: String,
        #[arg(long, default_value = "serve")]
        mode: String,
    },
    Status,
    Doctor,
    Deploy { id: String },
    /// Run MCP server over stdio (auto-starts daemon logic in-process)
    Mcp,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Mcp => {
            let engine = Engine::boot(EngineConfig::default())?;
            piercast_mcp::serve_stdio(engine).await?;
        }
        Commands::Doctor => {
            ensure_daemon().await?;
            let token = read_token()?;
            let port = paths::http_port();
            let url = format!("http://127.0.0.1:{port}/v1/health");
            let client = reqwest::Client::new();
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {token}"))
                .send()
                .await?;
            println!("daemon: {}", if resp.status().is_success() { "ok" } else { "fail" });
            println!("tailscale: {}", if piercast_mcp::tailscale_doctor_available() { "ok" } else { "unavailable" });
        }
        Commands::Status => {
            ensure_daemon().await?;
            let apps: serde_json::Value = api_get("/v1/apps").await?;
            println!("{}", serde_json::to_string_pretty(&apps)?);
        }
        Commands::Apps => {
            ensure_daemon().await?;
            let apps: serde_json::Value = api_get("/v1/apps").await?;
            println!("{}", serde_json::to_string_pretty(&apps)?);
        }
        Commands::Upsert { path } => {
            ensure_daemon().await?;
            // Prefer local engine upsert for path absolutizing when daemon remote
            let engine = Engine::boot(EngineConfig::default())?;
            let app = engine.upsert_from_yaml_path(&path)?;
            // Also POST to running daemon if different process — best effort
            let _ = api_post("/v1/apps", &app).await;
            println!("upserted {}", app.id);
        }
        Commands::Start { id, mode, force_install } => {
            ensure_daemon().await?;
            let body = serde_json::json!({"mode": mode, "force_install": force_install});
            let v = api_post(&format!("/v1/apps/{id}/start"), &body).await?;
            println!("{v}");
        }
        Commands::Stop { id } => {
            ensure_daemon().await?;
            let v = api_post(&format!("/v1/apps/{id}/stop"), &serde_json::json!({})).await?;
            println!("{v}");
        }
        Commands::Restart { id } => {
            ensure_daemon().await?;
            let v = api_post(&format!("/v1/apps/{id}/restart"), &serde_json::json!({})).await?;
            println!("{v}");
        }
        Commands::Kill { id } => {
            ensure_daemon().await?;
            let v = api_post(&format!("/v1/apps/{id}/kill"), &serde_json::json!({})).await?;
            println!("{v}");
        }
        Commands::Open { id } => {
            ensure_daemon().await?;
            let v = api_post(&format!("/v1/apps/{id}/open"), &serde_json::json!({})).await?;
            println!("{v}");
        }
        Commands::Logs { id, tail } => {
            ensure_daemon().await?;
            let v: serde_json::Value = api_get(&format!("/v1/apps/{id}/logs?tail={tail}")).await?;
            if let Some(lines) = v.get("lines").and_then(|l| l.as_array()) {
                for line in lines {
                    if let Some(s) = line.as_str() {
                        println!("{s}");
                    }
                }
            } else {
                println!("{v}");
            }
        }
        Commands::Expose { id, mode } => {
            ensure_daemon().await?;
            let body = serde_json::json!({"mode": mode});
            let v = api_post(&format!("/v1/apps/{id}/expose"), &body).await?;
            println!("{v}");
        }
        Commands::Deploy { id } => {
            bail!("deploy is explicit-only and not wired to Open/Start; use piercastd Deploy action / future piercast deploy for app `{id}`");
        }
    }
    Ok(())
}

async fn ensure_daemon() -> Result<()> {
    let port = paths::http_port();
    let url = format!("http://127.0.0.1:{port}/v1/health");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()?;
    if client.get(&url).send().await.is_ok() {
        return Ok(());
    }
    // auto-start piercastd
    let _ = std::process::Command::new("piercastd").spawn();
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if client.get(&url).send().await.is_ok() {
            return Ok(());
        }
    }
    bail!("could not reach piercastd on {url}");
}

fn read_token() -> Result<String> {
    let path = paths::pairing_path(&paths::default_data_dir());
    let text = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text)?;
    Ok(v.get("token").and_then(|t| t.as_str()).unwrap_or("").to_string())
}

async fn client() -> Result<(reqwest::Client, String, u16)> {
    let token = read_token()?;
    let port = paths::http_port();
    let client = reqwest::Client::new();
    Ok((client, token, port))
}

async fn api_get(path: &str) -> Result<serde_json::Value> {
    let (client, token, port) = client().await?;
    let resp = client
        .get(format!("http://127.0.0.1:{port}{path}"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await?)
}

async fn api_post(path: &str, body: &impl serde::Serialize) -> Result<serde_json::Value> {
    let (client, token, port) = client().await?;
    let resp = client
        .post(format!("http://127.0.0.1:{port}{path}"))
        .header("Authorization", format!("Bearer {token}"))
        .json(body)
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await.unwrap_or_else(|_| serde_json::json!({"ok": true})))
}

// silence unused import in some builds
#[allow(dead_code)]
fn _mode(s: &str) -> Result<StartMode> {
    s.parse().map_err(|e: piercast_schema::SchemaError| anyhow::anyhow!(e.to_string()))
}
