use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Schema(#[from] piercast_schema::SchemaError),
    #[error(transparent)]
    Supervisor(#[from] crate::supervisor::SupervisorError),
    #[error(transparent)]
    Graph(#[from] crate::graph::GraphError),
}
