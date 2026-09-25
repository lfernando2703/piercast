//! Shell out to the `tailscale` CLI for serve / funnel.

use std::process::Command;

use piercast_schema::ExposeMode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DOWNLOAD_URL: &str = "https://tailscale.com/download";

#[derive(Debug, Error)]
pub enum TailscaleError {
    #[error("tailscale_unavailable: install from {DOWNLOAD_URL}")]
    Unavailable,
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposeResult {
    pub mode: ExposeMode,
    pub url: Option<String>,
}

pub fn is_available() -> bool {
    which_tailscale().is_some()
}

fn which_tailscale() -> Option<String> {
    if let Ok(p) = which("tailscale") {
        return Some(p);
    }
    None
}

fn which(bin: &str) -> Result<String, ()> {
    let output = Command::new("which").arg(bin).output().map_err(|_| ())?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.is_empty() {
            Err(())
        } else {
            Ok(s)
        }
    } else {
        Err(())
    }
}

pub fn expose(port: u16, mode: ExposeMode, path: &str) -> Result<ExposeResult, TailscaleError> {
    let Some(_bin) = which_tailscale() else {
        return Err(TailscaleError::Unavailable);
    };
    match mode {
        ExposeMode::Off => {
            let _ = run_args(&["serve", "reset"]);
            let _ = run_args(&["funnel", "reset"]);
            Ok(ExposeResult {
                mode: ExposeMode::Off,
                url: None,
            })
        }
        ExposeMode::Serve => {
            // Prefer background HTTPS serve of local app port — never the daemon API.
            let target = format!("http://127.0.0.1:{port}");
            let path = if path.is_empty() { "/" } else { path };
            run_args(&[
                "serve",
                "--bg",
                "--https=443",
                &format!("{path}={target}"),
            ])?;
            let url = parse_serve_url().or_else(|| Some(format!("https://localhost{path}")));
            Ok(ExposeResult {
                mode: ExposeMode::Serve,
                url,
            })
        }
        ExposeMode::Funnel => {
            run_args(&["funnel", "--bg", &port.to_string()])?;
            let url = parse_serve_url();
            Ok(ExposeResult {
                mode: ExposeMode::Funnel,
                url,
            })
        }
    }
}

fn run_args(args: &[&str]) -> Result<String, TailscaleError> {
    let output = Command::new("tailscale")
        .args(args)
        .output()
        .map_err(|_| TailscaleError::Unavailable)?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(TailscaleError::Other(err.trim().to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_serve_url() -> Option<String> {
    let out = run_args(&["serve", "status", "--json"]).ok()?;
    // Best-effort scrape for https URL
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&out) {
        let s = v.to_string();
        for token in s.split('"') {
            if token.starts_with("https://") {
                return Some(token.to_string());
            }
        }
    }
    for line in out.lines() {
        if let Some(idx) = line.find("https://") {
            return Some(line[idx..].split_whitespace().next()?.to_string());
        }
    }
    None
}

pub fn doctor() -> serde_json::Value {
    serde_json::json!({
        "available": is_available(),
        "download_url": DOWNLOAD_URL,
    })
}
