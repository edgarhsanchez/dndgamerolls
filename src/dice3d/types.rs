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
            DiceType::D4 => Color::srgba(0.3, 0.4, 0.9, 0.85), // Blue crystal
            DiceType::D6 => Color::srgba(0.1, 0.1, 0.1, 0.9),  // Black/smoke crystal
            DiceType::D8 => Color::srgba(0.6, 0.2, 0.8, 0.85), // Purple crystal
            DiceType::D10 => Color::srgba(0.95, 0.95, 0.95, 0.85), // White/clear crystal
            DiceType::D12 => Color::srgba(0.95, 0.5, 0.1, 0.85), // Orange crystal
            DiceType::D20 => Color::srgba(0.95, 0.85, 0.2, 0.85), // Yellow crystal
        }
    }

    pub fn parse(s: &str) -> Option<DiceType> {
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

/// Resource for camera zoom level
#[derive(Resource)]
pub struct ZoomState {
    pub level: f32, // 0.0 = closest, 1.0 = farthest
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            level: 0.3,        // Start closer (30% of range)
            min_distance: 4.0, // Can zoom in much closer
            max_distance: 25.0,
        }
    }
}

impl ZoomState {
    pub fn get_distance(&self) -> f32 {
        self.min_distance + self.level * (self.max_distance - self.min_distance)
    }
}

/// Component for the zoom slider container
#[derive(Component)]
pub struct ZoomSliderContainer;

/// Component for the zoom slider handle
#[derive(Component)]
pub struct ZoomSliderHandle;

/// Component for the zoom slider track
#[derive(Component)]
pub struct ZoomSliderTrack;

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

#[derive(Resource, Default)]
pub struct CharacterData {
    pub sheet: Option<CharacterSheet>,
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
        self.sheet
            .as_ref()
            .and_then(|s| s.skills.get(skill).map(|sk| sk.modifier))
    }

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
    fn test_dice_type_max_value() {
        assert_eq!(DiceType::D4.max_value(), 4);
        assert_eq!(DiceType::D6.max_value(), 6);
        assert_eq!(DiceType::D8.max_value(), 8);
        assert_eq!(DiceType::D10.max_value(), 10);
        assert_eq!(DiceType::D12.max_value(), 12);
        assert_eq!(DiceType::D20.max_value(), 20);
    }

    #[test]
    fn test_dice_type_name() {
        assert_eq!(DiceType::D4.name(), "D4");
        assert_eq!(DiceType::D6.name(), "D6");
        assert_eq!(DiceType::D20.name(), "D20");
    }

    #[test]
    fn test_dice_type_parse() {
        assert_eq!(DiceType::parse("d4"), Some(DiceType::D4));
        assert_eq!(DiceType::parse("D4"), Some(DiceType::D4));
        assert_eq!(DiceType::parse("d20"), Some(DiceType::D20));
        assert_eq!(DiceType::parse("D20"), Some(DiceType::D20));
        assert_eq!(DiceType::parse("invalid"), None);
        assert_eq!(DiceType::parse("d100"), None);
    }

    #[test]
    fn test_dice_config_default() {
        let config = DiceConfig::default();
        assert_eq!(config.dice_to_roll, vec![DiceType::D20]);
        assert_eq!(config.modifier, 0);
        assert!(config.modifier_name.is_empty());
    }

    #[test]
    fn test_command_history_add() {
        let mut history = CommandHistory::default();
        assert!(history.commands.is_empty());

        history.add_command("--dice 2d6".to_string());
        assert_eq!(history.commands.len(), 1);
        assert_eq!(history.commands[0], "--dice 2d6");

        // Adding same command should not duplicate
        history.add_command("--dice 2d6".to_string());
        assert_eq!(history.commands.len(), 1);

        // Adding different command should add
        history.add_command("--dice 1d20".to_string());
        assert_eq!(history.commands.len(), 2);

        // Empty command should not be added
        history.add_command("".to_string());
        history.add_command("   ".to_string());
        assert_eq!(history.commands.len(), 2);
    }

    #[test]
    fn test_command_input_default() {
        let input = CommandInput::default();
        assert!(input.text.is_empty());
        assert!(!input.active);
    }

    #[test]
    fn test_dice_results_default() {
        let results = DiceResults::default();
        assert!(results.results.is_empty());
    }

    #[test]
    fn test_roll_state_default() {
        let state = RollState::default();
        assert!(!state.rolling);
        assert_eq!(state.settle_timer, 0.0);
    }

    #[test]
    fn test_character_data_default() {
        let data = CharacterData::default();
        assert!(data.sheet.is_none());
        assert!(data.get_skill_modifier("stealth").is_none());
        assert!(data.get_ability_modifier("dex").is_none());
        assert!(data.get_saving_throw_modifier("dex").is_none());
    }
}
