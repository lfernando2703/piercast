use std::path::PathBuf;

/// Platform data directory for Piercast.
pub fn default_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Library/Application Support/Piercast")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Piercast")
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("piercast")
    }
}

pub fn socket_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("piercast.sock")
}

pub fn pairing_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("pairing.json")
}

pub fn db_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("registry.sqlite")
}

pub fn lock_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("piercast.lock")
}

pub const DEFAULT_HTTP_PORT: u16 = 47923;

pub fn http_port() -> u16 {
    std::env::var("PIERCAST_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_HTTP_PORT)
}
