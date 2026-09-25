//! Piercast core: registry, dependency graph, supervisor, health, metrics, SQLite.

pub mod error;
pub mod graph;
pub mod health;
pub mod paths;
pub mod store;
pub mod supervisor;
pub mod import_wizard;

pub use error::CoreError;
pub use graph::{resolve_start_order, GraphError};
pub use health::{HealthMachine, HealthState};
pub use supervisor::{
    AppRuntime, BootstrapSession, DaemonEvent, DeviceToken, Engine, EngineConfig, RuntimeStatus,
    Supervisor, SupervisorError,
};
