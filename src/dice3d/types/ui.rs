//! UI-related types and components
//!
//! This module contains all UI components: text displays, command input,
//! command history, zoom slider, tab navigation, and character screen.

use bevy::prelude::*;

use std::collections::HashMap;

use super::dice::{DiceConfig, DiceType};

use bevy::animation::prelude::{AnimationGraph, AnimationNodeIndex};

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

/// Tracks whether the UI is currently capturing the mouse pointer.
///
/// Used to prevent "click-through" into the 3D scene while interacting with
/// draggable panels and other UI overlays.
#[derive(Resource, Default)]
pub struct UiPointerCapture {
    pub mouse_captured: bool,
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

/// Marker for the app-level tab bar (to distinguish from character sheet tabs)
#[derive(Component)]
pub struct AppTabBar;

/// Marker for a tab button (legacy, kept for compatibility)
#[derive(Component)]
pub struct TabButton {
    pub tab: AppTab,
}

/// Marker for an app-level tab button with index
#[derive(Component)]
pub struct AppTabButton {
    pub index: usize,
}

/// Marker for app tab text (for styling updates)
#[derive(Component)]
pub struct AppTabText;

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

/// Marker component for the Material text field used for dice commands.
#[derive(Component)]
pub struct CommandInputField;

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

/// Marker for the Material slider controlling camera zoom
#[derive(Component)]
pub struct ZoomSlider;

/// Resource controlling how strong the "Shake" action is.
#[derive(Resource)]
pub struct ShakeState {
    /// 0.0 = no shake, 1.0 = max shake.
    pub strength: f32,
}

impl Default for ShakeState {
    fn default() -> Self {
        Self { strength: 0.6 }
    }
}

/// Marker for the Material slider controlling shake strength.
#[derive(Component)]
pub struct ShakeSlider;

/// Marker for the Material slider controlling container shake distance.
#[derive(Component)]
pub struct ShakeDistanceSlider;

/// Marker for the Material slider controlling container shake speed.
#[derive(Component)]
pub struct ShakeSpeedSlider;

// ============================================================================
// Shake Curve Editor (Settings Modal)
// ============================================================================

/// Marker for the shake curve graph container.
#[derive(Component)]
pub struct ShakeCurveGraphRoot;

/// Marker for the inner plot area of the shake curve graph.
///
/// This is inset from the graph border so dots/handles never touch the edges,
/// and all cursor-to-(t,value) math uses this node's size.
#[derive(Component)]
pub struct ShakeCurveGraphPlotRoot;

/// Marker for sampled curve dot (for rendering the line).
#[derive(Component)]
pub struct ShakeCurveGraphDot {
    pub axis: ShakeAxis,
    pub index: usize,
}

/// Draggable point handle in the curve editor.
#[derive(Component)]
pub struct ShakeCurvePointHandle {
    pub id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShakeAxis {
    X,
    Y,
    Z,
}

/// Edit mode for the shake curve graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShakeCurveEditMode {
    #[default]
    None,
    Add,
    Delete,
}

/// Chip to toggle add/delete edit modes.
#[derive(Component, Debug, Clone, Copy)]
pub struct ShakeCurveEditModeChip {
    pub mode: ShakeCurveEditMode,
}

/// Chip to enable/disable adding points to an axis.
#[derive(Component, Debug, Clone, Copy)]
pub struct ShakeCurveAxisChip {
    pub axis: ShakeAxis,
}

#[derive(Debug, Clone, Copy)]
pub struct ShakeCurvePoint {
    pub id: u64,
    pub t: f32,
    pub value: f32,

    /// Optional Bezier handles in normalized curve space.
    ///
    /// Stored as absolute (t, value) positions in the same normalized domain as the points.
    /// `in_handle` is used for the segment coming *into* this point; `out_handle` for the segment
    /// going *out of* this point.
    pub in_handle: Option<Vec2>,
    pub out_handle: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShakeCurveBezierHandleKind {
    In,
    Out,
}

/// Draggable Bezier handle for the currently-selected curve point.
#[derive(Component, Debug, Clone, Copy)]
pub struct ShakeCurveBezierHandle {
    pub point_id: u64,
    pub kind: ShakeCurveBezierHandleKind,
}

/// User-configurable parameters for the container shake animation.
#[derive(Resource, Debug, Clone)]
pub struct ContainerShakeConfig {
    /// Max left/right offset (world units) at shake strength = 1.0.
    pub distance: f32,
    /// 0.0 = slow, 1.0 = fast.
    pub speed: f32,

    /// Fine speed multiplier applied on top of `speed`.
    pub speed_fine: f32,

    /// Total time (seconds) of a single shake, from start (t=0) to finish (t=1).
    pub duration_seconds: f32,

    /// Per-axis curves sampled over the full shake from start (t=0) to finish (t=1).
    ///
    /// Points are in normalized time `t` in [0..1] and `value` in [-1..1].
    pub curve_points_x: Vec<ShakeCurvePoint>,
    pub curve_points_y: Vec<ShakeCurvePoint>,
    pub curve_points_z: Vec<ShakeCurvePoint>,

    /// Monotonic ID generator for new curve points (only used by the UI editor).
    pub next_curve_point_id: u64,
}

impl Default for ContainerShakeConfig {
    fn default() -> Self {
        Self {
            distance: 0.8,
            speed: 0.5,
            speed_fine: 1.0,
            duration_seconds: 1.0,
            curve_points_x: vec![
                ShakeCurvePoint {
                    id: 1,
                    t: 0.0,
                    value: 0.0,
                    in_handle: None,
                    out_handle: None,
                },
                ShakeCurvePoint {
                    id: 2,
                    t: 0.25,
                    value: 1.0,
                    in_handle: None,
                    out_handle: None,
                },
                ShakeCurvePoint {
                    id: 3,
                    t: 0.50,
                    value: -1.0,
                    in_handle: None,
                    out_handle: None,
                },
                ShakeCurvePoint {
                    id: 4,
                    t: 0.75,
                    value: 0.0,
                    in_handle: None,
                    out_handle: None,
                },
                ShakeCurvePoint {
                    id: 5,
                    t: 1.0,
                    value: 0.0,
                    in_handle: None,
                    out_handle: None,
                },
            ],
            curve_points_y: vec![
                ShakeCurvePoint {
                    id: 6,
                    t: 0.0,
                    value: 0.0,
                    in_handle: None,
                    out_handle: None,
                },
                ShakeCurvePoint {
                    id: 7,
                    t: 1.0,
                    value: 0.0,
                    in_handle: None,
                    out_handle: None,
                },
            ],
            curve_points_z: vec![
                ShakeCurvePoint {
                    id: 8,
                    t: 0.0,
                    value: 0.0,
                    in_handle: None,
                    out_handle: None,
                },
                ShakeCurvePoint {
                    id: 9,
                    t: 1.0,
                    value: 0.0,
                    in_handle: None,
                    out_handle: None,
                },
            ],
            next_curve_point_id: 10,
        }
    }
}

/// Runtime animation state for shaking the dice container (box/cup).
#[derive(Resource, Default)]
pub struct ContainerShakeAnimation {
    pub active: bool,
    pub elapsed: f32,
    pub phase: f32,
    pub min_frequency_hz: f32,
    pub max_frequency_hz: f32,
    pub duration: f32,
    pub amplitude: f32,
    pub base_positions: HashMap<Entity, Vec3>,
}

// ============================================================================
// Dice Box Lid Animations (glTF clips)
// ============================================================================

/// Marker on the spawned box glTF scene root entity.
///
/// Used to find an `AnimationPlayer` to drive lid animations.
#[derive(Component)]
pub struct DiceBoxVisualSceneRoot;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DiceBoxLidState {
    Closed,
    #[default]
    Open,
    Closing,
    Opening,
}

#[derive(Debug, Clone)]
pub enum PendingRollRequest {
    /// Re-roll the currently spawned dice in place (left-click in box).
    RerollExisting,
    /// Quick-roll uses a single die and updates modifier display.
    QuickRollSingleDie {
        die_type: DiceType,
        modifier: i32,
        modifier_name: String,
    },
    /// Start a fresh roll by applying the provided config and spawning dice.
    ///
    /// Used by command submit/history and character sheet rolls when the container is a Box.
    StartNewRoll { config: DiceConfig },
}

#[derive(Resource, Default)]
pub struct DiceBoxLidAnimationController {
    pub gltf_handle: Option<Handle<bevy::gltf::Gltf>>,

    pub opening_clip: Option<Handle<AnimationClip>>,
    pub closing_clip: Option<Handle<AnimationClip>>,
    pub idle_opened_clip: Option<Handle<AnimationClip>>,
    pub idle_closed_clip: Option<Handle<AnimationClip>>,

    pub animation_graph: Option<Handle<AnimationGraph>>,
    pub opening_node: Option<AnimationNodeIndex>,
    pub closing_node: Option<AnimationNodeIndex>,
    pub idle_opened_node: Option<AnimationNodeIndex>,
    pub idle_closed_node: Option<AnimationNodeIndex>,

    pub open_duration: f32,
    pub close_duration: f32,
    pub idle_opened_duration: f32,
    pub idle_closed_duration: f32,

    pub animator_entity: Option<Entity>,

    /// Which idle node we most recently started looping.
    ///
    /// Used to avoid re-starting idle every frame (which can cause visible flicker
    /// if a looping clip wraps frequently).
    pub active_idle_node: Option<AnimationNodeIndex>,

    pub lid_state: DiceBoxLidState,
    /// Remaining seconds in the current open/close animation.
    pub state_timer: f32,

    /// When set, a roll is waiting for the lid to close.
    pub pending_roll: Option<PendingRollRequest>,

    /// When set, the next time we are able we should play LidOpening.
    ///
    /// This is queued by the roll-completed event so that if the event fires
    /// before animation assets/nodes are ready, the opening still plays once
    /// the assets load.
    pub pending_open_after_roll: bool,

    #[cfg(debug_assertions)]
    pub debug_last_idle_node: Option<AnimationNodeIndex>,

    #[cfg(debug_assertions)]
    pub debug_last_lid_state: Option<DiceBoxLidState>,

    #[cfg(debug_assertions)]
    pub debug_last_pending_roll_some: Option<bool>,

    #[cfg(debug_assertions)]
    pub debug_last_pending_open_after_roll: Option<bool>,

    #[cfg(debug_assertions)]
    pub debug_logged_player_scan: bool,
}

/// Marker for the dice roller view root (to show/hide)
#[derive(Component)]
pub struct DiceRollerRoot;

/// Root node for the dice-box control buttons (Rotate / Shake).
#[derive(Component)]
pub struct DiceBoxControlsRoot;

/// Button that rotates the dice box view.
#[derive(Component)]
pub struct DiceBoxRotateButton;

/// Button that shakes the dice box.
#[derive(Component)]
pub struct DiceBoxShakeButton;

// ============================================================================
// Dice Container Controls Panel (draggable)
// ============================================================================

/// Root node for the draggable dice container controls panel.
#[derive(Component)]
pub struct DiceBoxControlsPanelRoot;

/// Drag handle for the dice container controls panel.
#[derive(Component)]
pub struct DiceBoxControlsPanelHandle;

/// Internal drag state for the dice container controls panel.
#[derive(Component, Default)]
pub struct DiceBoxControlsPanelDragState {
    pub dragging: bool,
    pub grab_offset: Vec2,
}

/// Rotate camera around the dice container (draggable panel).
#[derive(Component)]
pub struct DiceBoxControlsPanelRotateButton;

/// Shake dice inside the container.
#[derive(Component)]
pub struct DiceBoxShakeBoxButton;

/// Toggle between box and cup container styles.
#[derive(Component)]
pub struct DiceBoxToggleContainerButton;

/// The Text entity that renders the toggle-container icon glyph.
#[derive(Component)]
pub struct DiceBoxToggleContainerIconText;

/// Text node showing current container mode.
#[derive(Component)]
pub struct DiceBoxContainerModeText;

/// Root node for the draggable results panel.
#[derive(Component)]
pub struct ResultsPanelRoot;

/// Drag handle button for the results panel.
#[derive(Component)]
pub struct ResultsPanelHandle;

/// Internal drag state for the results panel.
#[derive(Component, Default)]
pub struct ResultsPanelDragState {
    pub dragging: bool,
    pub grab_offset: Vec2,
}

/// Root node for the draggable slider group panel.
#[derive(Component)]
pub struct SliderGroupRoot;

/// Drag handle button for the slider group panel.
#[derive(Component)]
pub struct SliderGroupHandle;

/// Internal drag state for the slider group panel.
#[derive(Component, Default)]
pub struct SliderGroupDragState {
    pub dragging: bool,
    pub grab_offset: Vec2,
}

/// Root node for the draggable command history panel.
#[derive(Component)]
pub struct CommandHistoryPanelRoot;

/// Drag handle button for the command history panel.
#[derive(Component)]
pub struct CommandHistoryPanelHandle;

/// Internal drag state for the command history panel.
#[derive(Component, Default)]
pub struct CommandHistoryPanelDragState {
    pub dragging: bool,
    pub grab_offset: Vec2,
}

// ============================================================================
// Quick Roll Panel Components
// ============================================================================

/// Marker for the quick roll panel container
#[derive(Component)]
pub struct QuickRollPanel;

/// Drag handle button for the quick roll panel.
#[derive(Component)]
pub struct QuickRollPanelHandle;

/// Internal drag state for the quick roll panel.
#[derive(Component, Default)]
pub struct QuickRollPanelDragState {
    pub dragging: bool,
    pub grab_offset: Vec2,
}

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

// ============================================================================
// Character Sheet Roll UI (dice buttons -> 3D dice roller)
// ============================================================================

/// Dice roll button for a skill check (from the character sheet).
#[derive(Component)]
pub struct RollSkillButton {
    pub skill: String,
}

/// Text node that displays the last roll total for an attribute.
#[derive(Component)]
pub struct AttributeRollResultText {
    pub attribute: String,
}

/// Text node that displays the last roll total for a skill.
#[derive(Component)]
pub struct SkillRollResultText {
    pub skill: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CharacterScreenRollTarget {
    Attribute(String),
    Skill(String),
}

/// Bridges character-sheet dice buttons to the dice roller and back.
///
/// - When a dice icon is clicked on the character sheet, `pending` is set.
/// - When the dice roller finishes, we store the total in the corresponding map.
#[derive(Resource, Default)]
pub struct CharacterScreenRollBridge {
    pub pending: Option<CharacterScreenRollTarget>,
    pub last_attribute_totals: HashMap<String, i32>,
    pub last_skill_totals: HashMap<String, i32>,
    pub last_character_id: Option<i64>,
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
