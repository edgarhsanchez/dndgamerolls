//! Legacy SQLite -> SurrealDB conversion helpers.
//!
//! This module contains the data-loading portion of the legacy character migration.
//! The actual upsert into SurrealDB is performed via [`CharacterDatabase`].

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use base64::Engine;

use crate::dice3d::types::database::CharacterDatabase;

fn decode_legacy_data_text(s: &str) -> Result<Vec<u8>, String> {
    let trimmed = s.trim();
    // Some encodings may prefix the payload.
    let trimmed = trimmed.strip_prefix("base64:").unwrap_or(trimmed);

    // Prefer base64 when it works (common for storing binary blobs as TEXT),
    // but fall back to returning the raw UTF-8 bytes (some legacy DBs stored
    // JSON/RON directly in TEXT columns).
    match base64::engine::general_purpose::STANDARD.decode(trimmed) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Ok(trimmed.as_bytes().to_vec()),
    }
}

/// A single legacy SQLite character row.
#[derive(Debug, Clone)]
pub struct LegacySqliteCharacterRow {
    pub legacy_id: i64,
    pub data: Vec<u8>,
}

/// Resolve the legacy SQLite database path (if the app-data folder can be resolved).
pub fn legacy_sqlite_path() -> Option<PathBuf> {
    CharacterDatabase::legacy_sqlite_path()
}

/// Load legacy character rows from the SQLite database.
///
/// This reads the raw `data` blob for each row and does not deserialize it.
pub fn load_legacy_character_rows(
    sqlite_path: &Path,
) -> Result<Vec<LegacySqliteCharacterRow>, String> {
    let conn = rusqlite::Connection::open(sqlite_path).map_err(|e| {
        format!(
            "Failed to open legacy SQLite database {:?}: {}",
            sqlite_path, e
        )
    })?;

    let mut stmt = conn
        .prepare("SELECT id, data FROM characters")
        .map_err(|e| format!("Failed to prepare legacy characters query: {}", e))?;

    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Failed to query legacy characters rows: {}", e))?;

    let mut out = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to iterate legacy characters rows: {}", e))?
    {
        let legacy_id: i64 = row
            .get(0)
            .map_err(|e| format!("Failed to decode legacy id: {}", e))?;

        let data = match row
            .get_ref(1)
            .map_err(|e| format!("Failed to read legacy data ref for id {}: {}", legacy_id, e))?
        {
            rusqlite::types::ValueRef::Blob(b) => b.to_vec(),
            rusqlite::types::ValueRef::Text(t) => {
                let s = std::str::from_utf8(t).map_err(|e| {
                    format!(
                        "Legacy SQLite data for id {} is not valid UTF-8: {}",
                        legacy_id, e
                    )
                })?;
                decode_legacy_data_text(s)?
            }
            rusqlite::types::ValueRef::Null => {
                return Err(format!(
                    "Legacy SQLite row id {} has NULL data; cannot migrate",
                    legacy_id
                ));
            }
            other => {
                return Err(format!(
                    "Legacy SQLite row id {} has unsupported data type {:?}; cannot migrate",
                    legacy_id, other
                ));
            }
        };

        out.push(LegacySqliteCharacterRow { legacy_id, data });
    }

    Ok(out)
}

/// Best-effort probe of a legacy SQLite DB's schema.
///
/// This is used for diagnostics when the expected `characters` table appears empty
/// (e.g. wrong file, different schema, or missing WAL sidecar files).
#[derive(Debug, Clone)]
pub struct LegacySqliteProbe {
    pub file_size_bytes: Option<u64>,
    pub file_modified: Option<SystemTime>,
    pub journal_mode: Option<String>,
    pub user_version: Option<i64>,
    pub schema_version: Option<i64>,
    pub characters_row_count: Option<i64>,
    pub characters_non_null_data_count: Option<i64>,
    pub tables: Vec<String>,
    pub characters_columns: Vec<String>,
}

/// Inspect table names and (if present) the columns of the `characters` table.
pub fn probe_legacy_sqlite(sqlite_path: &Path) -> Result<LegacySqliteProbe, String> {
    let meta = std::fs::metadata(sqlite_path).ok();
    let file_size_bytes = meta.as_ref().map(|m| m.len());
    let file_modified = meta.and_then(|m| m.modified().ok());

    let conn = rusqlite::Connection::open(sqlite_path).map_err(|e| {
        format!(
            "Failed to open legacy SQLite database {:?}: {}",
            sqlite_path, e
        )
    })?;

    // These pragmas/queries are best-effort; if they fail we keep going.
    let journal_mode = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
        .ok();
    let user_version = conn
        .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
        .ok();
    let schema_version = conn
        .query_row("PRAGMA schema_version", [], |row| row.get::<_, i64>(0))
        .ok();

    let characters_row_count = conn
        .query_row("SELECT COUNT(*) FROM characters", [], |row| {
            row.get::<_, i64>(0)
        })
        .ok();
    let characters_non_null_data_count = conn
        .query_row(
            "SELECT COUNT(*) FROM characters WHERE data IS NOT NULL AND length(data) > 0",
            [],
            |row| row.get::<_, i64>(0),
        )
        .ok();

    let mut tables_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .map_err(|e| format!("Failed to prepare sqlite_master query: {}", e))?;
    let tables_iter = tables_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to read sqlite_master rows: {}", e))?;
    let tables = tables_iter.flatten().collect::<Vec<_>>();

    // If the table doesn't exist, this returns an empty list.
    let mut chars_stmt = conn
        .prepare("PRAGMA table_info(characters)")
        .map_err(|e| format!("Failed to prepare PRAGMA table_info(characters): {}", e))?;
    let cols_iter = chars_stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("Failed to read PRAGMA table_info rows: {}", e))?;
    let characters_columns = cols_iter.flatten().collect::<Vec<_>>();

    Ok(LegacySqliteProbe {
        file_size_bytes,
        file_modified,
        journal_mode,
        user_version,
        schema_version,
        characters_row_count,
        characters_non_null_data_count,
        tables,
        characters_columns,
    })
}

/// Rename the legacy SQLite database file so the app doesn't keep re-checking it.
pub fn backup_legacy_sqlite(sqlite_path: &Path) -> Result<(), String> {
    let backup = sqlite_path.with_extension("db.bak");
    std::fs::rename(sqlite_path, &backup).map_err(|e| {
        format!(
            "Failed to rename legacy SQLite database {:?} -> {:?}: {}",
            sqlite_path, backup, e
        )
    })?;
    Ok(())
}
