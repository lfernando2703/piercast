//! Axum HTTP control plane on 127.0.0.1 (+ unix socket / windows pipe stubs).

mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use piercast_core::{paths, Engine};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub use routes::api_router;

pub struct ApiServer {
    pub engine: Arc<Engine>,
    pub port: u16,
}

impl ApiServer {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self {
            engine,
            port: paths::http_port(),
        }
    }

    pub fn router(&self) -> Router {
        api_router(self.engine.clone()).layer(TraceLayer::new_for_http())
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        let app = self.router();
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;
        tracing::info!(%addr, "piercast control plane listening");

        #[cfg(unix)]
        {
            let sock = paths::socket_path(self.engine.data_dir());
            let _ = std::fs::remove_file(&sock);
            // Create socket path placeholder file marker for clients; full uds hyper serve is stubbed.
            // Clients use TCP 127.0.0.1:47923.
            if let Ok(listener) = tokio::net::UnixListener::bind(&sock) {
                tracing::info!(path=%sock.display(), "unix socket bound (accept loop stub)");
                tokio::spawn(async move {
                    loop {
                        if listener.accept().await.is_err() {
                            break;
                        }
                    }
                });
            }
        }
        #[cfg(windows)]
        {
            tracing::info!(r"windows named pipe \\.\pipe\piercast stub — use TCP 127.0.0.1");
        }

        axum::serve(listener, app).await?;
        Ok(())
    }
}
