//! Character data types for D&D character sheets
//!
//! This module contains all types related to loading and accessing
//! character data from JSON files.

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Complete character sheet data
#[derive(Debug, Deserialize, Clone)]
pub struct CharacterSheet {
    pub character: CharacterInfo,
    pub attributes: Attributes,
    pub modifiers: AttributeModifiers,
    pub combat: Combat,
    #[serde(rename = "proficiencyBonus")]
    pub proficiency_bonus: i32,
    #[serde(rename = "savingThrows")]
    pub saving_throws: HashMap<String, SavingThrow>,
    pub skills: HashMap<String, Skill>,
}

/// Basic character information
#[derive(Debug, Deserialize, Clone)]
pub struct CharacterInfo {
    pub name: String,
    #[serde(rename = "alterEgo")]
    pub alter_ego: Option<String>,
    pub class: String,
    pub subclass: Option<String>,
    pub race: String,
    pub level: i32,
}

/// Character ability scores
#[derive(Debug, Deserialize, Clone)]
pub struct Attributes {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// Ability score modifiers
#[derive(Debug, Deserialize, Clone)]
pub struct AttributeModifiers {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// Combat statistics
#[derive(Debug, Deserialize, Clone)]
pub struct Combat {
    #[serde(rename = "armorClass")]
    pub armor_class: i32,
    pub initiative: i32,
    pub speed: i32,
}

/// Saving throw data
#[derive(Debug, Deserialize, Clone)]
pub struct SavingThrow {
    pub proficient: bool,
    pub modifier: i32,
}

/// Skill data
#[derive(Debug, Deserialize, Clone)]
pub struct Skill {
    pub proficient: bool,
    pub modifier: i32,
    pub expertise: Option<bool>,
}

/// Resource containing the loaded character data
#[derive(Resource, Default)]
pub struct CharacterData {
    pub sheet: Option<CharacterSheet>,
}

impl CharacterData {
    /// Load character data from a JSON file
    pub fn load_from_file(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(sheet) => {
                    println!("Loaded character sheet from {}", path);
                    Self { sheet: Some(sheet) }
                }
                Err(e) => {
                    eprintln!("Failed to parse character sheet: {}", e);
                    Self { sheet: None }
                }
            },
            Err(e) => {
                eprintln!("Failed to read character file: {}", e);
                Self { sheet: None }
            }
        }
    }

    /// Get the modifier for a skill by name
    pub fn get_skill_modifier(&self, skill: &str) -> Option<i32> {
        self.sheet
            .as_ref()
            .and_then(|s| s.skills.get(skill).map(|sk| sk.modifier))
    }

    /// Get the modifier for an ability by name
    pub fn get_ability_modifier(&self, ability: &str) -> Option<i32> {
        self.sheet
            .as_ref()
            .map(|s| match ability.to_lowercase().as_str() {
                "str" | "strength" => s.modifiers.strength,
                "dex" | "dexterity" => s.modifiers.dexterity,
                "con" | "constitution" => s.modifiers.constitution,
                "int" | "intelligence" => s.modifiers.intelligence,
                "wis" | "wisdom" => s.modifiers.wisdom,
                "cha" | "charisma" => s.modifiers.charisma,
                _ => 0,
            })
    }

    /// Get the modifier for a saving throw by ability name
    pub fn get_saving_throw_modifier(&self, ability: &str) -> Option<i32> {
        self.sheet.as_ref().and_then(|s| {
            s.saving_throws
                .get(&ability.to_lowercase())
                .map(|st| st.modifier)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_data_default() {
        let data = CharacterData::default();
        assert!(data.sheet.is_none());
        assert!(data.get_skill_modifier("stealth").is_none());
        assert!(data.get_ability_modifier("dex").is_none());
        assert!(data.get_saving_throw_modifier("dex").is_none());
    }
}
