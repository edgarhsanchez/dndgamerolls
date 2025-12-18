//! Embedded SurrealDB database for character & settings storage.
//!
//! This module provides persistent storage for:
//! - characters
//! - app settings
//! - command history
//!
//! The embedded database is stored in the same app-data folder previously used for the
//! legacy SQLite `characters.db` file.

use bevy::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use std::sync::Mutex;

use surrealdb::engine::local::{Db, Mem, SurrealKv};
use surrealdb::Surreal;
use surrealdb::types::{Array as SurrealArray, Object as SurrealObject, Value as SurrealValue};

use super::character::{CharacterListEntry, CharacterSheet};

/// Legacy SQLite database file name (for one-time migration).
const LEGACY_SQLITE_FILE: &str = "characters.db";
/// SurrealDB folder name (embedded database).
const DATABASE_FOLDER: &str = "characters.surrealdb";
/// App data folder name.
const APP_DATA_FOLDER: &str = "DnDGameRolls";

const NS: &str = "dndgamerolls";
const DB: &str = "dndgamerolls";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CharacterDocument {
    /// Stable internal ID (never changes).
    sid: i64,
    /// Legacy SQLite character ID, if this record was converted.
    ///
    /// This is used to prevent repeated conversion runs from re-importing
    /// the same characters.
    #[serde(default)]
    legacy_sqlite_id: Option<i64>,
    /// Convenience fields for listing/indexing.
    name: String,
    class: String,
    race: String,
    level: i32,
    /// Full character sheet.
    sheet: CharacterSheet,
}

fn surreal_value_to_json(value: SurrealValue) -> JsonValue {
    value.into_json_value()
}

fn json_to_surreal_value(value: &JsonValue) -> SurrealValue {
    match value {
        JsonValue::Null => SurrealValue::Null,
        JsonValue::Bool(b) => SurrealValue::from_t(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                SurrealValue::from_t(i)
            } else if let Some(u) = n.as_u64() {
                // SurrealDB Number supports signed ints; clamp if needed.
                SurrealValue::from_t(i64::try_from(u).unwrap_or(i64::MAX))
            } else if let Some(f) = n.as_f64() {
                SurrealValue::from_t(f)
            } else {
                SurrealValue::Null
            }
        }
        JsonValue::String(s) => SurrealValue::from_t(s.clone()),
        JsonValue::Array(arr) => {
            let inner: Vec<SurrealValue> = arr.iter().map(json_to_surreal_value).collect();
            SurrealValue::Array(SurrealArray::from(inner))
        }
        JsonValue::Object(map) => {
            let mut obj = SurrealObject::new();
            for (k, v) in map {
                obj.insert(k.clone(), json_to_surreal_value(v));
            }
            SurrealValue::Object(obj)
        }
    }
}

fn to_surreal_value<T: Serialize>(value: &T, label: &str) -> Result<SurrealValue, String> {
    let json_value =
        serde_json::to_value(value).map_err(|e| format!("Failed to serialize {} to JSON: {}", label, e))?;
    Ok(json_to_surreal_value(&json_value))
}

fn from_surreal_value<T: DeserializeOwned>(value: SurrealValue, label: &str) -> Result<T, String> {
    let json_value = surreal_value_to_json(value);
    serde_json::from_value(json_value)
        .map_err(|e| format!("Failed to decode {} from SurrealDB value: {}", label, e))
}

/// Resource for managing the character database.
#[derive(Resource)]
pub struct CharacterDatabase {
    rt: tokio::runtime::Runtime,
    db: Mutex<Surreal<Db>>,
    /// Path to the embedded datastore.
    pub db_path: PathBuf,
}

impl CharacterDatabase {
    /// Get the app data directory for storing the database.
    /// Uses LocalAppData on Windows, which is accessible to MSIX apps.
    fn get_data_dir() -> Result<PathBuf, String> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let app_dir = PathBuf::from(&local_app_data).join(APP_DATA_FOLDER);
                match std::fs::create_dir_all(&app_dir) {
                    Ok(_) => return Ok(app_dir),
                    Err(e) => {
                        eprintln!("Failed to create app data directory {:?}: {}", app_dir, e);
                    }
                }
            }

            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                let app_dir = PathBuf::from(&user_profile)
                    .join("AppData")
                    .join("Local")
                    .join(APP_DATA_FOLDER);
                if std::fs::create_dir_all(&app_dir).is_ok() {
                    return Ok(app_dir);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let app_dir = PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join(APP_DATA_FOLDER);
                std::fs::create_dir_all(&app_dir)
                    .map_err(|e| format!("Failed to create app data directory: {}", e))?;
                return Ok(app_dir);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
                let app_dir = PathBuf::from(data_home).join(APP_DATA_FOLDER);
                std::fs::create_dir_all(&app_dir)
                    .map_err(|e| format!("Failed to create app data directory: {}", e))?;
                return Ok(app_dir);
            } else if let Ok(home) = std::env::var("HOME") {
                let app_dir = PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join(APP_DATA_FOLDER);
                std::fs::create_dir_all(&app_dir)
                    .map_err(|e| format!("Failed to create app data directory: {}", e))?;
                return Ok(app_dir);
            }
        }

        Ok(std::env::current_dir().unwrap_or_default())
    }

    fn make_runtime() -> Result<tokio::runtime::Runtime, String> {
        tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create tokio runtime: {}", e))
    }

    async fn init(db: &Surreal<Db>) -> Result<(), String> {
        db.use_ns(NS)
            .use_db(DB)
            .await
            .map_err(|e| format!("Failed to select namespace/db: {}", e))?;

        // Minimal schema + indexes.
        let schema = r#"
            DEFINE TABLE character SCHEMALESS;
            DEFINE INDEX character_sid_unique ON character FIELDS sid UNIQUE;
            DEFINE INDEX character_name ON character FIELDS name;
            DEFINE INDEX character_legacy_sqlite_id ON character FIELDS legacy_sqlite_id;

            DEFINE TABLE setting SCHEMALESS;

            DEFINE TABLE command_history SCHEMALESS;
        "#;

        db.query(schema)
            .await
            .map_err(|e| format!("Failed to initialize schema: {}", e))?;

        Ok(())
    }

    /// Open or create the database.
    pub fn open() -> Result<Self, String> {
        let data_dir = Self::get_data_dir()?;
        let db_path = data_dir.join(DATABASE_FOLDER);

        let rt = Self::make_runtime()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database folder {:?}: {}", parent, e))?;
        }

        let db = rt
            .block_on(async { Surreal::new::<SurrealKv>(db_path.to_string_lossy().to_string()).await })
            .map_err(|e| format!("Failed to open SurrealDB: {}", e))?;

        rt.block_on(Self::init(&db))?;

        let this = Self {
            rt,
            db: Mutex::new(db),
            db_path: db_path.clone(),
        };

        Ok(this)
    }

    /// Path to the legacy SQLite database file (if the app-data folder can be resolved).
    pub fn legacy_sqlite_path() -> Option<PathBuf> {
        Self::get_data_dir()
            .ok()
            .map(|dir| dir.join(LEGACY_SQLITE_FILE))
    }

    /// Returns true if a character record exists with the given SurrealDB record id.
    pub fn character_id_exists(&self, id: i64) -> Result<bool, String> {
        self.with_db(|db| {
            self.rt.block_on(async {
                // Avoid typed record decoding; use raw SurrealDB Value.
                let record: Option<SurrealValue> = db
                    .select(("character", id))
                    .await
                    .map_err(|e| format!("Failed to check character existence: {}", e))?;
                Ok(record.is_some())
            })
        })
    }

    /// Returns true if any character record has `legacy_sqlite_id == legacy_id`.
    pub fn legacy_sqlite_id_exists(&self, legacy_id: i64) -> Result<bool, String> {
        self.with_db(|db| {
            self.rt.block_on(async {
                let mut response = db
                    .query("SELECT VALUE sid FROM character WHERE legacy_sqlite_id = $id LIMIT 1")
                    .bind(("id", legacy_id))
                    .await
                    .map_err(|e| format!("Failed to query legacy_sqlite_id: {}", e))?;
                let rows: Vec<i64> = response
                    .take(0)
                    .map_err(|e| format!("Failed to decode legacy_sqlite_id query: {}", e))?;
                Ok(!rows.is_empty())
            })
        })
    }

    /// Open database at a specific path (for testing).
    pub fn open_at(path: PathBuf) -> Result<Self, String> {
        let rt = Self::make_runtime()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database folder {:?}: {}", parent, e))?;
        }

        let db = rt
            .block_on(async { Surreal::new::<SurrealKv>(path.to_string_lossy().to_string()).await })
            .map_err(|e| format!("Failed to open SurrealDB: {}", e))?;

        rt.block_on(Self::init(&db))?;

        Ok(Self {
            rt,
            db: Mutex::new(db),
            db_path: path,
        })
    }

    /// "In-memory" database for testing: uses a unique temp folder.
    pub fn open_in_memory() -> Result<Self, String> {
        let rt = Self::make_runtime()?;
        let db = rt
            .block_on(async { Surreal::new::<Mem>(()).await })
            .map_err(|e| format!("Failed to open SurrealDB (mem): {}", e))?;

        rt.block_on(Self::init(&db))?;

        Ok(Self {
            rt,
            db: Mutex::new(db),
            db_path: PathBuf::new(),
        })
    }

    fn with_db<T>(&self, f: impl FnOnce(&Surreal<Db>) -> Result<T, String>) -> Result<T, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        f(&db)
    }

    fn next_character_id(&self) -> Result<i64, String> {
        self.with_db(|db| {
            self.rt.block_on(async {
                let mut response = db
                    .query("SELECT VALUE sid FROM character ORDER BY sid DESC LIMIT 1")
                    .await
                    .map_err(|e| format!("Failed to query next id: {}", e))?;
                let rows: Vec<i64> = response
                    .take(0)
                    .map_err(|e| format!("Failed to decode next id: {}", e))?;
                Ok(rows.first().map(|sid| sid + 1).unwrap_or(1))
            })
        })
    }

    fn to_doc(
        sid: i64,
        sheet: &CharacterSheet,
        legacy_sqlite_id: Option<i64>,
    ) -> CharacterDocument {
        CharacterDocument {
            sid,
            legacy_sqlite_id,
            name: sheet.character.name.clone(),
            class: sheet.character.class.clone(),
            race: sheet.character.race.clone(),
            level: sheet.character.level,
            sheet: sheet.clone(),
        }
    }

    /// Save a new character and return its ID.
    pub fn create_character(&self, sheet: &CharacterSheet) -> Result<i64, String> {
        let sid = self.next_character_id()?;
        self.upsert_character(sid, sheet)?;
        Ok(sid)
    }

    fn upsert_character(&self, sid: i64, sheet: &CharacterSheet) -> Result<(), String> {
        let doc = Self::to_doc(sid, sheet, None);

        self.with_db(|db| {
            self.rt.block_on(async {
                let content = to_surreal_value(&doc, "character")?;
                let _: Option<SurrealValue> = db
                    .upsert(("character", sid))
                    .content(content)
                    .await
                    .map_err(|e| format!("Failed to save character: {}", e))?;
                Ok(())
            })
        })
    }

    /// Upsert a character converted from legacy SQLite.
    ///
    /// Uses the legacy SQLite id as the SurrealDB record id so it stays stable,
    /// and also records it in `legacy_sqlite_id` to prevent re-imports.
    pub fn upsert_legacy_character(
        &self,
        legacy_id: i64,
        sheet: &CharacterSheet,
    ) -> Result<(), String> {
        let doc = Self::to_doc(legacy_id, sheet, Some(legacy_id));

        self.with_db(|db| {
            self.rt.block_on(async {
                let content = to_surreal_value(&doc, "legacy character")?;
                let _: Option<SurrealValue> = db
                    .upsert(("character", legacy_id))
                    .content(content)
                    .await
                    .map_err(|e| format!("Failed to save legacy character: {}", e))?;
                Ok(())
            })
        })
    }

    /// Update an existing character by ID.
    pub fn update_character(&self, id: i64, sheet: &CharacterSheet) -> Result<(), String> {
        // Ensure it exists, to preserve the old behavior.
        self.load_character(id)?;
        self.upsert_character(id, sheet)
    }

    /// Save character - creates if id is None, updates if id exists.
    pub fn save_character(&self, id: Option<i64>, sheet: &CharacterSheet) -> Result<i64, String> {
        match id {
            Some(existing_id) => {
                self.update_character(existing_id, sheet)?;
                Ok(existing_id)
            }
            None => self.create_character(sheet),
        }
    }

    /// Load a character by ID.
    pub fn load_character(&self, id: i64) -> Result<CharacterSheet, String> {
        let doc = self.with_db(|db| {
            self.rt.block_on(async {
                let record: Option<SurrealValue> = db
                    .select(("character", id))
                    .await
                    .map_err(|e| format!("Failed to load character: {}", e))?;
                Ok(record)
            })
        })?;

        let Some(value) = doc else {
            return Err(format!("Character with id {} not found", id));
        };

        let decoded: CharacterDocument = from_surreal_value(value, "character")?;
        Ok(decoded.sheet)
    }

    /// Delete a character by ID.
    pub fn delete_character(&self, id: i64) -> Result<(), String> {
        self.with_db(|db| {
            self.rt.block_on(async {
                let _: Option<SurrealValue> = db
                    .delete(("character", id))
                    .await
                    .map_err(|e| format!("Failed to delete character: {}", e))?;
                Ok(())
            })
        })
    }

    /// List all characters (for the character selection UI).
    pub fn list_characters(&self) -> Result<Vec<CharacterListEntry>, String> {
        self.with_db(|db| {
            self.rt.block_on(async {
                let mut response = db
                    .query("SELECT sid AS id, name, class, level FROM character ORDER BY name")
                    .await
                    .map_err(|e| format!("Failed to query characters: {}", e))?;
                let rows: Vec<SurrealValue> = response
                    .take(0)
                    .map_err(|e| format!("Failed to decode character list: {}", e))?;

                let mut decoded = Vec::with_capacity(rows.len());
                for row in rows {
                    decoded.push(from_surreal_value(row, "character list row")?);
                }
                Ok(decoded)
            })
        })
    }

    /// Get character count.
    pub fn character_count(&self) -> Result<i64, String> {
        Ok(self.list_characters()?.len() as i64)
    }

    /// Check if a character with the given name exists (excluding a specific ID).
    pub fn name_exists(&self, name: &str, exclude_id: Option<i64>) -> Result<bool, String> {
        let list = self.list_characters()?;
        Ok(list
            .iter()
            .any(|c| c.name == name && exclude_id.map(|id| id != c.id).unwrap_or(true)))
    }

    /// Load a document by key from the `setting` table.
    pub fn get_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        let key = key.to_owned();
        self.with_db(move |db| {
            self.rt.block_on(async {
                let record: Option<SurrealValue> = db
                    .select(("setting", key.clone()))
                    .await
                    .map_err(|e| format!("Failed to load setting '{}': {}", key, e))?;

                let Some(record) = record else {
                    return Ok(None);
                };

                // Preferred schema: store everything as JSON under a single `value` field.
                // This avoids SurrealDB-specific types (Thing/type::thing) and avoids binding issues.
                let json = surreal_value_to_json(record);
                if let JsonValue::Object(map) = &json {
                    if let Some(inner) = map.get("value") {
                        if inner.is_null() {
                            return Ok(None);
                        }
                        let decoded: T = serde_json::from_value(inner.clone()).map_err(|e| {
                            format!("Failed to decode setting '{}' as JSON: {}", key, e)
                        })?;
                        return Ok(Some(decoded));
                    }
                }

                // Back-compat: older builds stored the settings directly as the record content.
                let decoded: T = serde_json::from_value(json).map_err(|e| {
                    format!("Failed to decode legacy setting '{}' as JSON: {}", key, e)
                })?;
                Ok(Some(decoded))
            })
        })
    }

    /// Upsert a document by key into the `setting` table.
    pub fn set_setting<T: Serialize + 'static>(&self, key: &str, value: T) -> Result<(), String> {
        let key = key.to_owned();
        self.with_db(move |db| {
            self.rt.block_on(async {
                let json_value = serde_json::to_value(&value)
                    .map_err(|e| format!("Failed to serialize setting '{}' to JSON: {}", key, e))?;

                #[derive(Serialize)]
                struct SettingDoc {
                    value: JsonValue,
                }

                let content = to_surreal_value(
                    &SettingDoc { value: json_value },
                    "setting",
                )?;

                // Store under a dedicated `value` field so we can reliably load primitives
                // (bool) and complex structs (AppSettings) without SurrealDB internal types.
                let _: Option<SurrealValue> = db
                    .upsert(("setting", key.clone()))
                    .content(content)
                    .await
                    .map_err(|e| format!("Failed to save setting '{}': {}", key, e))?;
                Ok(())
            })
        })
    }

    pub fn load_command_history(&self) -> Result<Vec<String>, String> {
        #[derive(Serialize, Deserialize, Default)]
        struct Doc {
            commands: Vec<String>,
        }

        let doc: Option<SurrealValue> = self.with_db(|db| {
            self.rt.block_on(async {
                db.select(("command_history", "default"))
                    .await
                    .map_err(|e| format!("Failed to load command history: {}", e))
            })
        })?;

        match doc {
            Some(v) => {
                let decoded: Doc = from_surreal_value(v, "command history")?;
                Ok(decoded.commands)
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn save_command_history(&self, commands: &[String]) -> Result<(), String> {
        #[derive(Serialize, Deserialize)]
        struct Doc {
            commands: Vec<String>,
        }

        self.with_db(|db| {
            self.rt.block_on(async {
                let content = to_surreal_value(
                    &Doc {
                        commands: commands.to_vec(),
                    },
                    "command history",
                )?;

                let _: Option<SurrealValue> = db
                    .upsert(("command_history", "default"))
                    .content(content)
                    .await
                    .map_err(|e| format!("Failed to save command history: {}", e))?;
                Ok(())
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice3d::types::character::{Attributes, CharacterInfo, Combat};

    fn create_test_sheet(name: &str) -> CharacterSheet {
        CharacterSheet {
            character: CharacterInfo {
                name: name.to_string(),
                class: "Fighter".to_string(),
                race: "Human".to_string(),
                level: 1,
                ..Default::default()
            },
            attributes: Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            combat: Combat {
                armor_class: 10,
                initiative: 0,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_create_and_load_character() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let sheet = create_test_sheet("Gimli");

        let id = db.create_character(&sheet).unwrap();
        let loaded = db.load_character(id).unwrap();
        assert_eq!(loaded.character.name, "Gimli");
    }

    #[test]
    fn test_update_character() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let mut sheet = create_test_sheet("Thorin");

        let id = db.create_character(&sheet).unwrap();

        sheet.character.name = "Thorin Oakenshield".to_string();
        db.update_character(id, &sheet).unwrap();

        let loaded = db.load_character(id).unwrap();
        assert_eq!(loaded.character.name, "Thorin Oakenshield");
        assert_eq!(db.character_count().unwrap(), 1);
    }

    #[test]
    fn test_list_characters() {
        let db = CharacterDatabase::open_in_memory().unwrap();

        db.create_character(&create_test_sheet("Aragorn")).unwrap();
        db.create_character(&create_test_sheet("Gandalf")).unwrap();
        db.create_character(&create_test_sheet("Legolas")).unwrap();

        let list = db.list_characters().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Aragorn");
        assert_eq!(list[1].name, "Gandalf");
        assert_eq!(list[2].name, "Legolas");
    }

    #[test]
    fn test_delete_character() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let sheet = create_test_sheet("Frodo");

        let id = db.create_character(&sheet).unwrap();
        assert_eq!(db.character_count().unwrap(), 1);

        db.delete_character(id).unwrap();
        assert_eq!(db.character_count().unwrap(), 0);
    }

    #[test]
    fn test_name_exists() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let sheet = create_test_sheet("Bilbo");

        let id = db.create_character(&sheet).unwrap();

        assert!(db.name_exists("Bilbo", None).unwrap());
        assert!(!db.name_exists("Frodo", None).unwrap());
        assert!(!db.name_exists("Bilbo", Some(id)).unwrap());
    }

    #[test]
    fn test_save_character_creates_or_updates() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let mut sheet = create_test_sheet("Sam");

        let id = db.save_character(None, &sheet).unwrap();
        assert_eq!(db.character_count().unwrap(), 1);

        sheet.character.level = 5;
        let same_id = db.save_character(Some(id), &sheet).unwrap();
        assert_eq!(id, same_id);
        assert_eq!(db.character_count().unwrap(), 1);

        let loaded = db.load_character(id).unwrap();
        assert_eq!(loaded.character.level, 5);
    }
}
