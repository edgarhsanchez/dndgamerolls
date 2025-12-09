//! UI-related types and components
//!
//! This module contains all UI components: text displays, command input,
//! command history, zoom slider, tab navigation, and character screen.

use bevy::prelude::*;

// ============================================================================
// Tab Navigation
// ============================================================================

/// Current active tab/view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, States, Hash)]
pub enum AppTab {
    #[default]
    DiceRoller,
    CharacterSheet,
    DndInfo,
    Contributors,
}

/// Resource for UI state
#[derive(Resource, Default)]
pub struct UiState {
    pub active_tab: AppTab,
    pub character_list_open: bool,
    pub selected_character_index: Option<usize>,
    pub editing_field: Option<EditingField>,
    pub show_save_confirmation: bool,
}

/// Which field is currently being edited
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditingField {
    CharacterName,
    CharacterClass,
    CharacterRace,
    CharacterLevel,
    AttributeStrength,
    AttributeDexterity,
    AttributeConstitution,
    AttributeIntelligence,
    AttributeWisdom,
    AttributeCharisma,
    ArmorClass,
    Initiative,
    Speed,
    HitPointsCurrent,
    HitPointsMaximum,
    ProficiencyBonus,
    // Skills and saves are handled by name
    Skill(String),
    SavingThrow(String),
    // Label editing (renaming skills/saves)
    SkillLabel(String),       // Editing the name of a skill
    SavingThrowLabel(String), // Editing the name of a saving throw
    // Custom fields (name is the key in the HashMap)
    CustomBasicInfo(String),      // Custom basic info value
    CustomBasicInfoLabel(String), // Renaming custom basic info
    CustomAttribute(String),      // Custom attribute score
    CustomAttributeLabel(String), // Renaming custom attribute
    CustomCombat(String),         // Custom combat value
    CustomCombatLabel(String),    // Renaming custom combat
}

// ============================================================================
// Tab Bar Components
// ============================================================================

/// Marker for the tab bar container
#[derive(Component)]
pub struct TabBar;

/// Marker for a tab button
#[derive(Component)]
pub struct TabButton {
    pub tab: AppTab,
}

// ============================================================================
// Character Screen Components
// ============================================================================

/// Marker for the character screen root
#[derive(Component)]
pub struct CharacterScreenRoot;

// ============================================================================
// DnD Info Screen Components
// ============================================================================

/// Marker for the DnD info screen root
#[derive(Component)]
pub struct DndInfoScreenRoot;

/// Marker for scrollable info content
#[derive(Component)]
pub struct InfoScrollContent;

/// Marker for character list panel
#[derive(Component)]
pub struct CharacterListPanel;

/// Marker for character list item
#[derive(Component)]
pub struct CharacterListItem {
    pub index: usize,
}

/// Marker for the main character stats panel
#[derive(Component)]
pub struct CharacterStatsPanel;

/// Which group can be edited
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GroupType {
    BasicInfo,
    Attributes,
    Combat,
    SavingThrows,
    Skills,
}

/// Marker for stat group container (e.g., "Basic Info", "Attributes", etc.)
#[derive(Component)]
pub struct StatGroup {
    pub name: String,
    pub group_type: GroupType,
}

/// Marker for group edit button
#[derive(Component)]
pub struct GroupEditButton {
    pub group_type: GroupType,
}

/// Marker for group add button (plus icon, shown in edit mode)
#[derive(Component)]
pub struct GroupAddButton {
    pub group_type: GroupType,
}

/// Marker for delete entry button (shown in edit mode)
#[derive(Component)]
pub struct DeleteEntryButton {
    pub group_type: GroupType,
    pub entry_id: String, // The key/name of the entry to delete
}

/// Marker for editable label (attribute name, skill name, etc.)
#[derive(Component)]
pub struct EditableLabel {
    pub group_type: GroupType,
    pub label_id: String, // e.g., "strength", "acrobatics"
}

/// Resource for tracking which groups are in edit mode
#[derive(Resource, Default)]
pub struct GroupEditState {
    pub editing_groups: std::collections::HashSet<GroupType>,
}

/// Resource for tracking when adding a new entry to a group
#[derive(Resource, Default)]
pub struct AddingEntryState {
    /// Which group we're adding to (None if not adding)
    pub adding_to: Option<GroupType>,
    /// The name being typed for the new entry
    pub new_entry_name: String,
    /// The value being typed for the new entry (for groups that need a value)
    pub new_entry_value: String,
    /// Whether the value field is focused (false = name field, true = value field)
    pub value_focused: bool,
}

/// Marker for the new entry input field (for the name)
#[derive(Component)]
pub struct NewEntryInput {
    pub group_type: GroupType,
}

/// Marker for the new entry value input field
#[derive(Component)]
pub struct NewEntryValueInput {
    pub group_type: GroupType,
}

/// Marker for the confirm button when adding new entry
#[derive(Component)]
pub struct NewEntryConfirmButton {
    pub group_type: GroupType,
}

/// Marker for the cancel button when adding new entry
#[derive(Component)]
pub struct NewEntryCancelButton {
    pub group_type: GroupType,
}

/// Marker for a clickable label that can be renamed in edit mode
#[derive(Component)]
pub struct EditableLabelButton {
    pub field: EditingField,
    pub current_name: String,
}

/// Marker for the text inside an editable label
#[derive(Component)]
pub struct EditableLabelText {
    pub field: EditingField,
}

/// Marker for editable stat field
#[derive(Component)]
pub struct StatField {
    pub field: EditingField,
    pub is_numeric: bool,
}

/// Marker for editable text field value display
#[derive(Component)]
pub struct StatFieldValue {
    pub field: EditingField,
}

/// Marker for the save button
#[derive(Component)]
pub struct SaveButton;

/// Marker for the new character button
#[derive(Component)]
pub struct NewCharacterButton;

/// Marker for the "Roll All Stats" button (rolls d20 for all attributes)
#[derive(Component)]
pub struct RollAllStatsButton;

/// Marker for individual attribute dice roll button
#[derive(Component)]
pub struct RollAttributeButton {
    pub attribute: String,
}

/// Marker for skill row
#[derive(Component)]
pub struct SkillRow {
    pub skill_name: String,
}

/// Marker for proficiency checkbox (button-based)
#[derive(Component)]
pub struct ProficiencyCheckbox {
    pub target: ProficiencyTarget,
}

/// What the proficiency checkbox targets
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProficiencyTarget {
    Skill(String),
    SavingThrow(String),
}

/// Marker for expertise checkbox
#[derive(Component)]
pub struct ExpertiseCheckbox {
    pub skill_name: String,
}

/// Marker for saving throw row
#[derive(Component)]
pub struct SavingThrowRow {
    pub ability: String,
}

/// Marker for character list item text (for showing asterisk on unsaved changes)
#[derive(Component)]
pub struct CharacterListItemText {
    pub index: usize,
    pub base_name: String,
}

/// Marker for the scrollable content area
#[derive(Component)]
pub struct ScrollableContent;

/// Resource for tracking text input state
#[derive(Resource, Default)]
pub struct TextInputState {
    pub active_field: Option<EditingField>,
    pub current_text: String,
    pub cursor_position: usize,
    /// Fields that have been modified since last save
    pub modified_fields: std::collections::HashSet<EditingField>,
}

// ============================================================================
// Dice Roller Components (existing)
// ============================================================================

/// Marker component for the results text display
#[derive(Component)]
pub struct ResultsText;

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

/// Marker for the dice roller view root (to show/hide)
#[derive(Component)]
pub struct DiceRollerRoot;

// ============================================================================
// Quick Roll Panel Components
// ============================================================================

/// Marker for the quick roll panel container
#[derive(Component)]
pub struct QuickRollPanel;

/// Types of quick roll actions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickRollType {
    Skill(String),
    AbilityCheck(String),
    SavingThrow(String),
}

/// Component for quick roll buttons
#[derive(Component)]
pub struct QuickRollButton {
    pub roll_type: QuickRollType,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_zoom_state_default() {
        let zoom = ZoomState::default();
        assert_eq!(zoom.level, 0.3);
        assert_eq!(zoom.min_distance, 4.0);
        assert_eq!(zoom.max_distance, 25.0);
    }

    #[test]
    fn test_zoom_state_get_distance() {
        let zoom = ZoomState::default();
        // At level 0.3: 4.0 + 0.3 * (25.0 - 4.0) = 4.0 + 0.3 * 21.0 = 4.0 + 6.3 = 10.3
        assert!((zoom.get_distance() - 10.3).abs() < 0.01);

        let zoom_min = ZoomState {
            level: 0.0,
            min_distance: 4.0,
            max_distance: 25.0,
        };
        assert_eq!(zoom_min.get_distance(), 4.0);

        let zoom_max = ZoomState {
            level: 1.0,
            min_distance: 4.0,
            max_distance: 25.0,
        };
        assert_eq!(zoom_max.get_distance(), 25.0);
    }
}
