//! Episodic memory backed by SQLite — conversation history and past events.

use neuraos_types::{MemoryEntry, MemoryKind, MemoryQuery};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// SQLite-backed episodic memory store.
pub struct EpisodicMemory {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl EpisodicMemory {
    /// Open or create the SQLite database.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EpisodicError> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = rusqlite::Connection::open(p)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )?;
        Self::create_schema(&conn)?;
        info!("EpisodicMemory opened at {}", p.display());
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// In-memory database for testing.
    pub fn in_memory() -> Result<Self, EpisodicError> {
        let conn = rusqlite::Connection::open_in_memory()?;
        Self::create_schema(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn create_schema(conn: &rusqlite::Connection) -> Result<(), EpisodicError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memory_entries (
                id            TEXT PRIMARY KEY,
                agent_id      TEXT NOT NULL,
                kind          TEXT NOT NULL,
                content       TEXT NOT NULL,
                importance    REAL NOT NULL DEFAULT 0.5,
                access_count  INTEGER NOT NULL DEFAULT 0,
                created_at    TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                expires_at    TEXT,
                metadata      TEXT NOT NULL DEFAULT '{}'
            );
            CREATE INDEX IF NOT EXISTS idx_agent_id ON memory_entries(agent_id);
            CREATE INDEX IF NOT EXISTS idx_kind ON memory_entries(kind);
            CREATE INDEX IF NOT EXISTS idx_importance ON memory_entries(importance DESC);
            CREATE INDEX IF NOT EXISTS idx_created_at ON memory_entries(created_at DESC);",
        )?;
        Ok(())
    }

    /// Insert or replace a memory entry.
    pub async fn insert(&self, entry: &MemoryEntry) -> Result<(), EpisodicError> {
        let conn = self.conn.lock().await;
        let kind = serde_json::to_string(&entry.kind)?;
        let metadata = serde_json::to_string(&entry.metadata)?;
        conn.execute(
            "INSERT OR REPLACE INTO memory_entries
             (id, agent_id, kind, content, importance, access_count,
              created_at, last_accessed, expires_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                entry.id,
                entry.agent_id,
                kind,
                entry.content,
                entry.importance,
                entry.access_count,
                entry.created_at.to_rfc3339(),
                entry.last_accessed.to_rfc3339(),
                entry.expires_at.map(|t| t.to_rfc3339()),
                metadata,
            ],
        )?;
        debug!("Inserted memory entry {}", entry.id);
        Ok(())
    }

    /// Query recent entries for an agent.
    pub async fn query_recent(
        &self,
        agent_id: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, EpisodicError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, kind, content, importance, access_count,
                    created_at, last_accessed, expires_at, metadata
             FROM memory_entries
             WHERE agent_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![agent_id, limit as i64], row_to_entry)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EpisodicError::from)
    }

    /// Query entries above a minimum importance threshold.
    pub async fn query_by_importance(
        &self,
        agent_id: &str,
        min_importance: f32,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, EpisodicError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, kind, content, importance, access_count,
                    created_at, last_accessed, expires_at, metadata
             FROM memory_entries
             WHERE agent_id = ?1 AND importance >= ?2
             ORDER BY importance DESC, created_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![agent_id, min_importance, limit as i64],
            row_to_entry,
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EpisodicError::from)
    }

    /// Full-text search on content (LIKE-based, not FTS5 for portability).
    pub async fn search_text(
        &self,
        agent_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, EpisodicError> {
        let conn = self.conn.lock().await;
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, kind, content, importance, access_count,
                    created_at, last_accessed, expires_at, metadata
             FROM memory_entries
             WHERE agent_id = ?1 AND content LIKE ?2 ESCAPE '\\'
             ORDER BY importance DESC, created_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![agent_id, pattern, limit as i64],
            row_to_entry,
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EpisodicError::from)
    }

    /// Delete an entry by ID.
    pub async fn delete(&self, id: &str) -> Result<bool, EpisodicError> {
        let conn = self.conn.lock().await;
        let n = conn.execute("DELETE FROM memory_entries WHERE id = ?1", [id])?;
        Ok(n > 0)
    }

    /// Delete all entries for an agent.
    pub async fn clear_agent(&self, agent_id: &str) -> Result<usize, EpisodicError> {
        let conn = self.conn.lock().await;
        let n = conn.execute(
            "DELETE FROM memory_entries WHERE agent_id = ?1",
            [agent_id],
        )?;
        Ok(n)
    }

    /// Remove expired entries. Returns count removed.
    pub async fn purge_expired(&self) -> Result<usize, EpisodicError> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().to_rfc3339();
        let n = conn.execute(
            "DELETE FROM memory_entries WHERE expires_at IS NOT NULL AND expires_at < ?1",
            [now],
        )?;
        if n > 0 {
            info!("Purged {} expired memory entries", n);
        }
        Ok(n)
    }

    /// Total entry count.
    pub async fn count(&self) -> Result<u64, EpisodicError> {
        let conn = self.conn.lock().await;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_entries",
            [],
            |row| row.get(0),
        )?;
        Ok(n as u64)
    }

    /// Bump access count and update last_accessed timestamp.
    pub async fn record_access(&self, id: &str) -> Result<(), EpisodicError> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE memory_entries
             SET access_count = access_count + 1, last_accessed = ?1
             WHERE id = ?2",
            rusqlite::params![now, id],
        )?;
        Ok(())
    }
}

fn row_to_entry(
    row: &rusqlite::Row<'_>,
) -> Result<MemoryEntry, rusqlite::Error> {
    use chrono::DateTime;
    use std::str::FromStr;

    let id: String = row.get(0)?;
    let agent_id: String = row.get(1)?;
    let kind_str: String = row.get(2)?;
    let content: String = row.get(3)?;
    let importance: f32 = row.get(4)?;
    let access_count: u32 = row.get(5)?;
    let created_str: String = row.get(6)?;
    let accessed_str: String = row.get(7)?;
    let expires_str: Option<String> = row.get(8)?;
    let meta_str: String = row.get(9)?;

    let kind: MemoryKind = serde_json::from_str(&kind_str).unwrap_or(MemoryKind::Episodic);
    let metadata = serde_json::from_str(&meta_str).unwrap_or_default();
    let created_at = DateTime::parse_from_rfc3339(&created_str)
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());
    let last_accessed = DateTime::parse_from_rfc3339(&accessed_str)
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());
    let expires_at = expires_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|d| d.with_timezone(&chrono::Utc))
    });

    Ok(MemoryEntry {
        id,
        agent_id,
        kind,
        content,
        embedding: None, // embeddings stored separately
        importance,
        access_count,
        created_at,
        last_accessed,
        expires_at,
        metadata,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum EpisodicError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
