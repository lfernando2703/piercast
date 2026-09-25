//! MCP tools over stdio (rmcp) and optional loopback HTTP.

use std::path::PathBuf;
use std::sync::Arc;

use piercast_core::Engine;
use piercast_schema::{AppConfig, ExposeMode, StartMode};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt};
use serde::Deserialize;
use rmcp::schemars::JsonSchema;

#[derive(Clone)]
pub struct PiercastMcp {
    engine: Arc<Engine>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpsertArgs {
    /// Absolute path to piercast.yml, or omit when `config` is set.
    #[serde(default)]
    path: Option<String>,
    /// Inline app JSON config.
    #[serde(default)]
    config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IdArgs {
    id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct StartArgs {
    id: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    force_install: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct LogsArgs {
    id: String,
    #[serde(default)]
    tail: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ExposeArgs {
    id: String,
    mode: String,
}

#[tool_router]
impl PiercastMcp {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self {
            engine,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(name = "piercast_upsert_app", description = "Register or update an app from piercast.yml path or inline JSON")]
    async fn piercast_upsert_app(&self, Parameters(args): Parameters<UpsertArgs>) -> Result<CallToolResult, McpError> {
        let app = if let Some(cfg_v) = args.config {
            let cfg: AppConfig = serde_json::from_value(cfg_v)
                .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
            cfg.validate().map_err(|e| McpError::invalid_params(e.to_string(), None))?;
            self.engine.upsert(cfg.clone()).map_err(mcp_err)?;
            cfg
        } else if let Some(path) = args.path {
            self.engine
                .upsert_from_yaml_path(&PathBuf::from(path))
                .map_err(mcp_err)?
        } else {
            return Err(McpError::invalid_params("path or config required", None));
        };
        ok_json(&app)
    }

    #[tool(name = "piercast_list_apps", description = "List registered apps")]
    async fn piercast_list_apps(&self) -> Result<CallToolResult, McpError> {
        ok_json(&self.engine.list())
    }

    #[tool(name = "piercast_get_app", description = "Get one app by id")]
    async fn piercast_get_app(&self, Parameters(args): Parameters<IdArgs>) -> Result<CallToolResult, McpError> {
        let app = self
            .engine
            .get(&args.id)
            .ok_or_else(|| McpError::invalid_params(format!("not found: {}", args.id), None))?;
        ok_json(&app)
    }

    #[tool(name = "piercast_remove_app", description = "Remove an app from the registry")]
    async fn piercast_remove_app(&self, Parameters(args): Parameters<IdArgs>) -> Result<CallToolResult, McpError> {
        self.engine.remove(&args.id).map_err(mcp_err)?;
        ok_json(&serde_json::json!({"ok": true}))
    }

    #[tool(name = "piercast_start", description = "Start an app")]
    async fn piercast_start(&self, Parameters(args): Parameters<StartArgs>) -> Result<CallToolResult, McpError> {
        let mode = args
            .mode
            .as_deref()
            .unwrap_or("development")
            .parse::<StartMode>()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        self.engine
            .start(&args.id, mode, args.force_install)
            .await
            .map_err(mcp_err)?;
        ok_json(&serde_json::json!({"ok": true}))
    }

    #[tool(name = "piercast_stop", description = "Stop an app")]
    async fn piercast_stop(&self, Parameters(args): Parameters<IdArgs>) -> Result<CallToolResult, McpError> {
        self.engine.stop(&args.id).await.map_err(mcp_err)?;
        ok_json(&serde_json::json!({"ok": true}))
    }

    #[tool(name = "piercast_restart", description = "Restart an app")]
    async fn piercast_restart(&self, Parameters(args): Parameters<IdArgs>) -> Result<CallToolResult, McpError> {
        self.engine.restart(&args.id).await.map_err(mcp_err)?;
        ok_json(&serde_json::json!({"ok": true}))
    }

    #[tool(name = "piercast_kill", description = "Force-kill an app")]
    async fn piercast_kill(&self, Parameters(args): Parameters<IdArgs>) -> Result<CallToolResult, McpError> {
        self.engine.kill(&args.id).await.map_err(mcp_err)?;
        ok_json(&serde_json::json!({"ok": true}))
    }

    #[tool(name = "piercast_open", description = "Open an app (auto-start if needed)")]
    async fn piercast_open(&self, Parameters(args): Parameters<IdArgs>) -> Result<CallToolResult, McpError> {
        let url = self.engine.open(&args.id).await.map_err(mcp_err)?;
        ok_json(&serde_json::json!({"ok": true, "url": url}))
    }

    #[tool(name = "piercast_logs", description = "Fetch recent app logs")]
    async fn piercast_logs(&self, Parameters(args): Parameters<LogsArgs>) -> Result<CallToolResult, McpError> {
        let lines = self
            .engine
            .logs(&args.id, args.tail.unwrap_or(200))
            .map_err(mcp_err)?;
        ok_json(&serde_json::json!({"lines": lines}))
    }

    #[tool(name = "piercast_stats", description = "Usage and resource samples")]
    async fn piercast_stats(&self, Parameters(args): Parameters<IdArgs>) -> Result<CallToolResult, McpError> {
        let stats = self.engine.stats(&args.id).map_err(mcp_err)?;
        ok_json(&stats)
    }

    #[tool(name = "piercast_expose", description = "Expose app via Tailscale serve/funnel/off")]
    async fn piercast_expose(&self, Parameters(args): Parameters<ExposeArgs>) -> Result<CallToolResult, McpError> {
        let app = self
            .engine
            .get(&args.id)
            .ok_or_else(|| McpError::invalid_params(format!("not found: {}", args.id), None))?;
        let port = app
            .ports
            .as_ref()
            .and_then(|p| p.primary)
            .ok_or_else(|| McpError::invalid_params("no primary port", None))?;
        let path = app
            .expose
            .as_ref()
            .map(|e| e.path.clone())
            .unwrap_or_else(|| "/".into());
        let mode = match args.mode.as_str() {
            "off" => ExposeMode::Off,
            "serve" => ExposeMode::Serve,
            "funnel" => ExposeMode::Funnel,
            other => {
                return Err(McpError::invalid_params(format!("bad mode {other}"), None));
            }
        };
        match piercast_tailscale::expose(port, mode, &path) {
            Ok(res) => {
                self.engine.set_expose_url(&args.id, res.url.clone());
                ok_json(&res)
            }
            Err(piercast_tailscale::TailscaleError::Unavailable) => Err(McpError::internal_error(
                format!(
                    "tailscale_unavailable: {}",
                    piercast_tailscale::DOWNLOAD_URL
                ),
                None,
            )),
            Err(e) => Err(mcp_err(e)),
        }
    }

    #[tool(name = "piercast_doctor", description = "Daemon + tailscale + socket health")]
    async fn piercast_doctor(&self) -> Result<CallToolResult, McpError> {
        let sock = piercast_core::paths::socket_path(&self.engine.data_dir());
        ok_json(&serde_json::json!({
            "daemon": "ok",
            "data_dir": self.engine.data_dir(),
            "socket_exists": sock.exists(),
            "tailscale": piercast_tailscale::doctor(),
            "http_port": piercast_core::paths::http_port(),
        }))
    }
}

#[tool_handler]
impl ServerHandler for PiercastMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Piercast local app control".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

fn ok_json<T: serde::Serialize>(v: &T) -> Result<CallToolResult, McpError> {
    let s = serde_json::to_string_pretty(v).map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![Content::text(s)]))
}

fn mcp_err<E: std::fmt::Display>(e: E) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

pub fn tailscale_doctor_available() -> bool {
    piercast_tailscale::is_available()
}

pub async fn serve_stdio(engine: Arc<Engine>) -> anyhow::Result<()> {
    let server = PiercastMcp::new(engine);
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Optional loopback HTTP MCP (streamable) — stub that documents intent; stdio is primary.
pub async fn serve_loopback_http(_engine: Arc<Engine>, _port: u16) -> anyhow::Result<()> {
    tracing::warn!("loopback HTTP MCP not enabled in this build; use `piercast mcp` stdio");
    Ok(())
}
