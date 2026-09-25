//! TDD: graph order, cycle, open-auto-start, kill, health, schema validation.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use piercast_core::graph::{resolve_start_order, GraphError};
use piercast_core::health::{HealthMachine, HealthState};
use piercast_core::SupervisorError;
use piercast_core::{Engine, EngineConfig};
use piercast_schema::{
    AppConfig, CommandSpec, Commands, HealthConfig, HealthKind, Ports, StartCommands, StartMode,
};
use tempfile::TempDir;

fn sleep_app(id: &str, root: PathBuf, depends_on: Vec<String>) -> AppConfig {
    AppConfig {
        id: id.into(),
        name: id.into(),
        description: None,
        icon: None,
        root: root.to_string_lossy().into(),
        tags: None,
        commands: Commands {
            install: None,
            clean: None,
            build: None,
            start: StartCommands {
                development: CommandSpec {
                    argv: vec!["sleep".into(), "60".into()],
                    cwd: None,
                    env: None,
                    shell: false,
                },
                production: None,
            },
            deploy: None,
            stop: None,
        },
        env: None,
        env_files: None,
        ports: None,
        open_url: None,
        health: None,
        depends_on,
        restart: piercast_schema::RestartPolicy::Never,
        expose: None,
        resources: None,
    }
}

fn http_server_app(id: &str, root: PathBuf, port: u16) -> AppConfig {
    let mut app = sleep_app(id, root, vec![]);
    app.commands.start.development = CommandSpec {
        argv: vec![
            "python3".into(),
            "-m".into(),
            "http.server".into(),
            port.to_string(),
            "--bind".into(),
            "127.0.0.1".into(),
        ],
        cwd: None,
        env: None,
        shell: false,
    };
    app.ports = Some(Ports {
        primary: Some(port),
        extra: None,
    });
    app.health = Some(HealthConfig {
        kind: HealthKind::Http,
        target: format!("http://127.0.0.1:{port}/"),
        interval_ms: 200,
        timeout_ms: 500,
        healthy_threshold: 1,
        unhealthy_threshold: 3,
    });
    app.open_url = Some(format!("http://127.0.0.1:{port}/"));
    app
}

#[test]
fn schema_validation_accepts_valid_and_rejects_bad_id() {
    let yaml = r#"
id: demo-app
name: Demo
root: /tmp/demo
commands:
  start:
    development:
      argv: ["echo", "hi"]
"#;
    let ok = AppConfig::from_yaml(yaml).expect("valid yaml");
    assert_eq!(ok.id, "demo-app");

    let bad = r#"
id: Bad_ID!
name: Demo
root: /tmp/demo
commands:
  start:
    development:
      argv: ["echo", "hi"]
"#;
    assert!(AppConfig::from_yaml(bad).is_err());
}

#[test]
fn topological_start_order_b_before_a() {
    let mut apps = HashMap::new();
    apps.insert(
        "a".into(),
        sleep_app("a", PathBuf::from("/tmp/a"), vec!["b".into()]),
    );
    apps.insert(
        "b".into(),
        sleep_app("b", PathBuf::from("/tmp/b"), vec![]),
    );
    let order = resolve_start_order("a", &apps).expect("order");
    assert_eq!(order, vec!["b".to_string(), "a".to_string()]);
}

#[test]
fn dependency_cycle_errors() {
    let mut apps = HashMap::new();
    apps.insert(
        "a".into(),
        sleep_app("a", PathBuf::from("/tmp/a"), vec!["b".into()]),
    );
    apps.insert(
        "b".into(),
        sleep_app("b", PathBuf::from("/tmp/b"), vec!["a".into()]),
    );
    let err = resolve_start_order("a", &apps).unwrap_err();
    match err {
        GraphError::DependencyCycle => {}
        other => panic!("expected dependency_cycle, got {other:?}"),
    }
}

#[test]
fn health_state_machine_thresholds() {
    let mut m = HealthMachine::new(2, 3);
    assert_eq!(m.state(), HealthState::Unknown);
    m.observe(true);
    assert_eq!(m.state(), HealthState::Unknown);
    m.observe(true);
    assert_eq!(m.state(), HealthState::Healthy);
    m.observe(false);
    m.observe(false);
    assert_eq!(m.state(), HealthState::Healthy);
    m.observe(false);
    assert_eq!(m.state(), HealthState::Unhealthy);
    m.observe(true);
    m.observe(true);
    assert_eq!(m.state(), HealthState::Healthy);
}

#[tokio::test]
async fn start_respects_dependency_order_b_before_a() {
    let tmp = TempDir::new().unwrap();
    let order = Arc::new(Mutex::new(Vec::new()));
    let order_cb = order.clone();

    let engine = Engine::new(EngineConfig {
        data_dir: tmp.path().to_path_buf(),
        sample_interval: Duration::from_secs(60),
        stop_escalate_after: Duration::from_secs(10),
        open_timeout: Duration::from_secs(5),
        skip_browser_open: true,
        on_spawn: Some(Arc::new(move |id: &str| {
            order_cb.lock().unwrap().push(id.to_string());
        })),
    })
    .unwrap();

    std::fs::create_dir_all(tmp.path().join("b")).unwrap();
    std::fs::create_dir_all(tmp.path().join("a")).unwrap();
    engine
        .upsert(sleep_app("b", tmp.path().join("b"), vec![]))
        .unwrap();
    engine
        .upsert(sleep_app("a", tmp.path().join("a"), vec!["b".into()]))
        .unwrap();

    engine
        .start("a", StartMode::Development, false)
        .await
        .unwrap();

    let observed = order.lock().unwrap().clone();
    assert_eq!(observed, vec!["b".to_string(), "a".to_string()]);

    let _ = engine.kill("a").await;
    let _ = engine.kill("b").await;
}

#[tokio::test]
async fn open_auto_starts_when_not_running() {
    let tmp = TempDir::new().unwrap();
    let port = 18765u16;
    let engine = Engine::new(EngineConfig {
        data_dir: tmp.path().to_path_buf(),
        sample_interval: Duration::from_secs(60),
        stop_escalate_after: Duration::from_secs(10),
        open_timeout: Duration::from_secs(30),
        skip_browser_open: true,
        on_spawn: None,
    })
    .unwrap();

    engine
        .upsert(http_server_app("fixture", tmp.path().to_path_buf(), port))
        .unwrap();

    assert!(!engine.is_running("fixture").unwrap());
    engine.open("fixture").await.unwrap();
    assert!(engine.is_running("fixture").unwrap());

    let _ = engine.kill("fixture").await;
}

#[tokio::test]
async fn kill_terminates_process() {
    let tmp = TempDir::new().unwrap();
    let engine = Engine::new(EngineConfig {
        data_dir: tmp.path().to_path_buf(),
        sample_interval: Duration::from_secs(60),
        stop_escalate_after: Duration::from_secs(10),
        open_timeout: Duration::from_secs(5),
        skip_browser_open: true,
        on_spawn: None,
    })
    .unwrap();

    engine
        .upsert(sleep_app("victim", tmp.path().to_path_buf(), vec![]))
        .unwrap();
    engine
        .start("victim", StartMode::Development, false)
        .await
        .unwrap();
    assert!(engine.is_running("victim").unwrap());
    let pid = engine.pid("victim").unwrap().expect("pid");

    engine.kill("victim").await.unwrap();
    assert!(!engine.is_running("victim").unwrap());

    // process should be gone
    #[cfg(unix)]
    {
        let still = unsafe { libc::kill(pid as i32, 0) == 0 };
        assert!(!still, "pid {pid} still alive after kill");
    }
}

#[tokio::test]
async fn missing_mode_returns_mode_unavailable() {
    let tmp = TempDir::new().unwrap();
    let engine = Engine::new(EngineConfig {
        data_dir: tmp.path().to_path_buf(),
        sample_interval: Duration::from_secs(60),
        stop_escalate_after: Duration::from_secs(10),
        open_timeout: Duration::from_secs(5),
        skip_browser_open: true,
        on_spawn: None,
    })
    .unwrap();

    engine
        .upsert(sleep_app("only-dev", tmp.path().to_path_buf(), vec![]))
        .unwrap();

    let err = engine
        .start("only-dev", StartMode::Production, false)
        .await
        .unwrap_err();
    match err {
        SupervisorError::ModeUnavailable { configured, .. } => {
            assert!(configured.contains(&"development".to_string()));
            assert!(!configured.iter().any(|m| m == "production"));
        }
        other => panic!("expected mode_unavailable, got {other:?}"),
    }
}
