use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use piercast_schema::{AppConfig, CommandSpec, HealthKind, StartMode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;
use tracing::info;

use crate::graph::{resolve_start_order, GraphError};
use crate::health::{HealthMachine, HealthState};
use crate::store::Store;

pub type SpawnHook = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Debug, Error)]
pub enum SupervisorError {
    #[error("app not found: {0}")]
    NotFound(String),
    #[error("dependency_cycle")]
    DependencyCycle,
    #[error("mode_unavailable: requested {requested}; configured: {configured:?}")]
    ModeUnavailable {
        requested: String,
        configured: Vec<String>,
    },
    #[error("start_timeout")]
    StartTimeout,
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonEvent {
    Status {
        app_id: String,
        status: RuntimeStatus,
        health: Option<String>,
    },
    Log {
        app_id: String,
        line: String,
    },
}

pub struct AppRuntime {
    pub config: AppConfig,
    pub status: RuntimeStatus,
    pub last_mode: Option<StartMode>,
    pub pid: Option<u32>,
    pub health: HealthState,
    pub started_at: Option<Instant>,
    pub log_buffer: Vec<String>,
    pub expose_url: Option<String>,
    child: Option<Child>,
    health_machine: Option<HealthMachine>,
}

#[derive(Clone)]
pub struct EngineConfig {
    pub data_dir: PathBuf,
    pub sample_interval: Duration,
    pub stop_escalate_after: Duration,
    pub open_timeout: Duration,
    pub skip_browser_open: bool,
    pub on_spawn: Option<SpawnHook>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            data_dir: crate::paths::default_data_dir(),
            sample_interval: Duration::from_secs(2),
            stop_escalate_after: Duration::from_secs(10),
            open_timeout: Duration::from_secs(60),
            skip_browser_open: false,
            on_spawn: None,
        }
    }
}

pub struct Engine {
    cfg: EngineConfig,
    store: Mutex<Store>,
    apps: RwLock<HashMap<String, AppRuntime>>,
    event_tx: broadcast::Sender<DaemonEvent>,
    pairing_token: RwLock<String>,
    bootstrap: Mutex<Option<BootstrapSession>>,
}

pub type Supervisor = Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapSession {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub base_url: String,
    pub host_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingFile {
    pub token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub devices: Vec<DeviceToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceToken {
    pub name: String,
    pub token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Engine {
    pub fn new(cfg: EngineConfig) -> Result<Arc<Self>, SupervisorError> {
        Self::boot(cfg)
    }

    pub fn boot(cfg: EngineConfig) -> Result<Arc<Self>, SupervisorError> {
        std::fs::create_dir_all(&cfg.data_dir)?;
        let store = Store::open(&cfg.data_dir).map_err(|e| SupervisorError::Other(e.to_string()))?;
        let (event_tx, _) = broadcast::channel(256);
        let token = load_or_create_token(&cfg.data_dir)?;
        let engine = Arc::new(Self {
            cfg: cfg.clone(),
            store: Mutex::new(store),
            apps: RwLock::new(HashMap::new()),
            event_tx,
            pairing_token: RwLock::new(token),
            bootstrap: Mutex::new(None),
        });
        {
            let apps = engine
                .store
                .lock()
                .unwrap()
                .list_apps()
                .map_err(|e| SupervisorError::Other(e.to_string()))?;
            let mut map = engine.apps.write().unwrap();
            for app in apps {
                let last_mode = engine
                    .store
                    .lock()
                    .unwrap()
                    .last_mode(&app.id)
                    .ok()
                    .flatten()
                    .and_then(|m| m.parse().ok());
                map.insert(
                    app.id.clone(),
                    AppRuntime {
                        config: app,
                        status: RuntimeStatus::Stopped,
                        last_mode,
                        pid: None,
                        health: HealthState::Unknown,
                        started_at: None,
                        log_buffer: Vec::new(),
                        expose_url: None,
                        child: None,
                        health_machine: None,
                    },
                );
            }
        }
        let bg = engine.clone();
        tokio::spawn(async move { bg.metrics_loop().await });
        let bg2 = engine.clone();
        tokio::spawn(async move { bg2.health_loop().await });
        Ok(engine)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DaemonEvent> {
        self.event_tx.subscribe()
    }

    pub fn auth_token(&self) -> String {
        self.pairing_token.read().unwrap().clone()
    }

    pub fn data_dir(&self) -> &Path {
        &self.cfg.data_dir
    }

    pub fn validate_bearer(&self, header: Option<&str>) -> bool {
        let Some(h) = header else {
            return false;
        };
        let token = h.strip_prefix("Bearer ").unwrap_or(h);
        if token == *self.pairing_token.read().unwrap() {
            return true;
        }
        if let Ok(pf) = load_pairing(&self.cfg.data_dir) {
            if pf.devices.iter().any(|d| d.token == token) {
                return true;
            }
        }
        if let Some(b) = self.bootstrap.lock().unwrap().as_ref() {
            if b.token == token && b.expires_at > chrono::Utc::now() {
                return true;
            }
        }
        false
    }

    pub fn upsert(&self, app: AppConfig) -> Result<(), SupervisorError> {
        app.validate()
            .map_err(|e| SupervisorError::Other(e.to_string()))?;
        self.store
            .lock()
            .unwrap()
            .upsert_app(&app)
            .map_err(|e| SupervisorError::Other(e.to_string()))?;
        let mut map = self.apps.write().unwrap();
        if let Some(rt) = map.get_mut(&app.id) {
            rt.config = app;
        } else {
            map.insert(
                app.id.clone(),
                AppRuntime {
                    config: app,
                    status: RuntimeStatus::Stopped,
                    last_mode: None,
                    pid: None,
                    health: HealthState::Unknown,
                    started_at: None,
                    log_buffer: Vec::new(),
                    expose_url: None,
                    child: None,
                    health_machine: None,
                },
            );
        }
        Ok(())
    }

    pub fn upsert_from_yaml_path(&self, path: &Path) -> Result<AppConfig, SupervisorError> {
        let s = std::fs::read_to_string(path)?;
        let mut app = AppConfig::from_yaml(&s).map_err(|e| SupervisorError::Other(e.to_string()))?;
        if !Path::new(&app.root).is_absolute() {
            let root = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .canonicalize()
                .unwrap_or_else(|_| path.parent().unwrap_or(Path::new(".")).to_path_buf());
            app.root = root.to_string_lossy().into();
        }
        self.upsert(app.clone())?;
        Ok(app)
    }

    pub fn remove(&self, id: &str) -> Result<(), SupervisorError> {
        self.store
            .lock()
            .unwrap()
            .remove_app(id)
            .map_err(|e| SupervisorError::Other(e.to_string()))?;
        self.apps.write().unwrap().remove(id);
        Ok(())
    }

    pub fn list(&self) -> Vec<AppConfig> {
        self.apps
            .read()
            .unwrap()
            .values()
            .map(|r| r.config.clone())
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<AppConfig> {
        self.apps.read().unwrap().get(id).map(|r| r.config.clone())
    }

    pub fn runtime_snapshot(&self, id: &str) -> Option<(RuntimeStatus, HealthState, Option<u32>)> {
        self.apps
            .read()
            .unwrap()
            .get(id)
            .map(|r| (r.status, r.health, r.pid))
    }

    pub fn is_running(&self, id: &str) -> Result<bool, SupervisorError> {
        let map = self.apps.read().unwrap();
        let rt = map
            .get(id)
            .ok_or_else(|| SupervisorError::NotFound(id.into()))?;
        Ok(matches!(rt.status, RuntimeStatus::Running | RuntimeStatus::Starting) && rt.pid.is_some())
    }

    pub fn pid(&self, id: &str) -> Result<Option<u32>, SupervisorError> {
        let map = self.apps.read().unwrap();
        let rt = map
            .get(id)
            .ok_or_else(|| SupervisorError::NotFound(id.into()))?;
        Ok(rt.pid)
    }

    pub async fn start(
        self: &Arc<Self>,
        id: &str,
        mode: StartMode,
        force_install: bool,
    ) -> Result<(), SupervisorError> {
        let apps_cfg: HashMap<String, AppConfig> = {
            let map = self.apps.read().unwrap();
            map.iter()
                .map(|(k, v)| (k.clone(), v.config.clone()))
                .collect()
        };
        let order = match resolve_start_order(id, &apps_cfg) {
            Ok(o) => o,
            Err(GraphError::DependencyCycle) => return Err(SupervisorError::DependencyCycle),
            Err(e) => return Err(e.into()),
        };

        for dep_id in order {
            let already = {
                let map = self.apps.read().unwrap();
                let rt = map
                    .get(&dep_id)
                    .ok_or_else(|| SupervisorError::NotFound(dep_id.clone()))?;
                matches!(rt.status, RuntimeStatus::Running | RuntimeStatus::Starting)
            };
            if already && dep_id != id {
                continue;
            }
            if already && dep_id == id {
                return Ok(());
            }

            let start_mode = if dep_id == id {
                mode
            } else {
                let map = self.apps.read().unwrap();
                map.get(&dep_id)
                    .and_then(|r| r.last_mode)
                    .unwrap_or(StartMode::Development)
            };

            self.start_one(&dep_id, start_mode, force_install && dep_id == id)
                .await?;
        }
        Ok(())
    }

    async fn start_one(
        self: &Arc<Self>,
        id: &str,
        mode: StartMode,
        force_install: bool,
    ) -> Result<(), SupervisorError> {
        let (config, maybe_spec) = {
            let map = self.apps.read().unwrap();
            let rt = map
                .get(id)
                .ok_or_else(|| SupervisorError::NotFound(id.into()))?;
            let configured: Vec<String> = rt
                .config
                .configured_modes()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            let spec = rt.config.start_spec(mode).cloned();
            if spec.is_none() {
                return Err(SupervisorError::ModeUnavailable {
                    requested: mode.as_str().into(),
                    configured,
                });
            }
            (rt.config.clone(), spec)
        };
        let spec = maybe_spec.unwrap();

        if let Some(install) = &config.commands.install {
            let need = force_install || should_run_install(install, Path::new(&config.root));
            if need {
                run_command_blocking(install, Path::new(&config.root))?;
            }
        }

        if mode == StartMode::Production {
            if let Some(build) = &config.commands.build {
                run_command_blocking(build, Path::new(&config.root))?;
            }
        }

        {
            let mut map = self.apps.write().unwrap();
            let rt = map.get_mut(id).unwrap();
            rt.status = RuntimeStatus::Starting;
            self.emit_status(id, RuntimeStatus::Starting, rt.health);
        }

        if let Some(hook) = &self.cfg.on_spawn {
            hook(id);
        }

        let child = spawn_process(&spec, Path::new(&config.root))?;
        let pid = child.id();

        {
            let mut map = self.apps.write().unwrap();
            let rt = map.get_mut(id).unwrap();
            rt.child = Some(child);
            rt.pid = Some(pid);
            rt.status = RuntimeStatus::Running;
            rt.last_mode = Some(mode);
            rt.started_at = Some(Instant::now());
            rt.health = HealthState::Unknown;
            if let Some(h) = &config.health {
                rt.health_machine =
                    Some(HealthMachine::new(h.healthy_threshold, h.unhealthy_threshold));
            } else {
                rt.health_machine = None;
                rt.health = HealthState::Healthy;
            }
            self.emit_status(id, RuntimeStatus::Running, rt.health);
        }

        let _ = self.store.lock().unwrap().set_last_mode(id, mode.as_str());
        let _ = self.store.lock().unwrap().bump_launch(id);
        let _ = self.store.lock().unwrap().record_event(
            Some(id),
            "started",
            serde_json::json!({ "mode": mode.as_str(), "pid": pid }),
        );
        info!(app=%id, pid, mode=%mode.as_str(), "started");
        Ok(())
    }

    pub async fn stop(self: &Arc<Self>, id: &str) -> Result<(), SupervisorError> {
        let (stop_cmd, root, pid) = {
            let map = self.apps.read().unwrap();
            let rt = map
                .get(id)
                .ok_or_else(|| SupervisorError::NotFound(id.into()))?;
            (
                rt.config.commands.stop.clone(),
                rt.config.root.clone(),
                rt.pid,
            )
        };

        {
            let mut map = self.apps.write().unwrap();
            if let Some(rt) = map.get_mut(id) {
                rt.status = RuntimeStatus::Stopping;
                self.emit_status(id, RuntimeStatus::Stopping, rt.health);
            }
        }

        if let Some(cmd) = stop_cmd {
            let _ = run_command_blocking(&cmd, Path::new(&root));
        } else if let Some(pid) = pid {
            signal_term(pid);
        }

        let escalate_after = self.cfg.stop_escalate_after;
        let deadline = Instant::now() + escalate_after;
        loop {
            let alive = {
                let mut map = self.apps.write().unwrap();
                if let Some(rt) = map.get_mut(id) {
                    if let Some(child) = rt.child.as_mut() {
                        match child.try_wait() {
                            Ok(Some(_)) => false,
                            Ok(None) => true,
                            Err(_) => false,
                        }
                    } else {
                        pid.map(process_alive).unwrap_or(false)
                    }
                } else {
                    false
                }
            };
            if !alive {
                break;
            }
            if Instant::now() >= deadline {
                if let Some(pid) = pid {
                    signal_kill(pid);
                }
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        self.finalize_stopped(id);
        Ok(())
    }

    pub async fn kill(self: &Arc<Self>, id: &str) -> Result<(), SupervisorError> {
        let pid = {
            let map = self.apps.read().unwrap();
            let rt = map
                .get(id)
                .ok_or_else(|| SupervisorError::NotFound(id.into()))?;
            rt.pid
        };
        if let Some(pid) = pid {
            signal_kill(pid);
        }
        {
            let mut map = self.apps.write().unwrap();
            if let Some(rt) = map.get_mut(id) {
                if let Some(mut child) = rt.child.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        self.finalize_stopped(id);
        Ok(())
    }

    pub async fn restart(self: &Arc<Self>, id: &str) -> Result<(), SupervisorError> {
        let mode = {
            let map = self.apps.read().unwrap();
            map.get(id)
                .ok_or_else(|| SupervisorError::NotFound(id.into()))?
                .last_mode
                .unwrap_or(StartMode::Development)
        };
        let _ = self.stop(id).await;
        self.start(id, mode, false).await
    }

    pub async fn open(self: &Arc<Self>, id: &str) -> Result<Option<String>, SupervisorError> {
        let (running, last_mode, open_url, primary) = {
            let map = self.apps.read().unwrap();
            let rt = map
                .get(id)
                .ok_or_else(|| SupervisorError::NotFound(id.into()))?;
            let running =
                matches!(rt.status, RuntimeStatus::Running | RuntimeStatus::Starting) && rt.pid.is_some();
            (
                running,
                rt.last_mode.unwrap_or(StartMode::Development),
                rt.config.open_url.clone(),
                rt.config.ports.as_ref().and_then(|p| p.primary),
            )
        };

        if !running {
            self.start(id, last_mode, false).await?;
        }

        let deadline = Instant::now() + self.cfg.open_timeout;
        loop {
            let health = {
                let map = self.apps.read().unwrap();
                map.get(id)
                    .map(|r| r.health)
                    .unwrap_or(HealthState::Unknown)
            };
            if health == HealthState::Healthy {
                break;
            }
            if Instant::now() >= deadline {
                return Err(SupervisorError::StartTimeout);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.poll_health_once(id).await;
        }

        let _ = self.store.lock().unwrap().bump_foreground_open(id);

        let url = if let Some(tpl) = open_url {
            Some(tpl.replace("{port}", &primary.unwrap_or(0).to_string()))
        } else {
            primary.map(|port| format!("http://127.0.0.1:{port}"))
        };

        if let Some(ref u) = url {
            if !self.cfg.skip_browser_open {
                let _ = open::that(u);
            }
        }
        Ok(url)
    }

    fn finalize_stopped(&self, id: &str) {
        let mut map = self.apps.write().unwrap();
        if let Some(rt) = map.get_mut(id) {
            if let Some(started) = rt.started_at.take() {
                let ms = started.elapsed().as_millis() as u64;
                let _ = self.store.lock().unwrap().add_runtime(id, ms);
            }
            rt.child = None;
            rt.pid = None;
            rt.status = RuntimeStatus::Stopped;
            rt.health = HealthState::Unknown;
            rt.health_machine = None;
            self.emit_status(id, RuntimeStatus::Stopped, rt.health);
        }
        let _ = self
            .store
            .lock()
            .unwrap()
            .record_event(Some(id), "stopped", serde_json::json!({}));
    }

    fn emit_status(&self, id: &str, status: RuntimeStatus, health: HealthState) {
        let _ = self.event_tx.send(DaemonEvent::Status {
            app_id: id.to_string(),
            status,
            health: Some(format!("{health:?}").to_lowercase()),
        });
    }

    pub fn logs(&self, id: &str, tail: usize) -> Result<Vec<String>, SupervisorError> {
        let map = self.apps.read().unwrap();
        let rt = map
            .get(id)
            .ok_or_else(|| SupervisorError::NotFound(id.into()))?;
        let len = rt.log_buffer.len();
        let start = len.saturating_sub(tail);
        Ok(rt.log_buffer[start..].to_vec())
    }

    pub fn stats(&self, id: &str) -> Result<serde_json::Value, SupervisorError> {
        if !self.apps.read().unwrap().contains_key(id) {
            return Err(SupervisorError::NotFound(id.into()));
        }
        let store = self.store.lock().unwrap();
        let usage = store
            .usage(id)
            .map_err(|e| SupervisorError::Other(e.to_string()))?;
        let samples = store
            .recent_samples(id, 60)
            .map_err(|e| SupervisorError::Other(e.to_string()))?;
        Ok(serde_json::json!({ "usage": usage, "samples": samples }))
    }

    pub fn pair_bootstrap(&self, base_url: &str) -> BootstrapSession {
        let token = uuid::Uuid::new_v4().to_string();
        let host_name = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "piercast".into());
        let session = BootstrapSession {
            token: token.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
            base_url: base_url.to_string(),
            host_name,
        };
        *self.bootstrap.lock().unwrap() = Some(session.clone());
        session
    }

    pub fn pair_complete(
        &self,
        bootstrap_token: &str,
        device_name: &str,
    ) -> Result<DeviceToken, SupervisorError> {
        let ok = {
            let b = self.bootstrap.lock().unwrap();
            matches!(
                b.as_ref(),
                Some(s) if s.token == bootstrap_token && s.expires_at > chrono::Utc::now()
            )
        };
        if !ok {
            return Err(SupervisorError::Other(
                "invalid or expired bootstrap token".into(),
            ));
        }
        *self.bootstrap.lock().unwrap() = None;
        let device = DeviceToken {
            name: device_name.to_string(),
            token: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
        };
        let mut pf = load_pairing(&self.cfg.data_dir).unwrap_or_else(|_| PairingFile {
            token: self.auth_token(),
            created_at: chrono::Utc::now(),
            devices: vec![],
        });
        pf.devices.push(device.clone());
        save_pairing(&self.cfg.data_dir, &pf)?;
        Ok(device)
    }

    pub fn set_expose_url(&self, id: &str, url: Option<String>) {
        let mut map = self.apps.write().unwrap();
        if let Some(rt) = map.get_mut(id) {
            rt.expose_url = url;
        }
    }

    pub fn expose_url(&self, id: &str) -> Option<String> {
        self.apps
            .read()
            .unwrap()
            .get(id)
            .and_then(|r| r.expose_url.clone())
    }

    async fn metrics_loop(self: Arc<Self>) {
        let mut sys = sysinfo::System::new();
        loop {
            tokio::time::sleep(self.cfg.sample_interval).await;
            let targets: Vec<(String, u32)> = {
                let map = self.apps.read().unwrap();
                map.iter()
                    .filter(|(_, r)| r.status == RuntimeStatus::Running)
                    .filter_map(|(id, r)| r.pid.map(|p| (id.clone(), p)))
                    .collect()
            };
            if targets.is_empty() {
                continue;
            }
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            for (id, pid) in targets {
                let pid = sysinfo::Pid::from_u32(pid);
                if let Some(proc) = sys.process(pid) {
                    let cpu = proc.cpu_usage();
                    let mem = proc.memory();
                    let _ = self.store.lock().unwrap().record_sample(&id, cpu, mem);
                }
            }
        }
    }

    async fn health_loop(self: Arc<Self>) {
        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let ids: Vec<String> = {
                let map = self.apps.read().unwrap();
                map.iter()
                    .filter(|(_, r)| r.status == RuntimeStatus::Running && r.config.health.is_some())
                    .map(|(id, _)| id.clone())
                    .collect()
            };
            for id in ids {
                self.poll_health_once(&id).await;
            }
        }
    }

    async fn poll_health_once(&self, id: &str) {
        let health_cfg = {
            let map = self.apps.read().unwrap();
            let Some(rt) = map.get(id) else {
                return;
            };
            let Some(h) = rt.config.health.clone() else {
                return;
            };
            h
        };
        let ok = check_health(&health_cfg).await;
        let mut map = self.apps.write().unwrap();
        if let Some(rt) = map.get_mut(id) {
            if let Some(m) = rt.health_machine.as_mut() {
                let prev = m.state();
                let next = m.observe(ok);
                rt.health = next;
                if prev != next {
                    self.emit_status(id, rt.status, next);
                }
            }
        }
    }
}

fn should_run_install(install: &CommandSpec, root: &Path) -> bool {
    let joined = install.argv.join(" ").to_lowercase();
    if joined.contains("npm")
        || joined.contains("pnpm")
        || joined.contains("yarn")
        || joined.contains("bun")
    {
        return !root.join("node_modules").exists();
    }
    if joined.contains("pip") || joined.contains("uv") || joined.contains("poetry") {
        if std::env::var_os("PIERCAST_SKIP_VENV_CHECK").is_some() {
            return false;
        }
        return !root.join(".venv").exists();
    }
    if joined.contains("cargo") || joined.contains("docker") {
        return false;
    }
    !root.join(".piercast_installed").exists()
}

fn run_command_blocking(spec: &CommandSpec, default_cwd: &Path) -> Result<(), SupervisorError> {
    let cwd = spec
        .cwd
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_cwd.to_path_buf());
    let mut cmd = build_command(spec, &cwd)?;
    let status = cmd.status()?;
    if !status.success() {
        return Err(SupervisorError::Other(format!(
            "command failed with {status}"
        )));
    }
    Ok(())
}

fn spawn_process(spec: &CommandSpec, default_cwd: &Path) -> Result<Child, SupervisorError> {
    let cwd = spec
        .cwd
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_cwd.to_path_buf());
    let _ = std::fs::create_dir_all(&cwd);
    let mut cmd = build_command(spec, &cwd)?;
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    Ok(cmd.spawn()?)
}

fn build_command(spec: &CommandSpec, cwd: &Path) -> Result<Command, SupervisorError> {
    let mut cmd = if spec.shell {
        let script = spec
            .argv
            .first()
            .ok_or_else(|| SupervisorError::Other("empty argv".into()))?;
        #[cfg(windows)]
        {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(script);
            c
        }
        #[cfg(not(windows))]
        {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
            let mut c = Command::new(shell);
            c.arg("-lc").arg(script);
            c
        }
    } else {
        let prog = spec
            .argv
            .first()
            .ok_or_else(|| SupervisorError::Other("empty argv".into()))?;
        let mut c = Command::new(prog);
        c.args(&spec.argv[1..]);
        c
    };
    cmd.current_dir(cwd);
    if let Some(env) = &spec.env {
        for (k, v) in env {
            cmd.env(k, v);
        }
    }
    Ok(cmd)
}

fn signal_term(pid: u32) {
    #[cfg(unix)]
    unsafe {
        libc::kill(-(pid as i32), libc::SIGTERM);
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string()])
            .status();
    }
}

fn signal_kill(pid: u32) {
    #[cfg(unix)]
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
        libc::kill(pid as i32, libc::SIGKILL);
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .status();
    }
}

fn process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, 0) == 0
    }
    #[cfg(windows)]
    {
        let _ = pid;
        true
    }
}

async fn check_health(cfg: &piercast_schema::HealthConfig) -> bool {
    match cfg.kind {
        HealthKind::Http => {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_millis(cfg.timeout_ms))
                .build();
            let Ok(client) = client else {
                return false;
            };
            match client.get(&cfg.target).send().await {
                Ok(resp) => resp.status().is_success(),
                Err(_) => false,
            }
        }
        HealthKind::Tcp => {
            let timeout = Duration::from_millis(cfg.timeout_ms);
            matches!(
                tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&cfg.target)).await,
                Ok(Ok(_))
            )
        }
        HealthKind::Command => {
            let argv: Result<Vec<String>, _> = serde_json::from_str(&cfg.target);
            let Ok(argv) = argv else {
                return false;
            };
            if argv.is_empty() {
                return false;
            }
            let mut cmd = Command::new(&argv[0]);
            cmd.args(&argv[1..]);
            match cmd.status() {
                Ok(s) => s.success(),
                Err(_) => false,
            }
        }
    }
}

fn load_or_create_token(data_dir: &Path) -> Result<String, SupervisorError> {
    let path = crate::paths::pairing_path(data_dir);
    if path.exists() {
        let pf = load_pairing(data_dir)?;
        return Ok(pf.token);
    }
    let pf = PairingFile {
        token: uuid::Uuid::new_v4().to_string(),
        created_at: chrono::Utc::now(),
        devices: vec![],
    };
    save_pairing(data_dir, &pf)?;
    Ok(pf.token)
}

fn load_pairing(data_dir: &Path) -> Result<PairingFile, SupervisorError> {
    let s = std::fs::read_to_string(crate::paths::pairing_path(data_dir))?;
    serde_json::from_str(&s).map_err(|e| SupervisorError::Other(e.to_string()))
}

fn save_pairing(data_dir: &Path, pf: &PairingFile) -> Result<(), SupervisorError> {
    let s = serde_json::to_string_pretty(pf).unwrap();
    std::fs::write(crate::paths::pairing_path(data_dir), s)?;
    Ok(())
}
