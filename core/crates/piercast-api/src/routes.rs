use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::{SinkExt, StreamExt};
use piercast_core::{DaemonEvent, Engine, SupervisorError};
use piercast_schema::{AppConfig, ExposeMode, StartMode};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Engine>,
}

pub fn api_router(engine: Arc<Engine>) -> Router {
    let state = AppState { engine };
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/apps", get(list_apps).post(create_app))
        .route(
            "/v1/apps/{id}",
            get(get_app).put(put_app).delete(delete_app),
        )
        .route("/v1/apps/{id}/start", post(start_app))
        .route("/v1/apps/{id}/stop", post(stop_app))
        .route("/v1/apps/{id}/restart", post(restart_app))
        .route("/v1/apps/{id}/kill", post(kill_app))
        .route("/v1/apps/{id}/open", post(open_app))
        .route("/v1/apps/{id}/logs", get(logs_app))
        .route("/v1/apps/{id}/stats", get(stats_app))
        .route("/v1/apps/{id}/expose", post(expose_app))
        .route("/v1/events", get(events_ws))
        .route("/v1/pair/bootstrap", post(pair_bootstrap))
        .route("/v1/pair/complete", post(pair_complete))
        .with_state(state)
}

fn require_auth(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    if state.engine.validate_bearer(auth) {
        Ok(())
    } else {
        Err(ApiError::unauthorized())
    }
}

async fn health(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // health allows unauthenticated on loopback for local probes, but still checks if present
    let _ = headers;
    Json(serde_json::json!({
        "ok": true,
        "service": "piercastd",
        "version": env!("CARGO_PKG_VERSION"),
        "token_required": true,
        "auth_ok": state.engine.validate_bearer(
            headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok())
        ),
    }))
}

async fn list_apps(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    Ok(Json(state.engine.list()))
}

async fn create_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(app): Json<AppConfig>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    state.engine.upsert(app.clone()).map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(app)))
}

async fn get_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let app = state
        .engine
        .get(&id)
        .ok_or_else(|| ApiError::not_found(&id))?;
    let (status, health, pid) = state
        .engine
        .runtime_snapshot(&id)
        .unwrap_or((piercast_core::RuntimeStatus::Stopped, piercast_core::HealthState::Unknown, None));
    Ok(Json(serde_json::json!({
        "app": app,
        "status": status,
        "health": format!("{health:?}").to_lowercase(),
        "pid": pid,
        "expose_url": state.engine.expose_url(&id),
    })))
}

async fn put_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(mut app): Json<AppConfig>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    app.id = id;
    state.engine.upsert(app.clone()).map_err(ApiError::from)?;
    Ok(Json(app))
}

async fn delete_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    state.engine.remove(&id).map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct StartBody {
    pub mode: Option<String>,
    #[serde(default)]
    pub force_install: bool,
}

async fn start_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Option<Json<StartBody>>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let body = body.map(|j| j.0).unwrap_or(StartBody {
        mode: None,
        force_install: false,
    });
    let mode = body
        .mode
        .as_deref()
        .unwrap_or("development")
        .parse::<StartMode>()
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    state
        .engine
        .start(&id, mode, body.force_install)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true, "id": id, "mode": mode.as_str() })))
}

async fn stop_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    state.engine.stop(&id).await.map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn restart_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    state.engine.restart(&id).await.map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn kill_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    state.engine.kill(&id).await.map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn open_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let url = state.engine.open(&id).await.map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true, "url": url })))
}

#[derive(Debug, Deserialize)]
struct LogsQuery {
    tail: Option<usize>,
}

async fn logs_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<LogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let lines = state
        .engine
        .logs(&id, q.tail.unwrap_or(200))
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "lines": lines })))
}

async fn stats_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let stats = state.engine.stats(&id).map_err(ApiError::from)?;
    Ok(Json(stats))
}

#[derive(Debug, Deserialize)]
struct ExposeBody {
    mode: String,
}

async fn expose_app(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ExposeBody>,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let app = state
        .engine
        .get(&id)
        .ok_or_else(|| ApiError::not_found(&id))?;
    let port = app
        .ports
        .as_ref()
        .and_then(|p| p.primary)
        .ok_or_else(|| ApiError::bad_request("app has no primary port".into()))?;
    let path = app
        .expose
        .as_ref()
        .map(|e| e.path.clone())
        .unwrap_or_else(|| "/".into());
    let mode = match body.mode.as_str() {
        "off" => ExposeMode::Off,
        "serve" => ExposeMode::Serve,
        "funnel" => ExposeMode::Funnel,
        other => {
            return Err(ApiError::bad_request(format!("unknown expose mode {other}")));
        }
    };
    match piercast_tailscale::expose(port, mode, &path) {
        Ok(res) => {
            state.engine.set_expose_url(&id, res.url.clone());
            Ok(Json(serde_json::json!({
                "ok": true,
                "mode": body.mode,
                "url": res.url,
            })))
        }
        Err(piercast_tailscale::TailscaleError::Unavailable) => Err(ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "tailscale_unavailable".into(),
            message: format!(
                "tailscale CLI missing; download from {}",
                piercast_tailscale::DOWNLOAD_URL
            ),
        }),
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

async fn events_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let rx = state.engine.subscribe();
    Ok(ws.on_upgrade(move |socket| handle_ws(socket, rx)))
}

async fn handle_ws(socket: WebSocket, rx: tokio::sync::broadcast::Receiver<DaemonEvent>) {
    let (mut sink, mut stream) = socket.split();
    let mut rx = BroadcastStream::new(rx);
    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(p))) => {
                        let _ = sink.send(Message::Pong(p)).await;
                    }
                    _ => {}
                }
            }
            ev = rx.next() => {
                match ev {
                    Some(Ok(ev)) => {
                        if let Ok(text) = serde_json::to_string(&ev) {
                            if sink.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Err(_)) => continue,
                    None => break,
                }
            }
        }
    }
}

async fn pair_bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    require_auth(&state, &headers)?;
    let port = piercast_core::paths::http_port();
    let base = format!("http://127.0.0.1:{port}");
    let session = state.engine.pair_bootstrap(&base);
    Ok(Json(serde_json::json!({
        "base_url": session.base_url,
        "token": session.token,
        "host_name": session.host_name,
        "expires_at": session.expires_at,
    })))
}

#[derive(Debug, Deserialize)]
struct PairCompleteBody {
    token: String,
    device_name: Option<String>,
}

async fn pair_complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PairCompleteBody>,
) -> Result<impl IntoResponse, ApiError> {
    // bootstrap token itself authorizes complete
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let token = auth
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or(body.token.as_str());
    let device = state
        .engine
        .pair_complete(token, body.device_name.as_deref().unwrap_or("mobile"))
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({
        "device_name": device.name,
        "token": device.token,
        "created_at": device.created_at,
    })))
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: String,
    message: String,
}

impl ApiError {
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized".into(),
            message: "bearer token required".into(),
        }
    }
    fn not_found(id: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found".into(),
            message: format!("app {id} not found"),
        }
    }
    fn bad_request(message: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request".into(),
            message,
        }
    }
}

impl From<SupervisorError> for ApiError {
    fn from(e: SupervisorError) -> Self {
        let (status, code) = match &e {
            SupervisorError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            SupervisorError::DependencyCycle => (StatusCode::BAD_REQUEST, "dependency_cycle"),
            SupervisorError::ModeUnavailable { .. } => (StatusCode::BAD_REQUEST, "mode_unavailable"),
            SupervisorError::StartTimeout => (StatusCode::GATEWAY_TIMEOUT, "start_timeout"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "error"),
        };
        Self {
            status,
            code: code.into(),
            message: e.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.code,
                "message": self.message,
            })),
        )
            .into_response()
    }
}
