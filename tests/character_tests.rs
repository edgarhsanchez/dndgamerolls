//! Character management integration tests (SQLite-backed)
//!
//! These tests validate that character data persists via SQLite only.

use dndgamerolls::dice3d::types::character::{
    Attributes, CharacterData, CharacterSheet, CustomIntField, CustomStringField, SavingThrow, Skill,
};
use std::collections::HashMap;
use dndgamerolls::dice3d::types::database::CharacterDatabase;

#[test]
fn test_create_new_character_has_valid_data() {
    let char_data = CharacterData::create_new();

    assert!(
        char_data.sheet.is_some(),
        "New character should have a sheet"
    );
    assert!(char_data.is_modified, "New character should be modified");
    assert!(
        char_data.character_id.is_none(),
        "New character should not have a DB id"
    );
}

#[test]
fn test_modifier_calculation_formula() {
    // D&D modifier formula: (attribute - 10) / 2
    // Rust integer division rounds toward zero.
    assert_eq!(Attributes::calculate_modifier(1), -4);
    assert_eq!(Attributes::calculate_modifier(3), -3);
    assert_eq!(Attributes::calculate_modifier(8), -1);
    assert_eq!(Attributes::calculate_modifier(9), 0);
    assert_eq!(Attributes::calculate_modifier(10), 0);
    assert_eq!(Attributes::calculate_modifier(12), 1);
    assert_eq!(Attributes::calculate_modifier(20), 5);
}

#[test]
fn test_save_and_load_character_via_sqlite() {
    let db = CharacterDatabase::open_in_memory().expect("open in-memory db");

    let mut char_data = CharacterData::create_new();
    {
        let sheet = char_data.sheet.as_mut().unwrap();
        sheet.character.name = "Loadable Hero".to_string();
        sheet.character.class = "Wizard".to_string();
        sheet.character.race = "Elf".to_string();
        sheet.character.level = 5;
        sheet.attributes.intelligence = 18;
    }

    char_data.save_to_database(&db).expect("save to db");
    let id = char_data.character_id.expect("db id");
    assert!(!char_data.is_modified);

    let loaded = CharacterData::load_from_database(&db, id).expect("load from db");
    let sheet = loaded.sheet.as_ref().unwrap();
    assert_eq!(sheet.character.name, "Loadable Hero");
    assert_eq!(sheet.character.class, "Wizard");
    assert_eq!(sheet.character.race, "Elf");
    assert_eq!(sheet.character.level, 5);
    assert_eq!(sheet.attributes.intelligence, 18);
}

#[test]
fn test_list_characters_from_sqlite() {
    let db = CharacterDatabase::open_in_memory().expect("open in-memory db");

    for name in ["Alpha", "Beta", "Gamma"] {
        let mut char_data = CharacterData::create_new();
        char_data.sheet.as_mut().unwrap().character.name = name.to_string();
        char_data.save_to_database(&db).expect("save to db");
    }

    let list = db.list_characters().expect("list characters");
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].name, "Alpha");
    assert_eq!(list[1].name, "Beta");
    assert_eq!(list[2].name, "Gamma");
}

// ============================================================================
// Scenario 6: Character data access helpers
// ============================================================================

#[test]
fn test_get_skill_modifier() {
    let mut char_data = CharacterData::create_new();

    if let Some(ref mut sheet) = char_data.sheet {
        // Add a skill with known modifier
        let skill = dndgamerolls::dice3d::types::character::Skill {
            id: "test-stealth".to_string(),
            name: "Stealth".to_string(),
            slug: "stealth".to_string(),
            proficient: true,
            expertise: Some(false),
            modifier: 5,
            proficiency_type: None,
        };
        sheet.skills.insert(skill.id.clone(), skill);
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
fn test_save_to_database_without_sheet_fails() {
    let mut char_data = CharacterData::default();
    let db = CharacterDatabase::open_in_memory().expect("open in-memory db");

    let result = char_data.save_to_database(&db);
    assert!(result.is_err(), "Save without sheet should fail");
    assert!(result.unwrap_err().contains("No character data"));
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
// Scenario 9: Migration to ID-keyed maps
// ============================================================================

#[test]
fn test_migrate_legacy_name_keyed_maps_to_ids() {
    // Legacy data keyed by display names with missing ids/slugs.
    let mut sheet = CharacterSheet {
        skills: HashMap::from([
            (
                "stealth".to_string(),
                Skill {
                    id: String::new(),
                    name: String::new(),
                    slug: String::new(),
                    proficient: true,
                    modifier: 4,
                    expertise: Some(false),
                    proficiency_type: None,
                },
            ),
            (
                "acrobatics".to_string(),
                Skill {
                    id: "fixed-id".to_string(),
                    name: String::new(),
                    slug: String::new(),
                    proficient: false,
                    modifier: 2,
                    expertise: None,
                    proficiency_type: None,
                },
            ),
        ]),
        saving_throws: HashMap::from([(
            "strength".to_string(),
            SavingThrow {
                id: String::new(),
                name: String::new(),
                slug: String::new(),
                proficient: true,
                modifier: 3,
            },
        )]),
        custom_basic_info: HashMap::from([(
            "Nickname".to_string(),
            CustomStringField {
                id: String::new(),
                name: String::new(),
                value: "Rogue".to_string(),
            },
        )]),
        custom_attributes: HashMap::from([(
            "Grit".to_string(),
            CustomIntField {
                id: String::new(),
                name: String::new(),
                value: 7,
            },
        )]),
        custom_combat: HashMap::from([(
            "Parry".to_string(),
            CustomStringField {
                id: String::new(),
                name: String::new(),
                value: "+1 AC".to_string(),
            },
        )]),
        ..Default::default()
    };

    sheet.migrate_to_ids();

    // Skills: re-keyed by id, filled name/slug, values preserved.
    assert_eq!(sheet.skills.len(), 2);
    for skill in sheet.skills.values() {
        assert!(!skill.id.is_empty(), "skill id should be populated");
    }
    let stealth = sheet
        .skills
        .values()
        .find(|s| s.slug == "stealth")
        .expect("stealth skill present");
    assert_eq!(stealth.name, "stealth");
    assert!(stealth.proficient);
    assert_eq!(stealth.modifier, 4);

    let acrobatics = sheet
        .skills
        .get("fixed-id")
        .expect("existing id preserved and used as key");
    assert_eq!(acrobatics.slug, "acrobatics");
    assert_eq!(acrobatics.name, "acrobatics");

    // Saving throws migrated similarly.
    assert_eq!(sheet.saving_throws.len(), 1);
    let (save_id, save) = sheet.saving_throws.iter().next().unwrap();
    assert_eq!(save_id, &save.id);
    assert!(!save.slug.is_empty());
    assert_eq!(save.name, "strength");
    assert!(save.proficient);

    // Custom fields receive ids and keep values/names.
    let basic = sheet.custom_basic_info.values().next().unwrap();
    assert_eq!(basic.name, "Nickname");
    assert_eq!(basic.value, "Rogue");
    assert!(!basic.id.is_empty());

    let custom_attr = sheet.custom_attributes.values().next().unwrap();
    assert_eq!(custom_attr.name, "Grit");
    assert_eq!(custom_attr.value, 7);
    assert!(!custom_attr.id.is_empty());

    let custom_combat = sheet.custom_combat.values().next().unwrap();
    assert_eq!(custom_combat.name, "Parry");
    assert_eq!(custom_combat.value, "+1 AC");
    assert!(!custom_combat.id.is_empty());
}

#[test]
fn test_legacy_missing_ids_become_stable_after_resave() {
    let db = CharacterDatabase::open_in_memory().expect("open in-memory db");

    // Create a legacy-style sheet where map keys are names and entry ids are empty.
    let sheet = CharacterSheet {
        character: dndgamerolls::dice3d::types::CharacterInfo {
            name: "Legacy Hero".to_string(),
            class: "Rogue".to_string(),
            race: "Elf".to_string(),
            level: 3,
            ..Default::default()
        },
        skills: HashMap::from([(
            "stealth".to_string(),
            Skill {
                id: String::new(),
                name: String::new(),
                slug: String::new(),
                proficient: true,
                modifier: 4,
                ..Default::default()
            },
        )]),
        saving_throws: HashMap::from([(
            "dexterity".to_string(),
            SavingThrow {
                id: String::new(),
                name: String::new(),
                slug: String::new(),
                proficient: true,
                modifier: 2,
            },
        )]),
        ..Default::default()
    };

    let id = db.save_character(None, &sheet).expect("save legacy sheet");

    // First load: migration should generate ids.
    let loaded1 = db.load_character(id).expect("load migrated sheet");
    let (skill_id_1, skill_1) = loaded1
        .skills
        .iter()
        .next()
        .expect("skill present");
    assert!(!skill_id_1.is_empty());
    assert_eq!(skill_id_1, &skill_1.id);
    assert_eq!(skill_1.slug, "stealth");
    assert_eq!(skill_1.name, "stealth");

    let (save_id_1, save_1) = loaded1
        .saving_throws
        .iter()
        .next()
        .expect("save present");
    assert!(!save_id_1.is_empty());
    assert_eq!(save_id_1, &save_1.id);
    assert_eq!(save_1.slug, "dexterity");
    assert_eq!(save_1.name, "dexterity");

    // Persist the migrated version (this is what the UI now does on selection).
    db.save_character(Some(id), &loaded1)
        .expect("persist migrated sheet");

    // Second load: ids should be stable (no regeneration).
    let loaded2 = db.load_character(id).expect("reload persisted sheet");
    assert!(loaded2.skills.contains_key(skill_id_1));
    assert!(loaded2.saving_throws.contains_key(save_id_1));
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
fn test_save_to_database_clears_modified_flag() {
    let db = CharacterDatabase::open_in_memory().expect("open in-memory db");
    let mut char_data = CharacterData::create_new();
    assert!(char_data.is_modified, "Should start as modified");

    if let Some(ref mut sheet) = char_data.sheet {
        sheet.character.name = "Save Test".to_string();
    }
    char_data.save_to_database(&db).unwrap();

    assert!(
        !char_data.is_modified,
        "Modified flag should be cleared after save"
    );
}
