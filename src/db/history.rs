use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, Result};

const MAX_UNFAVORITED_HISTORY: i64 = 1_000;
const MAX_SUMMARY_CHARS: usize = 2_000;

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub query: String,
    pub result_summary: String,
    pub is_favorite: bool,
    pub created_at: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn init() -> Result<Self> {
        Self::open(default_db_path()?)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let conn = if path == Path::new(":memory:") {
            Connection::open_in_memory()?
        } else {
            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                std::fs::create_dir_all(parent)?;
            }
            Connection::open(path)?
        };

        conn.busy_timeout(Duration::from_millis(500))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrate(&conn)?;

        Ok(Self { conn })
    }

    pub fn add_record(&self, query: &str, result_summary: &str) -> Result<i64> {
        let query = query.trim();
        if query.is_empty() {
            return Err(AppError::InvalidInput("历史记录内容不能为空".to_string()));
        }

        let summary = truncate_chars(result_summary.trim(), MAX_SUMMARY_CHARS);
        let existing_id = self
            .conn
            .query_row(
                "SELECT id FROM history WHERE query = ?1 ORDER BY id DESC LIMIT 1",
                params![query],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;

        let id = if let Some(id) = existing_id {
            self.conn.execute(
                "UPDATE history
                 SET result_summary = ?1, created_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?2",
                params![summary, id],
            )?;
            id
        } else {
            self.conn.execute(
                "INSERT INTO history (query, result_summary, is_favorite, created_at)
                 VALUES (?1, ?2, 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                params![query, summary],
            )?;
            self.conn.last_insert_rowid()
        };

        self.prune_old_history()?;
        Ok(id)
    }

    pub fn list_history(&self, only_favorites: bool, limit: usize) -> Result<Vec<HistoryItem>> {
        let sql = if only_favorites {
            "SELECT query, result_summary, is_favorite, created_at
             FROM history
             WHERE is_favorite = 1
             ORDER BY created_at DESC, id DESC
             LIMIT ?1"
        } else {
            "SELECT query, result_summary, is_favorite, created_at
             FROM history
             ORDER BY created_at DESC, id DESC
             LIMIT ?1"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![limit.min(i64::MAX as usize) as i64], |row| {
            Ok(HistoryItem {
                query: row.get(0)?,
                result_summary: row.get(1)?,
                is_favorite: row.get::<_, i32>(2)? == 1,
                created_at: row.get(3)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn prune_old_history(&self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM history
             WHERE is_favorite = 0
               AND id NOT IN (
                   SELECT id FROM history
                   WHERE is_favorite = 0
                   ORDER BY created_at DESC, id DESC
                   LIMIT ?1
               )",
            params![MAX_UNFAVORITED_HISTORY],
        )?;
        Ok(())
    }
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS history (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             query TEXT NOT NULL,
             result_summary TEXT NOT NULL,
             is_favorite INTEGER NOT NULL DEFAULT 0,
             created_at TEXT NOT NULL DEFAULT (datetime('now'))
         );
         CREATE INDEX IF NOT EXISTS idx_history_favorite_created
             ON history (is_favorite, created_at DESC, id DESC);
         ",
    )?;
    Ok(())
}

fn default_db_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".translate").join("history.db"))
        .ok_or_else(|| AppError::General("无法确定本地数据目录".to_string()))
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{}…", truncated)
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_and_updates_history() {
        let db = Database::open(":memory:").unwrap();
        let first_id = db.add_record("hello", "你好").unwrap();
        db.add_record("world", "世界").unwrap();
        let second_id = db.add_record(" hello ", "您好").unwrap();

        assert_eq!(first_id, second_id);
        let items = db.list_history(false, 10).unwrap();
        assert_eq!(items.len(), 2);
        let hello = items.iter().find(|item| item.query == "hello").unwrap();
        assert_eq!(hello.result_summary, "您好");
    }
}
