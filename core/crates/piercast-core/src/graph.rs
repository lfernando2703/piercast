use std::collections::{HashMap, HashSet, VecDeque};

use piercast_schema::AppConfig;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("dependency_cycle")]
    DependencyCycle,
    #[error("missing dependency: {0}")]
    MissingDependency(String),
}

/// Topological start order for `root` and its transitive depends_on.
/// Dependencies come first (B before A when A depends_on B).
pub fn resolve_start_order(
    root: &str,
    apps: &HashMap<String, AppConfig>,
) -> Result<Vec<String>, GraphError> {
    if !apps.contains_key(root) {
        return Err(GraphError::MissingDependency(root.to_string()));
    }

    let mut needed = HashSet::new();
    let mut stack = vec![root.to_string()];
    while let Some(id) = stack.pop() {
        if !needed.insert(id.clone()) {
            continue;
        }
        let Some(app) = apps.get(&id) else {
            return Err(GraphError::MissingDependency(id));
        };
        for dep in &app.depends_on {
            if !apps.contains_key(dep) {
                return Err(GraphError::MissingDependency(dep.clone()));
            }
            stack.push(dep.clone());
        }
    }

    // Kahn on the subgraph
    let mut indeg: HashMap<String, usize> = needed.iter().map(|id| (id.clone(), 0)).collect();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for id in &needed {
        let app = &apps[id];
        for dep in &app.depends_on {
            if needed.contains(dep) {
                adj.entry(dep.clone()).or_default().push(id.clone());
                *indeg.get_mut(id).unwrap() += 1;
            }
        }
    }

    let mut q: VecDeque<String> = indeg
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(id, _)| id.clone())
        .collect();
    // stable-ish: prefer alphabetical when ties, but ensure root deps first via Kahn
    let mut q_vec: Vec<_> = q.drain(..).collect();
    q_vec.sort();
    q.extend(q_vec);

    let mut order = Vec::new();
    while let Some(n) = q.pop_front() {
        order.push(n.clone());
        if let Some(children) = adj.get(&n) {
            let mut nexts = Vec::new();
            for child in children {
                let d = indeg.get_mut(child).unwrap();
                *d -= 1;
                if *d == 0 {
                    nexts.push(child.clone());
                }
            }
            nexts.sort();
            q.extend(nexts);
        }
    }

    if order.len() != needed.len() {
        return Err(GraphError::DependencyCycle);
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use piercast_schema::{CommandSpec, Commands, StartCommands};

    fn app(id: &str, deps: &[&str]) -> AppConfig {
        AppConfig {
            id: id.into(),
            name: id.into(),
            description: None,
            icon: None,
            root: "/tmp".into(),
            tags: None,
            commands: Commands {
                install: None,
                clean: None,
                build: None,
                start: StartCommands {
                    development: CommandSpec {
                        argv: vec!["true".into()],
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
            depends_on: deps.iter().map(|s| (*s).to_string()).collect(),
            restart: piercast_schema::RestartPolicy::Never,
            expose: None,
            resources: None,
        }
    }

    #[test]
    fn b_before_a() {
        let mut m = HashMap::new();
        m.insert("a".into(), app("a", &["b"]));
        m.insert("b".into(), app("b", &[]));
        assert_eq!(
            resolve_start_order("a", &m).unwrap(),
            vec!["b".to_string(), "a".to_string()]
        );
    }
}
