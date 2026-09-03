use rusqlite::{params, Connection};
use std::path::PathBuf;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub id: i64,
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
        let mut db_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        db_dir.push(".translate");
        std::fs::create_dir_all(&db_dir)?;

        let db_path = db_dir.join("history.db");
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT NOT NULL,
                result_summary TEXT NOT NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(Database { conn })
    }

    pub fn add_record(&self, query: &str, result_summary: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO history (query, result_summary, is_favorite, created_at) VALUES (?1, ?2, 0, datetime('now', 'localtime'))",
            params![query, result_summary],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn toggle_favorite(&self, id: i64) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT is_favorite FROM history WHERE id = ?1")?;
        let current_fav: i32 = stmt.query_row(params![id], |row| row.get(0))?;
        let new_fav = if current_fav == 1 { 0 } else { 1 };
        self.conn.execute(
            "UPDATE history SET is_favorite = ?1 WHERE id = ?2",
            params![new_fav, id],
        )?;
        Ok(new_fav == 1)
    }

    pub fn list_history(&self, only_favorites: bool, limit: usize) -> Result<Vec<HistoryItem>> {
        let sql = if only_favorites {
            "SELECT id, query, result_summary, is_favorite, created_at FROM history WHERE is_favorite = 1 ORDER BY id DESC LIMIT ?1"
        } else {
            "SELECT id, query, result_summary, is_favorite, created_at FROM history ORDER BY id DESC LIMIT ?1"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(HistoryItem {
                id: row.get(0)?,
                query: row.get(1)?,
                result_summary: row.get(2)?,
                is_favorite: row.get::<_, i32>(3)? == 1,
                created_at: row.get(4)?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn delete_record(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_history(&self) -> Result<()> {
        self.conn.execute("DELETE FROM history WHERE is_favorite = 0", [])?;
        Ok(())
    }
}
