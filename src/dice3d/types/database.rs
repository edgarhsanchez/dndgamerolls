//! SQLite database for character storage
//!
//! This module provides persistent storage for characters using SQLite.
//! Each character has a stable internal ID that doesn't change when renamed.

use bevy::prelude::*;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

use super::character::{CharacterListEntry, CharacterSheet};

/// Database file name
const DATABASE_FILE: &str = "characters.db";
/// App data folder name
const APP_DATA_FOLDER: &str = "DnDGameRolls";

/// A character entry in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterEntry {
    /// Stable internal ID (never changes)
    pub id: i64,
    /// Character name (can be changed)
    pub name: String,
    /// Character class for display
    pub class: String,
    /// Character race for display  
    pub race: String,
    /// Character level for display
    pub level: i32,
    /// Full character sheet data as JSON
    pub data: String,
}

/// Resource for managing the character database
#[derive(Resource)]
pub struct CharacterDatabase {
    /// Database connection wrapped in Mutex for thread safety
    connection: Mutex<Connection>,
    /// Path to the database file
    pub db_path: PathBuf,
}

impl CharacterDatabase {
    /// Get the app data directory for storing the database
    /// Uses LocalAppData on Windows, which is accessible to MSIX apps
    fn get_data_dir() -> Result<PathBuf, String> {
        // Try to get the user's local app data directory
        // On Windows: C:\Users\<user>\AppData\Local\DnDGameRolls
        // On Linux: ~/.local/share/DnDGameRolls
        // On macOS: ~/Library/Application Support/DnDGameRolls

        #[cfg(target_os = "windows")]
        {
            // Try LOCALAPPDATA first (standard Windows location)
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let app_dir = PathBuf::from(&local_app_data).join(APP_DATA_FOLDER);
                println!("Attempting to use app data directory: {:?}", app_dir);
                match std::fs::create_dir_all(&app_dir) {
                    Ok(_) => {
                        println!(
                            "Successfully created/verified app data directory: {:?}",
                            app_dir
                        );
                        return Ok(app_dir);
                    }
                    Err(e) => {
                        eprintln!("Failed to create app data directory {:?}: {}", app_dir, e);
                        // Fall through to try other options
                    }
                }
            } else {
                eprintln!("LOCALAPPDATA environment variable not set");
            }

            // Fallback: try user profile directory
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                let app_dir = PathBuf::from(&user_profile)
                    .join("AppData")
                    .join("Local")
                    .join(APP_DATA_FOLDER);
                println!("Trying fallback user profile path: {:?}", app_dir);
                match std::fs::create_dir_all(&app_dir) {
                    Ok(_) => {
                        println!(
                            "Successfully created/verified fallback directory: {:?}",
                            app_dir
                        );
                        return Ok(app_dir);
                    }
                    Err(e) => {
                        eprintln!("Failed to create fallback directory {:?}: {}", app_dir, e);
                    }
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

        // Fallback to current directory
        Ok(std::env::current_dir().unwrap_or_default())
    }

    /// Open or create the database
    pub fn open() -> Result<Self, String> {
        let data_dir = Self::get_data_dir()?;
        let db_path = data_dir.join(DATABASE_FILE);

        println!("Opening database at: {:?}", db_path);

        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

        let db = Self {
            connection: Mutex::new(conn),
            db_path,
        };

        db.initialize_schema()?;

        Ok(db)
    }

    /// Open database at a specific path (for testing)
    pub fn open_at(path: PathBuf) -> Result<Self, String> {
        let conn =
            Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;

        let db = Self {
            connection: Mutex::new(conn),
            db_path: path,
        };

        db.initialize_schema()?;

        Ok(db)
    }

    /// Open an in-memory database (for testing or fallback)
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory database: {}", e))?;

        let db = Self {
            connection: Mutex::new(conn),
            db_path: PathBuf::from(":memory:"),
        };

        db.initialize_schema()?;

        Ok(db)
    }

    /// Create database tables if they don't exist
    fn initialize_schema(&self) -> Result<(), String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                class TEXT NOT NULL DEFAULT '',
                race TEXT NOT NULL DEFAULT '',
                level INTEGER NOT NULL DEFAULT 1,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_characters_name ON characters(name);

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )
        .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

        Ok(())
    }

    /// Get a JSON blob from the app settings table.
    pub fn get_setting_json(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT json FROM app_settings WHERE key = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let result: Result<String, _> = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(json) => Ok(Some(json)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to load setting '{}': {}", key, e)),
        }
    }

    /// Upsert a JSON blob into the app settings table.
    pub fn set_setting_json(&self, key: &str, json: &str) -> Result<(), String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "
            INSERT INTO app_settings (key, json, updated_at)
            VALUES (?1, ?2, CURRENT_TIMESTAMP)
            ON CONFLICT(key) DO UPDATE SET
                json = excluded.json,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![key, json],
        )
        .map_err(|e| format!("Failed to save setting '{}': {}", key, e))?;

        Ok(())
    }

    /// Save a new character and return its ID
    pub fn create_character(&self, sheet: &CharacterSheet) -> Result<i64, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        let data = serde_json::to_string(sheet)
            .map_err(|e| format!("Failed to serialize character: {}", e))?;

        conn.execute(
            "INSERT INTO characters (name, class, race, level, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                sheet.character.name,
                sheet.character.class,
                sheet.character.race,
                sheet.character.level,
                data
            ],
        )
        .map_err(|e| format!("Failed to create character: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// Update an existing character by ID
    pub fn update_character(&self, id: i64, sheet: &CharacterSheet) -> Result<(), String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        let data = serde_json::to_string(sheet)
            .map_err(|e| format!("Failed to serialize character: {}", e))?;

        let rows_updated = conn.execute(
            "UPDATE characters SET name = ?1, class = ?2, race = ?3, level = ?4, data = ?5, updated_at = CURRENT_TIMESTAMP WHERE id = ?6",
            params![
                sheet.character.name,
                sheet.character.class,
                sheet.character.race,
                sheet.character.level,
                data,
                id
            ],
        ).map_err(|e| format!("Failed to update character: {}", e))?;

        if rows_updated == 0 {
            return Err(format!("Character with id {} not found", id));
        }

        Ok(())
    }

    /// Save character - creates if id is None, updates if id exists
    pub fn save_character(&self, id: Option<i64>, sheet: &CharacterSheet) -> Result<i64, String> {
        match id {
            Some(existing_id) => {
                self.update_character(existing_id, sheet)?;
                Ok(existing_id)
            }
            None => self.create_character(sheet),
        }
    }

    /// Load a character by ID
    pub fn load_character(&self, id: i64) -> Result<CharacterSheet, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        let data: String = conn
            .query_row(
                "SELECT data FROM characters WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Character not found: {}", e))?;

        serde_json::from_str(&data).map_err(|e| format!("Failed to deserialize character: {}", e))
    }

    /// Delete a character by ID
    pub fn delete_character(&self, id: i64) -> Result<(), String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM characters WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete character: {}", e))?;

        Ok(())
    }

    /// List all characters (for the character selection UI)
    pub fn list_characters(&self) -> Result<Vec<CharacterListEntry>, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, name, class, level FROM characters ORDER BY name")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(CharacterListEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    class: row.get(2)?,
                    level: row.get(3)?,
                })
            })
            .map_err(|e| format!("Failed to query characters: {}", e))?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry.map_err(|e| format!("Failed to read character: {}", e))?);
        }

        Ok(result)
    }

    /// Get character count
    pub fn character_count(&self) -> Result<i64, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        conn.query_row("SELECT COUNT(*) FROM characters", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count characters: {}", e))
    }

    /// Check if a character with the given name exists (excluding a specific ID)
    pub fn name_exists(&self, name: &str, exclude_id: Option<i64>) -> Result<bool, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        let count: i64 = match exclude_id {
            Some(id) => conn.query_row(
                "SELECT COUNT(*) FROM characters WHERE name = ?1 AND id != ?2",
                params![name, id],
                |row| row.get(0),
            ),
            None => conn.query_row(
                "SELECT COUNT(*) FROM characters WHERE name = ?1",
                params![name],
                |row| row.get(0),
            ),
        }
        .map_err(|e| format!("Failed to check name: {}", e))?;

        Ok(count > 0)
    }

    /// Import a character from JSON (for migration from file-based storage)
    pub fn import_from_json(&self, json_str: &str) -> Result<i64, String> {
        let sheet: CharacterSheet =
            serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Check if character with same name already exists
        if self.name_exists(&sheet.character.name, None)? {
            return Err(format!(
                "Character '{}' already exists",
                sheet.character.name
            ));
        }

        self.create_character(&sheet)
    }

    /// Import a character from a JSON file path (for migration from file-based storage)
    pub fn import_from_file(&self, path: &std::path::Path) -> Result<i64, String> {
        let json_str =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        self.import_from_json(&json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice3d::types::character::{AttributeModifiers, Attributes, CharacterInfo, Combat};
    use std::collections::HashMap;

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
            modifiers: AttributeModifiers::default(),
            combat: Combat::default(),
            proficiency_bonus: 2,
            saving_throws: HashMap::new(),
            skills: HashMap::new(),
            equipment: None,
            features: Vec::new(),
            spells: None,
            custom_attributes: HashMap::new(),
            custom_basic_info: HashMap::new(),
            custom_combat: HashMap::new(),
        }
    }

    #[test]
    fn test_create_and_load_character() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let sheet = create_test_sheet("Thorin");

        let id = db.create_character(&sheet).unwrap();
        assert!(id > 0);

        let loaded = db.load_character(id).unwrap();
        assert_eq!(loaded.character.name, "Thorin");
    }

    #[test]
    fn test_update_character_name() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let mut sheet = create_test_sheet("Thorin");

        let id = db.create_character(&sheet).unwrap();

        // Rename the character
        sheet.character.name = "Thorin Oakenshield".to_string();
        db.update_character(id, &sheet).unwrap();

        // Verify the update - same ID, new name
        let loaded = db.load_character(id).unwrap();
        assert_eq!(loaded.character.name, "Thorin Oakenshield");

        // Verify only one character exists
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

        // Should be sorted by name
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

        // Should not count itself when checking
        assert!(!db.name_exists("Bilbo", Some(id)).unwrap());
    }

    #[test]
    fn test_save_character_creates_or_updates() {
        let db = CharacterDatabase::open_in_memory().unwrap();
        let mut sheet = create_test_sheet("Sam");

        // First save creates
        let id = db.save_character(None, &sheet).unwrap();
        assert_eq!(db.character_count().unwrap(), 1);

        // Second save with ID updates
        sheet.character.level = 5;
        let same_id = db.save_character(Some(id), &sheet).unwrap();
        assert_eq!(id, same_id);
        assert_eq!(db.character_count().unwrap(), 1);

        let loaded = db.load_character(id).unwrap();
        assert_eq!(loaded.character.level, 5);
    }
}
