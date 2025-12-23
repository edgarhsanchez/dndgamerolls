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
use std::time::{SystemTime, UNIX_EPOCH};

use surrealdb::engine::local::{Db, Mem, SurrealKv};
use surrealdb::Surreal;

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
        // Ensure the app data directory exists and is writable.
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create app data directory {:?}: {}", data_dir, e))?;

        // SurrealKV expects a directory path. If a file exists with this name (or a prior
        // incompatible store created something unexpected), back it up and recreate.
        if db_path.exists() && db_path.is_file() {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup_path = data_dir.join(format!("{}.file.bak-{}", DATABASE_FOLDER, ts));
            warn!(
                "Database path {:?} is a file; backing up to {:?} and recreating as a directory",
                db_path, backup_path
            );
            std::fs::rename(&db_path, &backup_path).map_err(|e| {
                format!(
                    "Failed to back up database file {:?} -> {:?}: {}",
                    db_path, backup_path, e
                )
            })?;
        }

        // Ensure the datastore directory exists.
        std::fs::create_dir_all(&db_path).map_err(|e| {
            format!(
                "Failed to create SurrealDB datastore dir {:?}: {}",
                db_path, e
            )
        })?;

        let db_path_str = db_path.to_string_lossy().to_string();
        let open_err_to_string = |e: surrealdb::Error| format!("Failed to open SurrealDB: {}", e);

        let db = match rt.block_on(async { Surreal::new::<SurrealKv>(db_path_str.clone()).await }) {
            Ok(db) => db,
            Err(e) => {
                // SurrealDB local stores can become incompatible across major/beta versions.
                // If we detect a recoverable datastore issue, back up the old folder and recreate.
                let err_string = open_err_to_string(e);
                let is_recoverable_datastore_issue = err_string.contains("Unsupported manifest format version")
                    || err_string.contains("Failed to load manifest")
                    // Common corruption / partial-write signatures (seen on Windows).
                    || err_string.contains("unexpected end of file")
                    || err_string.contains("failed to fill whole buffer")
                    || err_string.contains("There was a problem with the underlying datastore");

                if is_recoverable_datastore_issue && db_path.exists() {
                    let ts = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let backup_path = data_dir.join(format!("{}.bak-{}", DATABASE_FOLDER, ts));

                    warn!(
                        "SurrealDB datastore appears unreadable at {:?} ({}). Backing up to {:?} and recreating.",
                        db_path,
                        err_string,
                        backup_path
                    );

                    std::fs::rename(&db_path, &backup_path).map_err(|re| {
                        format!(
                            "{} (also failed to back up {:?} -> {:?}: {})",
                            err_string, db_path, backup_path, re
                        )
                    })?;

                    // Recreate the directory after backup.
                    std::fs::create_dir_all(&db_path).map_err(|ce| {
                        format!(
                            "{} (also failed to recreate datastore dir {:?}: {})",
                            err_string, db_path, ce
                        )
                    })?;

                    rt.block_on(async { Surreal::new::<SurrealKv>(db_path_str.clone()).await })
                        .map_err(open_err_to_string)?
                } else {
                    return Err(err_string);
                }
            }
        };

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
                let record: Option<JsonValue> = db
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
        // SurrealKV expects a directory path.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database folder {:?}: {}", parent, e))?;
        }
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create SurrealDB datastore dir {:?}: {}", path, e))?;

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
                let _: Option<CharacterDocument> = db
                    .upsert(("character", sid))
                    .content(doc)
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
                let _: Option<CharacterDocument> = db
                    .upsert(("character", legacy_id))
                    .content(doc)
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
        let doc: Option<CharacterDocument> = self.with_db(|db| {
            self.rt.block_on(async {
                db.select(("character", id))
                    .await
                    .map_err(|e| format!("Failed to load character: {}", e))
            })
        })?;

        let Some(decoded) = doc else {
            return Err(format!("Character with id {} not found", id));
        };

        Ok(decoded.sheet)
    }

    /// Delete a character by ID.
    pub fn delete_character(&self, id: i64) -> Result<(), String> {
        self.with_db(|db| {
            self.rt.block_on(async {
                let _: Option<CharacterDocument> = db
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
                response
                    .take::<Vec<CharacterListEntry>>(0)
                    .map_err(|e| format!("Failed to decode character list: {}", e))
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
                #[derive(Deserialize)]
                struct SettingRecord {
                    value: String,
                }

                // Preferred schema: store settings as a JSON string under a single `value` field.
                match db
                    .select::<Option<SettingRecord>>(("setting", key.clone()))
                    .await
                {
                    Ok(Some(record)) => {
                        let decoded: T = serde_json::from_str(&record.value).map_err(|e| {
                            format!("Failed to decode setting '{}' from JSON string: {}", key, e)
                        })?;
                        Ok(Some(decoded))
                    }
                    Ok(None) => Ok(None),
                    Err(_) => {
                        // Back-compat: older builds stored the settings directly as the record content.
                        let legacy: Option<T> =
                            db.select(("setting", key.clone())).await.map_err(|e| {
                                format!("Failed to load legacy setting '{}': {}", key, e)
                            })?;
                        Ok(legacy)
                    }
                }
            })
        })
    }

    /// Upsert a document by key into the `setting` table.
    pub fn set_setting<T: Serialize + 'static>(&self, key: &str, value: T) -> Result<(), String> {
        let key = key.to_owned();
        self.with_db(move |db| {
            self.rt.block_on(async {
                // Store settings as a JSON string. This avoids SurrealDB local-engine
                // serialization edge cases and keeps the schema stable across versions.
                let json_string = serde_json::to_string(&value).map_err(|e| {
                    format!(
                        "Failed to serialize setting '{}' to JSON string: {}",
                        key, e
                    )
                })?;

                #[derive(Serialize)]
                struct SettingDoc<T> {
                    value: T,
                }

                // Store under a dedicated `value` field so we can reliably load primitives
                // (bool) and complex structs (AppSettings) without SurrealDB internal types.
                #[derive(Deserialize)]
                struct SettingSaved {
                    #[allow(dead_code)]
                    value: String,
                }

                let _: Option<SettingSaved> = db
                    .upsert(("setting", key.clone()))
                    .content(SettingDoc { value: json_string })
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

        let doc: Option<Doc> = self.with_db(|db| {
            self.rt.block_on(async {
                db.select(("command_history", "default"))
                    .await
                    .map_err(|e| format!("Failed to load command history: {}", e))
            })
        })?;

        Ok(doc.unwrap_or_default().commands)
    }

    pub fn save_command_history(&self, commands: &[String]) -> Result<(), String> {
        #[derive(Serialize, Deserialize)]
        struct Doc {
            commands: Vec<String>,
        }

        self.with_db(|db| {
            self.rt.block_on(async {
                let _: Option<JsonValue> = db
                    .upsert(("command_history", "default"))
                    .content(Doc {
                        commands: commands.to_vec(),
                    })
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
    use crate::dice3d::types::settings::{AppSettings, ColorSetting};

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

    #[test]
    fn test_settings_round_trip_includes_background_color() {
        fn approx_eq(a: f32, b: f32) -> bool {
            (a - b).abs() < 1e-6
        }

        let db = CharacterDatabase::open_in_memory().unwrap();

        let settings = AppSettings {
            background_color: ColorSetting {
                a: 0.75,
                r: 0.12,
                g: 0.34,
                b: 0.56,
            },
            ..Default::default()
        };

        db.set_setting("app_settings", settings.clone()).unwrap();
        let loaded: AppSettings = db.get_setting("app_settings").unwrap().unwrap();

        assert!(approx_eq(
            loaded.background_color.a,
            settings.background_color.a
        ));
        assert!(approx_eq(
            loaded.background_color.r,
            settings.background_color.r
        ));
        assert!(approx_eq(
            loaded.background_color.g,
            settings.background_color.g
        ));
        assert!(approx_eq(
            loaded.background_color.b,
            settings.background_color.b
        ));
    }

    #[test]
    fn test_settings_persist_to_disk_round_trip() {
        // Use a unique folder under the OS temp dir.
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("dndgamerolls-test-{}", ts));

        // First run: save.
        {
            let db = CharacterDatabase::open_at(path.clone()).unwrap();
            let settings = AppSettings {
                background_color: ColorSetting {
                    a: 0.5,
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                },
                ..Default::default()
            };
            db.set_setting("app_settings", settings.clone()).unwrap();
        }

        // Second run: reload.
        {
            let db = CharacterDatabase::open_at(path.clone()).unwrap();
            let loaded: AppSettings = db.get_setting("app_settings").unwrap().unwrap();
            assert!((loaded.background_color.a - 0.5).abs() < 1e-6);
            assert!((loaded.background_color.r - 0.1).abs() < 1e-6);
            assert!((loaded.background_color.g - 0.2).abs() < 1e-6);
            assert!((loaded.background_color.b - 0.3).abs() < 1e-6);
        }

        // Best-effort cleanup.
        let _ = std::fs::remove_dir_all(&path);
    }
}
