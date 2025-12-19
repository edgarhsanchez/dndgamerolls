//! Character data types for D&D character sheets
//!
//! This module contains all types related to loading, saving, and accessing
//! character data persisted via the app's database layer.

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Character Schema Types - Full D&D 5e Character Sheet
// ============================================================================

/// Complete character sheet data
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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
    #[serde(default)]
    pub equipment: Option<Equipment>,
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub spells: Option<SpellCasting>,
    /// Custom fields for Basic Info group (name -> value)
    #[serde(rename = "customBasicInfo", default)]
    pub custom_basic_info: HashMap<String, String>,
    /// Custom attributes beyond the standard 6 (name -> score)
    #[serde(rename = "customAttributes", default)]
    pub custom_attributes: HashMap<String, i32>,
    /// Custom combat stats (name -> value as string)
    #[serde(rename = "customCombat", default)]
    pub custom_combat: HashMap<String, String>,
}

/// Basic character information
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CharacterInfo {
    pub name: String,
    #[serde(rename = "alterEgo", default)]
    pub alter_ego: Option<String>,
    #[serde(rename = "familyName", default)]
    pub family_name: Option<String>,
    #[serde(rename = "shopName", default)]
    pub shop_name: Option<String>,
    pub class: String,
    #[serde(default)]
    pub subclass: Option<String>,
    pub race: String,
    pub level: i32,
    #[serde(default)]
    pub experience: i32,
    #[serde(default)]
    pub alignment: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub languages: Vec<String>,
}

/// Character ability scores
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Attributes {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl Attributes {
    /// Calculate modifier from ability score (standard D&D formula)
    pub fn calculate_modifier(score: i32) -> i32 {
        (score - 10) / 2
    }

    /// Get all attributes as a vec of (name, score) tuples
    pub fn as_vec(&self) -> Vec<(&'static str, i32)> {
        vec![
            ("Strength", self.strength),
            ("Dexterity", self.dexterity),
            ("Constitution", self.constitution),
            ("Intelligence", self.intelligence),
            ("Wisdom", self.wisdom),
            ("Charisma", self.charisma),
        ]
    }
}

/// Ability score modifiers
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AttributeModifiers {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// Combat statistics
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Combat {
    #[serde(rename = "armorClass")]
    pub armor_class: i32,
    pub initiative: i32,
    #[serde(default)]
    pub speed: i32,
    #[serde(rename = "hitPoints", default)]
    pub hit_points: Option<HitPoints>,
    #[serde(rename = "hitDice", default)]
    pub hit_dice: Option<HitDice>,
    #[serde(rename = "deathSaves", default)]
    pub death_saves: Option<DeathSaves>,
}

/// Hit points tracking
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HitPoints {
    pub current: i32,
    pub maximum: i32,
    #[serde(default)]
    pub temporary: i32,
}

/// Hit dice tracking
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HitDice {
    pub total: String,
    pub current: i32,
}

/// Death saves tracking
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DeathSaves {
    pub successes: i32,
    pub failures: i32,
}

/// Saving throw data
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SavingThrow {
    pub proficient: bool,
    pub modifier: i32,
}

/// Skill data
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Skill {
    pub proficient: bool,
    pub modifier: i32,
    #[serde(default)]
    pub expertise: Option<bool>,
    #[serde(rename = "proficiencyType", default)]
    pub proficiency_type: Option<String>,
}

/// Equipment and inventory
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Equipment {
    #[serde(default)]
    pub weapons: Vec<Weapon>,
    #[serde(default)]
    pub armor: Option<Armor>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub currency: Currency,
}

/// Weapon data
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Weapon {
    pub name: String,
    #[serde(rename = "attackBonus")]
    pub attack_bonus: i32,
    pub damage: String,
    #[serde(rename = "damageType")]
    pub damage_type: String,
    #[serde(default)]
    pub properties: Vec<String>,
}

/// Armor data
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Armor {
    pub name: String,
    #[serde(rename = "armorClass")]
    pub armor_class: i32,
    #[serde(rename = "armorClassWithDex", default)]
    pub armor_class_with_dex: Option<i32>,
    #[serde(rename = "type", default)]
    pub armor_type: Option<String>,
}

/// Currency tracking
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Currency {
    #[serde(default)]
    pub copper: i32,
    #[serde(default)]
    pub silver: i32,
    #[serde(default)]
    pub electrum: i32,
    #[serde(default)]
    pub gold: i32,
    #[serde(default)]
    pub platinum: i32,
}

/// Character feature or trait
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Feature {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub damage: Option<String>,
}

/// Spellcasting information
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SpellCasting {
    #[serde(rename = "spellcastingAbility", default)]
    pub spellcasting_ability: Option<String>,
    #[serde(rename = "spellSaveDC", default)]
    pub spell_save_dc: Option<i32>,
    #[serde(rename = "spellAttackBonus", default)]
    pub spell_attack_bonus: Option<i32>,
    #[serde(rename = "spellSlots", default)]
    pub spell_slots: HashMap<String, i32>,
    #[serde(rename = "knownSpells", default)]
    pub known_spells: Vec<String>,
}

// ============================================================================
// Character Management
// ============================================================================

/// Character list entry for UI display (from database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterListEntry {
    /// Database ID (stable, never changes)
    pub id: i64,
    /// Character name (can be changed)
    pub name: String,
    /// Class for display
    pub class: String,
    /// Level for display  
    pub level: i32,
}

/// Resource for managing available characters
#[derive(Resource, Default)]
pub struct CharacterManager {
    /// Characters loaded from database
    pub characters: Vec<CharacterListEntry>,
    /// Currently selected character ID
    pub current_character_id: Option<i64>,
    /// Version counter that increments when the list needs to be refreshed
    pub list_version: u32,
}

impl CharacterManager {
    /// Refresh the character list from the database
    pub fn refresh_from_database(&mut self, db: &super::database::CharacterDatabase) {
        match db.list_characters() {
            Ok(entries) => {
                self.characters = entries
                    .into_iter()
                    .map(|e| CharacterListEntry {
                        id: e.id,
                        name: e.name,
                        class: e.class,
                        level: e.level,
                    })
                    .collect();
            }
            Err(e) => {
                eprintln!("Failed to refresh character list: {}", e);
            }
        }
    }
}

// ============================================================================
// Character Data Resource (existing, enhanced)
// ============================================================================

/// Resource containing the loaded character data
#[derive(Resource, Default)]
pub struct CharacterData {
    pub sheet: Option<CharacterSheet>,
    /// Database ID for this character (None if not yet saved to database)
    pub character_id: Option<i64>,
    pub is_modified: bool,
    /// Flag to trigger UI refresh - set when data changes, cleared after refresh
    pub needs_refresh: bool,
}

impl CharacterData {
    /// Save character to database
    /// Creates a new entry if character_id is None, updates existing if set
    pub fn save_to_database(
        &mut self,
        db: &super::database::CharacterDatabase,
    ) -> Result<(), String> {
        let sheet = self
            .sheet
            .as_ref()
            .ok_or_else(|| "No character data to save".to_string())?;

        let id = db.save_character(self.character_id, sheet)?;
        self.character_id = Some(id);
        self.is_modified = false;

        Ok(())
    }

    /// Load character from database by ID
    pub fn load_from_database(
        db: &super::database::CharacterDatabase,
        id: i64,
    ) -> Result<Self, String> {
        let sheet = db.load_character(id)?;

        Ok(Self {
            sheet: Some(sheet),
            character_id: Some(id),
            is_modified: false,
            needs_refresh: true,
        })
    }

    /// Create a new default character with randomly rolled stats using d20s
    pub fn create_new() -> Self {
        let mut rng = rand::thread_rng();

        // Roll d20 for each core attribute
        let strength = rng.gen_range(1..=20);
        let dexterity = rng.gen_range(1..=20);
        let constitution = rng.gen_range(1..=20);
        let intelligence = rng.gen_range(1..=20);
        let wisdom = rng.gen_range(1..=20);
        let charisma = rng.gen_range(1..=20);

        // Calculate modifiers from attributes (D&D formula: (score - 10) / 2)
        let str_mod = Attributes::calculate_modifier(strength);
        let dex_mod = Attributes::calculate_modifier(dexterity);
        let con_mod = Attributes::calculate_modifier(constitution);
        let int_mod = Attributes::calculate_modifier(intelligence);
        let wis_mod = Attributes::calculate_modifier(wisdom);
        let cha_mod = Attributes::calculate_modifier(charisma);

        // Calculate derived stats
        let armor_class = 10 + dex_mod; // Base AC + Dex modifier
        let initiative = dex_mod; // Initiative is Dex modifier
        let base_hp = 10; // Fighter's d10 hit die, max at level 1
        let max_hp = (base_hp + con_mod).max(1); // HP can't go below 1

        let sheet = CharacterSheet {
            character: CharacterInfo {
                name: "New Character".to_string(),
                class: "Fighter".to_string(),
                race: "Human".to_string(),
                level: 1,
                ..Default::default()
            },
            attributes: Attributes {
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
            },
            modifiers: AttributeModifiers {
                strength: str_mod,
                dexterity: dex_mod,
                constitution: con_mod,
                intelligence: int_mod,
                wisdom: wis_mod,
                charisma: cha_mod,
            },
            combat: Combat {
                armor_class,
                initiative,
                speed: 30,
                hit_points: Some(HitPoints {
                    current: max_hp,
                    maximum: max_hp,
                    temporary: 0,
                }),
                ..Default::default()
            },
            proficiency_bonus: 2,
            saving_throws: Self::create_saving_throws(
                str_mod, dex_mod, con_mod, int_mod, wis_mod, cha_mod,
            ),
            skills: Self::default_skills(),
            ..Default::default()
        };

        Self {
            sheet: Some(sheet),
            character_id: None, // New character, not yet in database
            is_modified: true,
            needs_refresh: true,
        }
    }

    /// Create saving throws with modifiers based on attribute scores
    fn create_saving_throws(
        str_mod: i32,
        dex_mod: i32,
        con_mod: i32,
        int_mod: i32,
        wis_mod: i32,
        cha_mod: i32,
    ) -> HashMap<String, SavingThrow> {
        let mut saves = HashMap::new();
        let mods = [
            ("strength", str_mod),
            ("dexterity", dex_mod),
            ("constitution", con_mod),
            ("intelligence", int_mod),
            ("wisdom", wis_mod),
            ("charisma", cha_mod),
        ];
        for (ability, modifier) in mods {
            saves.insert(
                ability.to_string(),
                SavingThrow {
                    proficient: false,
                    modifier,
                },
            );
        }
        saves
    }

    fn default_skills() -> HashMap<String, Skill> {
        let mut skills = HashMap::new();
        let skill_names = [
            "acrobatics",
            "animalHandling",
            "arcana",
            "athletics",
            "deception",
            "history",
            "insight",
            "intimidation",
            "investigation",
            "medicine",
            "nature",
            "perception",
            "performance",
            "persuasion",
            "religion",
            "sleightOfHand",
            "stealth",
            "survival",
        ];
        for name in skill_names {
            skills.insert(
                name.to_string(),
                Skill {
                    proficient: false,
                    modifier: 0,
                    ..Default::default()
                },
            );
        }
        skills
    }

    /// Get the modifier for a skill by name
    pub fn get_skill_modifier(&self, skill: &str) -> Option<i32> {
        self.sheet
            .as_ref()
            .and_then(|s| s.skills.get(skill).map(|sk| sk.modifier))
    }

    /// Get the modifier for an ability by name
    pub fn get_ability_modifier(&self, ability: &str) -> Option<i32> {
        self.sheet.as_ref().map(|s| {
            let key = ability.to_lowercase();
            match key.as_str() {
                "str" | "strength" => s.modifiers.strength,
                "dex" | "dexterity" => s.modifiers.dexterity,
                "con" | "constitution" => s.modifiers.constitution,
                "int" | "intelligence" => s.modifiers.intelligence,
                "wis" | "wisdom" => s.modifiers.wisdom,
                "cha" | "charisma" => s.modifiers.charisma,
                _ => {
                    // Custom attributes store scores; derive modifier with the standard formula.
                    if let Some(score) = s.custom_attributes.get(ability) {
                        return Attributes::calculate_modifier(*score);
                    }
                    if let Some((_, score)) = s
                        .custom_attributes
                        .iter()
                        .find(|(name, _)| name.to_lowercase() == key)
                    {
                        return Attributes::calculate_modifier(*score);
                    }

                    0
                }
            }
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

    #[test]
    fn test_calculate_modifier() {
        assert_eq!(Attributes::calculate_modifier(10), 0);
        assert_eq!(Attributes::calculate_modifier(20), 5);
        assert_eq!(Attributes::calculate_modifier(8), -1);
        assert_eq!(Attributes::calculate_modifier(15), 2);
    }
}
