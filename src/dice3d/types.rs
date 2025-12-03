use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Component)]
pub struct Die {
    pub die_type: DiceType,
    pub face_normals: Vec<(Vec3, u32)>,
}

#[derive(Component)]
pub struct DiceBox;

#[derive(Component)]
pub struct ResultsText;

#[derive(Component)]
pub struct MainCamera;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
}

impl DiceType {
    pub fn max_value(&self) -> u32 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DiceType::D4 => "D4",
            DiceType::D6 => "D6",
            DiceType::D8 => "D8",
            DiceType::D10 => "D10",
            DiceType::D12 => "D12",
            DiceType::D20 => "D20",
        }
    }

    pub fn color(&self) -> Color {
        // Translucent crystal-like colors
        match self {
            DiceType::D4 => Color::srgba(0.3, 0.4, 0.9, 0.85),   // Blue crystal
            DiceType::D6 => Color::srgba(0.1, 0.1, 0.1, 0.9),    // Black/smoke crystal
            DiceType::D8 => Color::srgba(0.6, 0.2, 0.8, 0.85),   // Purple crystal
            DiceType::D10 => Color::srgba(0.95, 0.95, 0.95, 0.85), // White/clear crystal
            DiceType::D12 => Color::srgba(0.95, 0.5, 0.1, 0.85), // Orange crystal
            DiceType::D20 => Color::srgba(0.95, 0.85, 0.2, 0.85), // Yellow crystal
        }
    }

    pub fn from_str(s: &str) -> Option<DiceType> {
        match s.to_lowercase().as_str() {
            "d4" => Some(DiceType::D4),
            "d6" => Some(DiceType::D6),
            "d8" => Some(DiceType::D8),
            "d10" => Some(DiceType::D10),
            "d12" => Some(DiceType::D12),
            "d20" => Some(DiceType::D20),
            _ => None,
        }
    }
}

#[derive(Resource, Default)]
pub struct DiceResults {
    pub results: Vec<(DiceType, u32)>,
}

#[derive(Resource, Default)]
pub struct RollState {
    pub rolling: bool,
    pub settle_timer: f32,
}

/// Configuration for which dice to spawn
#[derive(Resource, Clone)]
pub struct DiceConfig {
    pub dice_to_roll: Vec<DiceType>,
    pub modifier: i32,
    pub modifier_name: String,
}

impl Default for DiceConfig {
    fn default() -> Self {
        Self {
            dice_to_roll: vec![DiceType::D20],
            modifier: 0,
            modifier_name: String::new(),
        }
    }
}

/// Component for the command input text display
#[derive(Component)]
pub struct CommandInputText;

/// Resource for storing the current command input
#[derive(Resource, Default)]
pub struct CommandInput {
    pub text: String,
    pub active: bool,
}

/// Component for command history list display
#[derive(Component)]
pub struct CommandHistoryList;

/// Component for individual command history items
#[derive(Component)]
pub struct CommandHistoryItem {
    pub index: usize,
}

/// Resource for storing command history
#[derive(Resource, Default)]
pub struct CommandHistory {
    pub commands: Vec<String>,
    pub selected_index: Option<usize>,
}

impl CommandHistory {
    pub fn add_command(&mut self, cmd: String) {
        // Only add if not already in the list
        if !cmd.trim().is_empty() && !self.commands.contains(&cmd) {
            self.commands.push(cmd);
        }
    }
}

// Character data structures for JSON loading
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

#[derive(Debug, Deserialize, Clone)]
pub struct Attributes {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AttributeModifiers {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Combat {
    #[serde(rename = "armorClass")]
    pub armor_class: i32,
    pub initiative: i32,
    pub speed: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SavingThrow {
    pub proficient: bool,
    pub modifier: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Skill {
    pub proficient: bool,
    pub modifier: i32,
    pub expertise: Option<bool>,
}

#[derive(Resource)]
pub struct CharacterData {
    pub sheet: Option<CharacterSheet>,
}

impl Default for CharacterData {
    fn default() -> Self {
        Self { sheet: None }
    }
}

impl CharacterData {
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

    pub fn get_skill_modifier(&self, skill: &str) -> Option<i32> {
        self.sheet.as_ref().and_then(|s| {
            s.skills.get(skill).map(|sk| sk.modifier)
        })
    }

    pub fn get_ability_modifier(&self, ability: &str) -> Option<i32> {
        self.sheet.as_ref().map(|s| {
            match ability.to_lowercase().as_str() {
                "str" | "strength" => s.modifiers.strength,
                "dex" | "dexterity" => s.modifiers.dexterity,
                "con" | "constitution" => s.modifiers.constitution,
                "int" | "intelligence" => s.modifiers.intelligence,
                "wis" | "wisdom" => s.modifiers.wisdom,
                "cha" | "charisma" => s.modifiers.charisma,
                _ => 0,
            }
        })
    }

    pub fn get_saving_throw_modifier(&self, ability: &str) -> Option<i32> {
        self.sheet.as_ref().and_then(|s| {
            s.saving_throws.get(&ability.to_lowercase()).map(|st| st.modifier)
        })
    }
}

