//! Character data types for D&D character sheets
//!
//! This module contains all types related to loading, saving, and accessing
//! character data persisted via the app's database layer.

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    #[serde(default, deserialize_with = "deserialize_saving_throws")]
    pub saving_throws: HashMap<String, SavingThrow>,
    #[serde(default, deserialize_with = "deserialize_skills")]
    pub skills: HashMap<String, Skill>,
    #[serde(default)]
    pub equipment: Option<Equipment>,
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub spells: Option<SpellCasting>,
    /// Custom fields for Basic Info group
    #[serde(
        rename = "customBasicInfo",
        default,
        deserialize_with = "deserialize_custom_basic_info"
    )]
    pub custom_basic_info: HashMap<String, CustomStringField>,
    /// Custom attributes beyond the standard 6
    #[serde(
        rename = "customAttributes",
        default,
        deserialize_with = "deserialize_custom_attributes"
    )]
    pub custom_attributes: HashMap<String, CustomIntField>,
    /// Custom combat stats (string values)
    #[serde(
        rename = "customCombat",
        default,
        deserialize_with = "deserialize_custom_combat"
    )]
    pub custom_combat: HashMap<String, CustomStringField>,
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
    #[serde(default = "new_id")]
    pub id: String,
    /// Display name (editable)
    #[serde(default)]
    pub name: String,
    /// Canonical slug used for lookups (lowercase, no spaces)
    #[serde(default)]
    pub slug: String,
    pub proficient: bool,
    pub modifier: i32,
}

/// Skill data
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Skill {
    #[serde(default = "new_id")]
    pub id: String,
    /// Display name (editable)
    #[serde(default)]
    pub name: String,
    /// Canonical slug used for lookups (lowercase, no spaces)
    #[serde(default)]
    pub slug: String,
    pub proficient: bool,
    pub modifier: i32,
    #[serde(default)]
    pub expertise: Option<bool>,
    #[serde(rename = "proficiencyType", default)]
    pub proficiency_type: Option<String>,
}

/// Custom string field with stable id
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CustomStringField {
    #[serde(default = "new_id")]
    pub id: String,
    /// Display name
    #[serde(default)]
    pub name: String,
    /// Stored value
    #[serde(default)]
    pub value: String,
}

/// Custom integer field with stable id
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CustomIntField {
    #[serde(default = "new_id")]
    pub id: String,
    /// Display name
    #[serde(default)]
    pub name: String,
    /// Stored value
    pub value: i32,
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
        let mut rng = rand::rng();

        // Roll d20 for each core attribute
        let strength = rng.random_range(1..=20);
        let dexterity = rng.random_range(1..=20);
        let constitution = rng.random_range(1..=20);
        let intelligence = rng.random_range(1..=20);
        let wisdom = rng.random_range(1..=20);
        let charisma = rng.random_range(1..=20);

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
            let id = new_id();
            saves.insert(
                id.clone(),
                SavingThrow {
                    id,
                    name: ability.to_string(),
                    slug: ability.to_string(),
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
            let id = new_id();
            skills.insert(
                id.clone(),
                Skill {
                    id,
                    name: name.to_string(),
                    slug: make_slug(name),
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
            .and_then(|s| resolve_skill(&s.skills, skill))
            .map(|sk| sk.modifier)
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
                    // Custom attributes: search by slug/name
                    s.custom_attributes
                        .values()
                        .find(|entry| make_slug(&entry.name) == key || entry.name.to_lowercase() == key)
                        .map(|entry| Attributes::calculate_modifier(entry.value))
                        .unwrap_or(0)
                }
            }
        })
    }

    /// Get the modifier for a saving throw by ability name
    pub fn get_saving_throw_modifier(&self, ability: &str) -> Option<i32> {
        self.sheet
            .as_ref()
            .and_then(|s| resolve_saving_throw(&s.saving_throws, ability))
            .map(|st| st.modifier)
    }
}

// ============================================================================
// ID / Slug helpers
// ============================================================================

fn new_id() -> String {
    Uuid::now_v7().to_string()
}

fn make_slug(name: &str) -> String {
    name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "")
}

fn resolve_skill<'a>(
    skills: &'a HashMap<String, Skill>,
    query: &str,
) -> Option<&'a Skill> {
    let q = make_slug(query);
    // direct id match
    if let Some(sk) = skills.get(query) {
        return Some(sk);
    }
    // Prefer the best-matching slug/name entry when duplicates exist.
    skills
        .values()
        .filter(|s| s.slug == q || make_slug(&s.name) == q)
        .max_by_key(|s| {
            let proficiency_weight = if s.expertise.unwrap_or(false) {
                2
            } else if s.proficient {
                1
            } else {
                0
            };
            (proficiency_weight, s.modifier)
        })
}

fn resolve_saving_throw<'a>(
    saves: &'a HashMap<String, SavingThrow>,
    query: &str,
) -> Option<&'a SavingThrow> {
    let q = make_slug(query);
    if let Some(sv) = saves.get(query) {
        return Some(sv);
    }
    saves.values().find(|s| s.slug == q || make_slug(&s.name) == q)
}

// ============================================================================
// Migration helpers
// ============================================================================

fn migrate_skill_entry(key: String, mut skill: Skill) -> (String, Skill) {
    if skill.id.is_empty() {
        skill.id = new_id();
    }
    if skill.slug.is_empty() {
        // Prefer existing slug from key to preserve canonical names.
        skill.slug = make_slug(&key);
    }
    if skill.name.is_empty() {
        skill.name = key.clone();
    }
    let id = skill.id.clone();
    (id, skill)
}

fn migrate_save_entry(key: String, mut save: SavingThrow) -> (String, SavingThrow) {
    if save.id.is_empty() {
        save.id = new_id();
    }
    if save.slug.is_empty() {
        save.slug = make_slug(&key);
    }
    if save.name.is_empty() {
        save.name = key.clone();
    }
    let id = save.id.clone();
    (id, save)
}

fn migrate_custom_string_entry(key: String, value: String) -> (String, CustomStringField) {
    let id = new_id();
    (
        id.clone(),
        CustomStringField {
            id,
            name: key,
            value,
        },
    )
}

fn migrate_custom_int_entry(key: String, value: i32) -> (String, CustomIntField) {
    let id = new_id();
    (
        id.clone(),
        CustomIntField {
            id,
            name: key,
            value,
        },
    )
}

impl CharacterSheet {
    /// Migrate legacy name-keyed data to ID-keyed maps and ensure IDs/names/slugs are populated.
    pub fn migrate_to_ids(&mut self) {
        // Skills
        let mut migrated_skills = HashMap::new();
        for (key, skill) in std::mem::take(&mut self.skills) {
            let (id, sk) = migrate_skill_entry(key, skill);
            migrated_skills.insert(id, sk);
        }
        self.skills = migrated_skills;

        // Saving throws
        let mut migrated_saves = HashMap::new();
        for (key, save) in std::mem::take(&mut self.saving_throws) {
            let (id, sv) = migrate_save_entry(key, save);
            migrated_saves.insert(id, sv);
        }
        self.saving_throws = migrated_saves;

        // Custom basic info (string)
        if !self.custom_basic_info.values().all(|v| !v.id.is_empty()) {
            let mut migrated = HashMap::new();
            for (key, entry) in std::mem::take(&mut self.custom_basic_info) {
                let (id, val) = if entry.id.is_empty() && entry.name.is_empty() {
                    migrate_custom_string_entry(key, entry.value)
                } else {
                    let mut v = entry;
                    if v.id.is_empty() {
                        v.id = new_id();
                    }
                    if v.name.is_empty() {
                        v.name = key;
                    }
                    (v.id.clone(), v)
                };
                migrated.insert(id, val);
            }
            self.custom_basic_info = migrated;
        }

        // Custom attributes (int)
        if !self.custom_attributes.values().all(|v| !v.id.is_empty()) {
            let mut migrated = HashMap::new();
            for (key, entry) in std::mem::take(&mut self.custom_attributes) {
                let (id, val) = if entry.id.is_empty() && entry.name.is_empty() {
                    migrate_custom_int_entry(key, entry.value)
                } else {
                    let mut v = entry;
                    if v.id.is_empty() {
                        v.id = new_id();
                    }
                    if v.name.is_empty() {
                        v.name = key;
                    }
                    (v.id.clone(), v)
                };
                migrated.insert(id, val);
            }
            self.custom_attributes = migrated;
        }

        // Custom combat (string)
        if !self.custom_combat.values().all(|v| !v.id.is_empty()) {
            let mut migrated = HashMap::new();
            for (key, entry) in std::mem::take(&mut self.custom_combat) {
                let (id, val) = if entry.id.is_empty() && entry.name.is_empty() {
                    migrate_custom_string_entry(key, entry.value)
                } else {
                    let mut v = entry;
                    if v.id.is_empty() {
                        v.id = new_id();
                    }
                    if v.name.is_empty() {
                        v.name = key;
                    }
                    (v.id.clone(), v)
                };
                migrated.insert(id, val);
            }
            self.custom_combat = migrated;
        }
    }
}

// ============================================================================
// Legacy-friendly deserializers
// ============================================================================

fn deserialize_skills<'de, D>(deserializer: D) -> Result<HashMap<String, Skill>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: HashMap<String, Skill> = HashMap::deserialize(deserializer)?;
    Ok(raw)
}

fn deserialize_saving_throws<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, SavingThrow>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: HashMap<String, SavingThrow> = HashMap::deserialize(deserializer)?;
    Ok(raw)
}

fn deserialize_custom_basic_info<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, CustomStringField>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Entry {
        Legacy(HashMap<String, String>),
        New(HashMap<String, CustomStringField>),
    }

    match Entry::deserialize(deserializer)? {
        Entry::Legacy(map) => Ok(map
            .into_iter()
            .map(|(k, v)| migrate_custom_string_entry(k, v))
            .collect()),
        Entry::New(map) => Ok(map),
    }
}

fn deserialize_custom_attributes<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, CustomIntField>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Entry {
        Legacy(HashMap<String, i32>),
        New(HashMap<String, CustomIntField>),
    }

    match Entry::deserialize(deserializer)? {
        Entry::Legacy(map) => Ok(map
            .into_iter()
            .map(|(k, v)| migrate_custom_int_entry(k, v))
            .collect()),
        Entry::New(map) => Ok(map),
    }
}

fn deserialize_custom_combat<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, CustomStringField>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Entry {
        Legacy(HashMap<String, String>),
        New(HashMap<String, CustomStringField>),
    }

    match Entry::deserialize(deserializer)? {
        Entry::Legacy(map) => Ok(map
            .into_iter()
            .map(|(k, v)| migrate_custom_string_entry(k, v))
            .collect()),
        Entry::New(map) => Ok(map),
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
