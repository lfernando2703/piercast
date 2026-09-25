//! Serde types matching `packages/schema/piercast.schema.json`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub commands: Commands,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env_files: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Ports>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthConfig>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default = "default_restart")]
    pub restart: RestartPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expose: Option<ExposeConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Resources>,
}

fn default_restart() -> RestartPolicy {
    RestartPolicy::OnFailure
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commands {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install: Option<CommandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean: Option<CommandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<CommandSpec>,
    pub start: StartCommands,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy: Option<CommandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<CommandSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartCommands {
    pub development: CommandSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub production: Option<CommandSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandSpec {
    pub argv: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(default)]
    pub shell: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ports {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Vec<u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthConfig {
    pub kind: HealthKind,
    pub target: String,
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_healthy_threshold")]
    pub healthy_threshold: u32,
    #[serde(default = "default_unhealthy_threshold")]
    pub unhealthy_threshold: u32,
}

fn default_interval() -> u64 {
    5000
}
fn default_timeout() -> u64 {
    2000
}
fn default_healthy_threshold() -> u32 {
    1
}
fn default_unhealthy_threshold() -> u32 {
    3
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthKind {
    Http,
    Tcp,
    Command,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExposeConfig {
    #[serde(default = "default_expose_mode")]
    pub default: ExposeMode,
    #[serde(default = "default_expose_path")]
    pub path: String,
}

fn default_expose_mode() -> ExposeMode {
    ExposeMode::Off
}
fn default_expose_path() -> String {
    "/".into()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExposeMode {
    Off,
    Serve,
    Funnel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resources {
    #[serde(default = "default_cpu_warn")]
    pub cpu_warn_percent: f32,
    #[serde(default = "default_mem_warn")]
    pub memory_warn_mb: u64,
}

fn default_cpu_warn() -> f32 {
    85.0
}
fn default_mem_warn() -> u64 {
    1024
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StartMode {
    Development,
    Production,
}

impl StartMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
    }
}

impl std::str::FromStr for StartMode {
    type Err = SchemaError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(SchemaError::Validation(format!(
                "unknown mode '{other}'"
            ))),
        }
    }
}

impl AppConfig {
    pub fn from_yaml(s: &str) -> Result<Self, SchemaError> {
        let app: Self = serde_yaml::from_str(s).map_err(|e| SchemaError::Parse(e.to_string()))?;
        app.validate()?;
        Ok(app)
    }

    pub fn from_json(s: &str) -> Result<Self, SchemaError> {
        let app: Self = serde_json::from_str(s).map_err(|e| SchemaError::Parse(e.to_string()))?;
        app.validate()?;
        Ok(app)
    }

    pub fn validate(&self) -> Result<(), SchemaError> {
        validate_id(&self.id)?;
        if self.name.is_empty() || self.name.len() > 256 {
            return Err(SchemaError::Validation("name length invalid".into()));
        }
        if self.root.is_empty() {
            return Err(SchemaError::Validation("root is required".into()));
        }
        validate_command(&self.commands.start.development, "start.development")?;
        if let Some(p) = &self.commands.start.production {
            validate_command(p, "start.production")?;
        }
        for (label, cmd) in [
            ("install", self.commands.install.as_ref()),
            ("clean", self.commands.clean.as_ref()),
            ("build", self.commands.build.as_ref()),
            ("deploy", self.commands.deploy.as_ref()),
            ("stop", self.commands.stop.as_ref()),
        ] {
            if let Some(c) = cmd {
                validate_command(c, label)?;
            }
        }
        if let Some(h) = &self.health {
            if h.target.is_empty() {
                return Err(SchemaError::Validation("health.target empty".into()));
            }
            if h.interval_ms < 100 {
                return Err(SchemaError::Validation("health.interval_ms < 100".into()));
            }
            if h.timeout_ms < 50 {
                return Err(SchemaError::Validation("health.timeout_ms < 50".into()));
            }
        }
        Ok(())
    }

    pub fn configured_modes(&self) -> Vec<&'static str> {
        let mut m = vec!["development"];
        if self.commands.start.production.is_some() {
            m.push("production");
        }
        m
    }

    pub fn start_spec(&self, mode: StartMode) -> Option<&CommandSpec> {
        match mode {
            StartMode::Development => Some(&self.commands.start.development),
            StartMode::Production => self.commands.start.production.as_ref(),
        }
    }
}

fn validate_id(id: &str) -> Result<(), SchemaError> {
    if id.is_empty() || id.len() > 128 {
        return Err(SchemaError::Validation("id length invalid".into()));
    }
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return Err(SchemaError::Validation("id empty".into()));
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(SchemaError::Validation(
            "id must start with [a-z0-9]".into(),
        ));
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
        return Err(SchemaError::Validation(
            "id must match ^[a-z0-9][a-z0-9_-]*$".into(),
        ));
    }
    Ok(())
}

fn validate_command(cmd: &CommandSpec, label: &str) -> Result<(), SchemaError> {
    if cmd.argv.is_empty() {
        return Err(SchemaError::Validation(format!(
            "{label}.argv must be non-empty"
        )));
    }
    if cmd.shell && cmd.argv.len() != 1 {
        return Err(SchemaError::Validation(format!(
            "{label}: shell=true requires argv length 1"
        )));
    }
    Ok(())
}

impl Default for RestartPolicy {
    fn default() -> Self { Self::OnFailure }
}
