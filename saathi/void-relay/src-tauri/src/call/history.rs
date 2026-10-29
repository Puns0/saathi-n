use rusqlite::{Connection, params};
use parking_lot::Mutex;
use std::sync::Arc;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CallRecord {
    pub id: String,
    pub timestamp: String,
    pub number: String,
    pub duration_sec: u32,
    pub platform: String,
    pub language: String,
    pub summary: String,
    pub full_transcript: String,
}

pub fn init_db(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS call_history (
            id TEXT PRIMARY KEY,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            number TEXT NOT NULL,
            duration_sec INTEGER NOT NULL,
            platform TEXT NOT NULL,
            language TEXT NOT NULL,
            summary TEXT NOT NULL,
            full_transcript TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create call_history table");
}

pub fn add_record(db: &Arc<Mutex<Connection>>, record: &CallRecord) -> anyhow::Result<()> {
    let conn = db.lock();
    conn.execute(
        "INSERT INTO call_history (id, timestamp, number, duration_sec, platform, language, summary, full_transcript)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.id,
            record.timestamp,
            record.number,
            record.duration_sec,
            record.platform,
            record.language,
            record.summary,
            record.full_transcript,
        ],
    )?;
    Ok(())
}

pub fn get_history(db: &Arc<Mutex<Connection>>, limit: usize) -> anyhow::Result<Vec<CallRecord>> {
    let conn = db.lock();
    let mut stmt = conn.prepare("SELECT id, timestamp, number, duration_sec, platform, language, summary, full_transcript FROM call_history ORDER BY timestamp DESC LIMIT ?1")?;
    
    let records = stmt.query_map([limit as u32], |row| {
        Ok(CallRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            number: row.get(2)?,
            duration_sec: row.get(3)?,
            platform: row.get(4)?,
            language: row.get(5)?,
            summary: row.get(6)?,
            full_transcript: row.get(7)?,
        })
    })?;

    let mut result = Vec::new();
    for rec in records {
        if let Ok(r) = rec {
            result.push(r);
        }
    }
    
    Ok(result)
}

pub fn clear_history(db: &Arc<Mutex<Connection>>) -> anyhow::Result<()> {
    let conn = db.lock();
    conn.execute("DELETE FROM call_history", [])?;
    Ok(())
}
