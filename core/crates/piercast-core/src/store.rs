use std::path::Path;

use chrono::{DateTime, Utc};
use piercast_schema::AppConfig;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRow {
    pub app_id: String,
    pub launches: u64,
    pub foreground_opens: u64,
    pub total_runtime_ms: u64,
    pub last_launched_at: Option<DateTime<Utc>>,
    pub last_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRow {
    pub app_id: String,
    pub ts: DateTime<Utc>,
    pub cpu_pct: f32,
    pub mem_rss_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRow {
    pub app_id: Option<String>,
    pub ts: DateTime<Utc>,
    pub kind: String,
    pub detail_json: serde_json::Value,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(data_dir: &Path) -> rusqlite::Result<Self> {
        std::fs::create_dir_all(data_dir).ok();
        let conn = Connection::open(paths::db_path(data_dir))?;
        let s = Self { conn };
        s.migrate()?;
        Ok(s)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS apps (
                id TEXT PRIMARY KEY,
                config_json TEXT NOT NULL,
                last_mode TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS samples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_id TEXT NOT NULL,
                ts TEXT NOT NULL,
                cpu_pct REAL NOT NULL,
                mem_rss_bytes INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_samples_app_ts ON samples(app_id, ts);
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_id TEXT,
                ts TEXT NOT NULL,
                kind TEXT NOT NULL,
                detail_json TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS usage (
                app_id TEXT PRIMARY KEY,
                launches INTEGER NOT NULL DEFAULT 0,
                foreground_opens INTEGER NOT NULL DEFAULT 0,
                total_runtime_ms INTEGER NOT NULL DEFAULT 0,
                last_launched_at TEXT,
                last_updated_at TEXT
            );
            "#,
        )?;
        Ok(())
    }

    pub fn upsert_app(&self, app: &AppConfig) -> rusqlite::Result<()> {
        let now = Utc::now().to_rfc3339();
        let json = serde_json::to_string(app).expect("serialize app");
        self.conn.execute(
            "INSERT INTO apps(id, config_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET config_json=excluded.config_json, updated_at=excluded.updated_at",
            params![app.id, json, now],
        )?;
        self.conn.execute(
            "INSERT INTO usage(app_id, last_updated_at) VALUES (?1, ?2)
             ON CONFLICT(app_id) DO UPDATE SET last_updated_at=excluded.last_updated_at",
            params![app.id, now],
        )?;
        Ok(())
    }

    pub fn remove_app(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM apps WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn list_apps(&self) -> rusqlite::Result<Vec<AppConfig>> {
        let mut stmt = self.conn.prepare("SELECT config_json FROM apps ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            let s: String = row.get(0)?;
            Ok(s)
        })?;
        let mut out = Vec::new();
        for r in rows {
            let s = r?;
            if let Ok(app) = serde_json::from_str::<AppConfig>(&s) {
                out.push(app);
            }
        }
        Ok(out)
    }

    pub fn get_app(&self, id: &str) -> rusqlite::Result<Option<AppConfig>> {
        self.conn
            .query_row(
                "SELECT config_json FROM apps WHERE id=?1",
                params![id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|s| serde_json::from_str(&s).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e))))
            .transpose()
    }

    pub fn set_last_mode(&self, id: &str, mode: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE apps SET last_mode=?2 WHERE id=?1",
            params![id, mode],
        )?;
        Ok(())
    }

    pub fn last_mode(&self, id: &str) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT last_mode FROM apps WHERE id=?1",
                params![id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map(|o| o.flatten())
    }

    pub fn record_sample(&self, app_id: &str, cpu_pct: f32, mem_rss_bytes: u64) -> rusqlite::Result<()> {
        let ts = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO samples(app_id, ts, cpu_pct, mem_rss_bytes) VALUES (?1,?2,?3,?4)",
            params![app_id, ts, cpu_pct, mem_rss_bytes as i64],
        )?;
        Ok(())
    }

    pub fn recent_samples(&self, app_id: &str, limit: usize) -> rusqlite::Result<Vec<SampleRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT app_id, ts, cpu_pct, mem_rss_bytes FROM samples WHERE app_id=?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![app_id, limit as i64], |row| {
            let ts: String = row.get(1)?;
            Ok(SampleRow {
                app_id: row.get(0)?,
                ts: DateTime::parse_from_rfc3339(&ts)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                cpu_pct: row.get(2)?,
                mem_rss_bytes: row.get::<_, i64>(3)? as u64,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn record_event(
        &self,
        app_id: Option<&str>,
        kind: &str,
        detail: serde_json::Value,
    ) -> rusqlite::Result<()> {
        let ts = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO events(app_id, ts, kind, detail_json) VALUES (?1,?2,?3,?4)",
            params![app_id, ts, kind, detail.to_string()],
        )?;
        Ok(())
    }

    pub fn bump_launch(&self, app_id: &str) -> rusqlite::Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO usage(app_id, launches, last_launched_at, last_updated_at)
             VALUES (?1, 1, ?2, ?2)
             ON CONFLICT(app_id) DO UPDATE SET
               launches = launches + 1,
               last_launched_at = excluded.last_launched_at,
               last_updated_at = excluded.last_updated_at",
            params![app_id, now],
        )?;
        Ok(())
    }

    pub fn bump_foreground_open(&self, app_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO usage(app_id, foreground_opens) VALUES (?1, 1)
             ON CONFLICT(app_id) DO UPDATE SET foreground_opens = foreground_opens + 1",
            params![app_id],
        )?;
        Ok(())
    }

    pub fn add_runtime(&self, app_id: &str, ms: u64) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE usage SET total_runtime_ms = total_runtime_ms + ?2 WHERE app_id=?1",
            params![app_id, ms as i64],
        )?;
        Ok(())
    }

    pub fn usage(&self, app_id: &str) -> rusqlite::Result<UsageRow> {
        self.conn
            .query_row(
                "SELECT app_id, launches, foreground_opens, total_runtime_ms, last_launched_at, last_updated_at
                 FROM usage WHERE app_id=?1",
                params![app_id],
                |row| {
                    let parse = |s: Option<String>| {
                        s.and_then(|t| {
                            DateTime::parse_from_rfc3339(&t)
                                .ok()
                                .map(|d| d.with_timezone(&Utc))
                        })
                    };
                    Ok(UsageRow {
                        app_id: row.get(0)?,
                        launches: row.get::<_, i64>(1)? as u64,
                        foreground_opens: row.get::<_, i64>(2)? as u64,
                        total_runtime_ms: row.get::<_, i64>(3)? as u64,
                        last_launched_at: parse(row.get(4)?),
                        last_updated_at: parse(row.get(5)?),
                    })
                },
            )
            .or_else(|_| {
                Ok(UsageRow {
                    app_id: app_id.to_string(),
                    launches: 0,
                    foreground_opens: 0,
                    total_runtime_ms: 0,
                    last_launched_at: None,
                    last_updated_at: None,
                })
            })
    }
}
