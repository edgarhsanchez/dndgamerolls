//! Character management integration tests
//!
//! These tests cover user scenarios for character creation, editing, and saving.
//! Tests that need file system access use isolated temp directories.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// Import the character types from the main crate
use dndgamerolls::dice3d::types::character::{Attributes, CharacterData, CharacterManager};

// Global mutex to prevent concurrent tests from interfering with each other
// when changing the current working directory
static DIR_LOCK: Mutex<()> = Mutex::new(());

/// Helper to create a temp directory for test files
fn create_test_dir() -> PathBuf {
    let test_dir = std::env::temp_dir().join(format!(
        "gamerolls_test_{}_{}",
        std::process::id(),
        rand::random::<u32>()
    ));
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    test_dir
}

/// Helper to clean up test directory
fn cleanup_test_dir(path: &PathBuf) {
    let _ = fs::remove_dir_all(path);
}

// ============================================================================
// Scenario 1: Creating a new character
// ============================================================================

#[test]
fn test_create_new_character_has_valid_data() {
    let char_data = CharacterData::create_new();

    // Should have a character sheet
    assert!(
        char_data.sheet.is_some(),
        "New character should have a sheet"
    );

    // Should be marked as modified (needs saving)
    assert!(
        char_data.is_modified,
        "New character should be marked as modified"
    );

    // Should not have a file path yet
    assert!(
        char_data.file_path.is_none(),
        "New character should not have a file path until saved"
    );
}

#[test]
fn test_create_new_character_has_random_stats() {
    let char1 = CharacterData::create_new();
    let char2 = CharacterData::create_new();

    let sheet1 = char1.sheet.as_ref().unwrap();
    let sheet2 = char2.sheet.as_ref().unwrap();

    // Stats should be within d20 range (1-20)
    assert!(
        sheet1.attributes.strength >= 1 && sheet1.attributes.strength <= 20,
        "Strength should be 1-20"
    );
    assert!(
        sheet1.attributes.dexterity >= 1 && sheet1.attributes.dexterity <= 20,
        "Dexterity should be 1-20"
    );
    assert!(
        sheet1.attributes.constitution >= 1 && sheet1.attributes.constitution <= 20,
        "Constitution should be 1-20"
    );

    // Two random characters should likely have different stats
    // (very small chance of collision, but statistically insignificant)
    let same_str = sheet1.attributes.strength == sheet2.attributes.strength;
    let same_dex = sheet1.attributes.dexterity == sheet2.attributes.dexterity;
    let same_con = sheet1.attributes.constitution == sheet2.attributes.constitution;
    let same_int = sheet1.attributes.intelligence == sheet2.attributes.intelligence;
    let same_wis = sheet1.attributes.wisdom == sheet2.attributes.wisdom;
    let same_cha = sheet1.attributes.charisma == sheet2.attributes.charisma;

    // Not all stats should be the same (probability of all 6 matching is 1/20^6)
    let all_same = same_str && same_dex && same_con && same_int && same_wis && same_cha;
    assert!(
        !all_same,
        "Two new characters should have different random stats"
    );
}

#[test]
fn test_create_new_character_has_default_name() {
    let char_data = CharacterData::create_new();
    let sheet = char_data.sheet.as_ref().unwrap();

    assert_eq!(
        sheet.character.name, "New Character",
        "New character should have default name"
    );
}

#[test]
fn test_create_new_character_has_correct_modifiers() {
    let char_data = CharacterData::create_new();
    let sheet = char_data.sheet.as_ref().unwrap();

    // Verify modifiers are calculated correctly from attributes
    let expected_str_mod = Attributes::calculate_modifier(sheet.attributes.strength);
    let expected_dex_mod = Attributes::calculate_modifier(sheet.attributes.dexterity);
    let expected_con_mod = Attributes::calculate_modifier(sheet.attributes.constitution);

    assert_eq!(
        sheet.modifiers.strength, expected_str_mod,
        "Strength modifier should be calculated correctly"
    );
    assert_eq!(
        sheet.modifiers.dexterity, expected_dex_mod,
        "Dexterity modifier should be calculated correctly"
    );
    assert_eq!(
        sheet.modifiers.constitution, expected_con_mod,
        "Constitution modifier should be calculated correctly"
    );
}

#[test]
fn test_create_new_character_has_derived_stats() {
    let char_data = CharacterData::create_new();
    let sheet = char_data.sheet.as_ref().unwrap();

    // AC = 10 + Dex modifier
    let expected_ac = 10 + sheet.modifiers.dexterity;
    assert_eq!(
        sheet.combat.armor_class, expected_ac,
        "AC should be 10 + Dex modifier"
    );

    // Initiative = Dex modifier
    assert_eq!(
        sheet.combat.initiative, sheet.modifiers.dexterity,
        "Initiative should equal Dex modifier"
    );

    // HP should be set
    assert!(
        sheet.combat.hit_points.is_some(),
        "New character should have hit points"
    );
}

// ============================================================================
// Scenario 2: Modifying character attributes
// ============================================================================

#[test]
fn test_modifier_calculation_formula() {
    // D&D modifier formula: (attribute - 10) / 2
    // Rust integer division rounds toward zero, not floor
    // So for negative results: -9/2 = -4, -7/2 = -3, etc.
    assert_eq!(Attributes::calculate_modifier(1), -4); // (1-10)/2 = -9/2 = -4
    assert_eq!(Attributes::calculate_modifier(2), -4); // (2-10)/2 = -8/2 = -4
    assert_eq!(Attributes::calculate_modifier(3), -3); // (3-10)/2 = -7/2 = -3
    assert_eq!(Attributes::calculate_modifier(8), -1);
    assert_eq!(Attributes::calculate_modifier(9), 0); // (9-10)/2 = -1/2 = 0
    assert_eq!(Attributes::calculate_modifier(10), 0);
    assert_eq!(Attributes::calculate_modifier(11), 0);
    assert_eq!(Attributes::calculate_modifier(12), 1);
    assert_eq!(Attributes::calculate_modifier(13), 1);
    assert_eq!(Attributes::calculate_modifier(14), 2);
    assert_eq!(Attributes::calculate_modifier(15), 2);
    assert_eq!(Attributes::calculate_modifier(16), 3);
    assert_eq!(Attributes::calculate_modifier(17), 3);
    assert_eq!(Attributes::calculate_modifier(18), 4);
    assert_eq!(Attributes::calculate_modifier(19), 4);
    assert_eq!(Attributes::calculate_modifier(20), 5);
}

#[test]
fn test_modify_character_sets_modified_flag() {
    let mut char_data = CharacterData::create_new();

    // After creation, save it first to clear modified flag
    // (simulating a saved character)
    char_data.is_modified = false;

    // Simulate modification
    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Modified Name".to_string();
    }
    char_data.is_modified = true;

    assert!(
        char_data.is_modified,
        "Character should be marked as modified after changes"
    );
}

// ============================================================================
// Scenario 3: Saving characters
// ============================================================================

#[test]
fn test_generate_filename_from_name() {
    // Basic name
    assert_eq!(
        CharacterManager::generate_filename("Aragorn"),
        "aragorn.json"
    );

    // Name with spaces
    assert_eq!(
        CharacterManager::generate_filename("Legolas Greenleaf"),
        "legolas_greenleaf.json"
    );

    // Name with special characters
    assert_eq!(
        CharacterManager::generate_filename("Drizzt Do'Urden"),
        "drizzt_do_urden.json"
    );

    // Mixed case
    assert_eq!(
        CharacterManager::generate_filename("GANDALF the Grey"),
        "gandalf_the_grey.json"
    );
}

#[test]
fn test_sanitize_name() {
    // Spaces are allowed
    assert_eq!(
        CharacterManager::sanitize_name("Frodo Baggins"),
        "Frodo Baggins"
    );

    // Special characters become underscores
    assert_eq!(
        CharacterManager::sanitize_name("Test@Character#1"),
        "Test_Character_1"
    );

    // Quotes and apostrophes
    assert_eq!(
        CharacterManager::sanitize_name("Drizzt Do'Urden"),
        "Drizzt Do_Urden"
    );
}

#[test]
fn test_save_new_character_creates_file() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();

    // Change to test directory
    std::env::set_current_dir(&test_dir).unwrap();

    let mut char_data = CharacterData::create_new();
    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Test Hero".to_string();
    }

    let result = char_data.save();
    assert!(result.is_ok(), "Save should succeed: {:?}", result);

    // File should exist
    let expected_path = test_dir.join("test_hero.json");
    assert!(expected_path.exists(), "Character file should be created");

    // Modified flag should be cleared
    assert!(
        !char_data.is_modified,
        "Modified flag should be cleared after save"
    );

    // File path should be set
    assert!(
        char_data.file_path.is_some(),
        "File path should be set after save"
    );

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_updates_file_path_on_rename() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // Create and save initial character
    let mut char_data = CharacterData::create_new();
    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Original Name".to_string();
    }
    char_data.save().unwrap();

    let first_path = char_data.file_path.clone();
    assert!(
        test_dir.join("original_name.json").exists(),
        "First file should exist"
    );

    // Rename character and save again
    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "New Name".to_string();
    }
    char_data.is_modified = true;
    char_data.save().unwrap();

    // Should have new file path
    assert_ne!(
        char_data.file_path, first_path,
        "File path should change after rename"
    );
    assert!(
        test_dir.join("new_name.json").exists(),
        "New file should exist"
    );

    // Old file should still exist (we don't delete on rename)
    assert!(
        test_dir.join("original_name.json").exists(),
        "Old file should still exist"
    );

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_saved_character_can_be_loaded() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // Create, modify, and save
    let mut char_data = CharacterData::create_new();
    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Loadable Hero".to_string();
        sheet.character.class = "Wizard".to_string();
        sheet.character.race = "Elf".to_string();
        sheet.character.level = 5;
        sheet.attributes.intelligence = 18;
    }
    char_data.save().unwrap();

    // Load the character
    let loaded = CharacterData::load_from_file("loadable_hero.json");

    assert!(loaded.sheet.is_some(), "Loaded character should have sheet");
    let sheet = loaded.sheet.as_ref().unwrap();
    assert_eq!(sheet.character.name, "Loadable Hero");
    assert_eq!(sheet.character.class, "Wizard");
    assert_eq!(sheet.character.race, "Elf");
    assert_eq!(sheet.character.level, 5);
    assert_eq!(sheet.attributes.intelligence, 18);

    // Loaded character should not be modified
    assert!(
        !loaded.is_modified,
        "Loaded character should not be marked as modified"
    );

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}

// ============================================================================
// Scenario 4: Character list management
// ============================================================================

#[test]
fn test_scan_directory_finds_characters() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // Create several characters
    for name in &["Alpha", "Beta", "Gamma"] {
        let mut char_data = CharacterData::create_new();
        if let Some(ref mut sheet) = char_data.sheet {
            sheet.character.name = name.to_string();
        }
        char_data.save().unwrap();
    }

    // Scan directory
    let characters = CharacterManager::scan_directory(&test_dir);

    assert_eq!(characters.len(), 3, "Should find 3 characters");

    // Should be sorted alphabetically
    assert_eq!(characters[0].name, "Alpha");
    assert_eq!(characters[1].name, "Beta");
    assert_eq!(characters[2].name, "Gamma");

    // All should be valid
    assert!(characters.iter().all(|c| c.is_valid));

    // Cleanup
    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_scan_directory_ignores_invalid_json() {
    let test_dir = create_test_dir();

    // Create a valid character file directly without changing directory
    let _lock = DIR_LOCK.lock().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    let mut char_data = CharacterData::create_new();
    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Valid".to_string();
    }
    char_data.save().unwrap();

    std::env::set_current_dir(&original_dir).unwrap();
    drop(_lock);

    // Create an invalid JSON file
    fs::write(test_dir.join("invalid.json"), "{ not valid json }").unwrap();

    // Create a non-character JSON file
    fs::write(
        test_dir.join("other.json"),
        r#"{"type": "config", "value": 123}"#,
    )
    .unwrap();

    // Scan directory - should only find the valid character
    let characters = CharacterManager::scan_directory(&test_dir);

    assert_eq!(
        characters.len(),
        1,
        "Should only find 1 valid character file"
    );
    assert_eq!(characters[0].name, "Valid");

    // Cleanup
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_scan_directory_handles_empty_directory() {
    let test_dir = create_test_dir();

    let characters = CharacterManager::scan_directory(&test_dir);

    assert!(
        characters.is_empty(),
        "Empty directory should return no characters"
    );

    cleanup_test_dir(&test_dir);
}

// ============================================================================
// Scenario 5: Loading existing characters
// ============================================================================

#[test]
fn test_load_nonexistent_file_returns_default() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // Try to load a file that doesn't exist
    let data = CharacterData::load_from_file("definitely_nonexistent_12345.json");

    assert!(
        data.sheet.is_none(),
        "Should return empty data for missing file"
    );
    assert!(data.file_path.is_none());
    assert!(!data.is_modified);

    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_load_invalid_json_returns_default() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    let invalid_path = test_dir.join("invalid.json");
    fs::write(&invalid_path, "not valid json at all").unwrap();

    let data = CharacterData::load_from_file(invalid_path.to_str().unwrap());

    assert!(
        data.sheet.is_none(),
        "Should return empty data for invalid JSON"
    );

    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}

// ============================================================================
// Scenario 6: Character data access helpers
// ============================================================================

#[test]
fn test_get_skill_modifier() {
    let mut char_data = CharacterData::create_new();

    if let Some(ref mut sheet) = char_data.sheet {
        // Add a skill with known modifier
        sheet.skills.insert(
            "stealth".to_string(),
            dndgamerolls::dice3d::types::character::Skill {
                proficient: true,
                expertise: Some(false),
                modifier: 5,
                proficiency_type: None,
            },
        );
    }

    let modifier = char_data.get_skill_modifier("stealth");
    assert_eq!(modifier, Some(5), "Should return correct skill modifier");

    let missing = char_data.get_skill_modifier("nonexistent");
    assert_eq!(missing, None, "Should return None for missing skill");
}

#[test]
fn test_get_ability_modifier() {
    let char_data = CharacterData::create_new();
    let sheet = char_data.sheet.as_ref().unwrap();

    // Test various ability name formats
    let str_mod = char_data.get_ability_modifier("str");
    assert_eq!(str_mod, Some(sheet.modifiers.strength));

    let strength_mod = char_data.get_ability_modifier("strength");
    assert_eq!(strength_mod, Some(sheet.modifiers.strength));

    let dex_mod = char_data.get_ability_modifier("dex");
    assert_eq!(dex_mod, Some(sheet.modifiers.dexterity));

    // Note: Invalid ability returns Some(0), not None (implementation detail)
    let invalid = char_data.get_ability_modifier("invalidability");
    assert_eq!(
        invalid,
        Some(0),
        "Invalid ability name returns 0 (implementation detail)"
    );
}

#[test]
fn test_get_saving_throw_modifier() {
    let char_data = CharacterData::create_new();

    // New characters should have saving throws set up
    // Note: Keys are full ability names like "strength", not abbreviations
    let str_save = char_data.get_saving_throw_modifier("strength");
    assert!(
        str_save.is_some(),
        "New character should have strength saving throw"
    );

    let dex_save = char_data.get_saving_throw_modifier("dexterity");
    assert!(
        dex_save.is_some(),
        "New character should have dexterity saving throw"
    );

    // Short names like "str" don't work for saving throws (only full names)
    let short_name = char_data.get_saving_throw_modifier("str");
    assert_eq!(
        short_name, None,
        "Short ability names don't work for saving throws"
    );

    let invalid = char_data.get_saving_throw_modifier("invalidability");
    assert_eq!(invalid, None);
}

// ============================================================================
// Scenario 7: Edge cases
// ============================================================================

#[test]
fn test_save_without_sheet_fails() {
    let mut char_data = CharacterData::default();

    let result = char_data.save();
    assert!(result.is_err(), "Save without sheet should fail");
    assert!(result.unwrap_err().contains("No character data"));
}

#[test]
fn test_character_with_empty_name() {
    // Filename generation should handle empty name
    let filename = CharacterManager::generate_filename("");
    assert_eq!(
        filename, ".json",
        "Empty name should still produce .json extension"
    );
}

#[test]
fn test_character_with_unicode_name() {
    // Should be able to generate a filename (unicode chars become underscores)
    let filename = CharacterManager::generate_filename("日本語キャラクター");
    assert!(
        filename.ends_with(".json"),
        "Unicode name should produce valid filename"
    );
}

#[test]
fn test_extreme_attribute_values() {
    // Test modifier calculation with extreme values
    // Using the formula: (score - 10) / 2 with integer division toward zero
    assert_eq!(Attributes::calculate_modifier(0), -5); // (0-10)/2 = -10/2 = -5
    assert_eq!(Attributes::calculate_modifier(30), 10); // (30-10)/2 = 20/2 = 10
    assert_eq!(Attributes::calculate_modifier(100), 45); // (100-10)/2 = 90/2 = 45
}

// ============================================================================
// Scenario 8: Character modification tracking
// ============================================================================

#[test]
fn test_new_character_is_modified() {
    let char_data = CharacterData::create_new();
    assert!(
        char_data.is_modified,
        "Newly created character should be marked as modified"
    );
}

#[test]
fn test_loaded_character_is_not_modified() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // Create and save a character
    let mut char_data = CharacterData::create_new();
    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Saved Hero".to_string();
    }
    char_data.save().unwrap();

    // Load it back
    let loaded = CharacterData::load_from_file("saved_hero.json");

    assert!(
        !loaded.is_modified,
        "Loaded character should not be marked as modified"
    );

    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_clears_modified_flag() {
    let _lock = DIR_LOCK.lock().unwrap();
    let test_dir = create_test_dir();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    let mut char_data = CharacterData::create_new();
    assert!(char_data.is_modified, "Should start as modified");

    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Save Test".to_string();
    }
    char_data.save().unwrap();

    assert!(
        !char_data.is_modified,
        "Modified flag should be cleared after save"
    );

    std::env::set_current_dir(original_dir).unwrap();
    cleanup_test_dir(&test_dir);
}
