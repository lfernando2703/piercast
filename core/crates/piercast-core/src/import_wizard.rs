//! Draft piercast.yml from package.json / Procfile / docker-compose.yml.

use std::path::{Path, PathBuf};

use piercast_schema::{AppConfig, CommandSpec, Commands, HealthConfig, HealthKind, Ports, StartCommands};
use serde_json::Value;

#[derive(Debug)]
pub struct ImportDraft {
    pub app: AppConfig,
    pub source: String,
}

pub fn detect_and_draft(root: &Path) -> Option<ImportDraft> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let id = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("app")
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let id = id.trim_matches('-').to_string();
    let id = if id.is_empty() { "app".into() } else { id };

    if root.join("docker-compose.yml").exists() || root.join("compose.yaml").exists() {
        let file = if root.join("docker-compose.yml").exists() {
            "docker-compose.yml"
        } else {
            "compose.yaml"
        };
        return Some(ImportDraft {
            source: file.into(),
            app: base(
                &id,
                &root,
                CommandSpec {
                    argv: vec!["docker".into(), "compose".into(), "up".into()],
                    cwd: None,
                    env: None,
                    shell: false,
                },
                Some(CommandSpec {
                    argv: vec!["docker".into(), "compose".into(), "down".into()],
                    cwd: None,
                    env: None,
                    shell: false,
                }),
                None,
            ),
        });
    }

    if root.join("package.json").exists() {
        let text = std::fs::read_to_string(root.join("package.json")).ok()?;
        let v: Value = serde_json::from_str(&text).ok()?;
        let scripts = v.get("scripts")?.as_object()?;
        let dev = ["dev", "start:dev", "develop"]
            .iter()
            .find(|s| scripts.contains_key(**s))
            .map(|s| s.to_string())
            .or_else(|| scripts.contains_key("start").then(|| "start".into()))?;
        let build = scripts.contains_key("build").then(|| CommandSpec {
            argv: vec!["npm".into(), "run".into(), "build".into()],
            cwd: None,
            env: None,
            shell: false,
        });
        let production = scripts.contains_key("start").then(|| CommandSpec {
            argv: vec!["npm".into(), "run".into(), "start".into()],
            cwd: None,
            env: None,
            shell: false,
        });
        let mut app = base(
            &id,
            &root,
            CommandSpec {
                argv: vec!["npm".into(), "run".into(), dev],
                cwd: None,
                env: None,
                shell: false,
            },
            None,
            Some(3000),
        );
        app.commands.install = Some(CommandSpec {
            argv: vec!["npm".into(), "install".into()],
            cwd: None,
            env: None,
            shell: false,
        });
        app.commands.build = build;
        app.commands.start.production = production;
        app.health = Some(HealthConfig {
            kind: HealthKind::Http,
            target: "http://127.0.0.1:3000/".into(),
            interval_ms: 5000,
            timeout_ms: 2000,
            healthy_threshold: 1,
            unhealthy_threshold: 3,
        });
        app.open_url = Some("http://127.0.0.1:{port}/".into());
        return Some(ImportDraft {
            source: "package.json".into(),
            app,
        });
    }

    if root.join("Procfile").exists() {
        let text = std::fs::read_to_string(root.join("Procfile")).ok()?;
        let web = text.lines().find(|l| l.starts_with("web:"))?;
        let cmd = web.trim_start_matches("web:").trim().to_string();
        return Some(ImportDraft {
            source: "Procfile".into(),
            app: base(
                &id,
                &root,
                CommandSpec {
                    argv: vec![cmd],
                    cwd: None,
                    env: None,
                    shell: true,
                },
                None,
                Some(5000),
            ),
        });
    }
    None
}

fn base(
    id: &str,
    root: &Path,
    start: CommandSpec,
    stop: Option<CommandSpec>,
    port: Option<u16>,
) -> AppConfig {
    AppConfig {
        id: id.into(),
        name: id.into(),
        description: Some(format!("Imported from {}", root.display())),
        icon: None,
        root: root.to_string_lossy().into(),
        tags: Some(vec!["imported".into()]),
        commands: Commands {
            install: None,
            clean: None,
            build: None,
            start: StartCommands {
                development: start,
                production: None,
            },
            deploy: None,
            stop,
        },
        env: None,
        env_files: None,
        ports: port.map(|primary| Ports {
            primary: Some(primary),
            extra: None,
        }),
        open_url: port.map(|p| format!("http://127.0.0.1:{p}/")),
        health: None,
        depends_on: vec![],
        restart: Default::default(),
        expose: None,
        resources: None,
    }
}

pub fn write_draft_yaml(draft: &ImportDraft, out: &PathBuf) -> std::io::Result<()> {
    let yaml = serde_yaml::to_string(&draft.app).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(out, yaml)
}
