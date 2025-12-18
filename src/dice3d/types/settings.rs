//! Application settings types and persistence
//!
//! This module handles loading and saving application settings.

use super::DiceType;
use bevy::log::info;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::database::CharacterDatabase;
use super::ui::{
    ContainerShakeConfig, ShakeCurveBezierHandleKind, ShakeCurveEditMode, ShakeCurvePoint,
};

// ============================================================================
// Persistent Shake Curve Settings
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShakeCurvePointSetting {
    pub id: u64,
    pub t: f32,
    pub value: f32,

    #[serde(default)]
    pub in_handle: Option<[f32; 2]>,
    #[serde(default)]
    pub out_handle: Option<[f32; 2]>,
}

impl ShakeCurvePointSetting {
    pub fn from_runtime(p: &ShakeCurvePoint) -> Self {
        Self {
            id: p.id,
            t: p.t,
            value: p.value,
            in_handle: p.in_handle.map(|v| [v.x, v.y]),
            out_handle: p.out_handle.map(|v| [v.x, v.y]),
        }
    }

    pub fn to_runtime(&self) -> ShakeCurvePoint {
        ShakeCurvePoint {
            id: self.id,
            t: self.t,
            value: self.value,
            in_handle: self.in_handle.map(|a| Vec2::new(a[0], a[1])),
            out_handle: self.out_handle.map(|a| Vec2::new(a[0], a[1])),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShakeConfigSetting {
    #[serde(default = "default_shake_distance")]
    pub distance: f32,

    #[serde(default = "default_shake_speed")]
    pub speed: f32,

    #[serde(default = "default_shake_speed_fine")]
    pub speed_fine: f32,

    #[serde(default = "default_shake_duration_seconds")]
    pub duration_seconds: f32,

    #[serde(default)]
    pub curve_points_x: Vec<ShakeCurvePointSetting>,
    #[serde(default)]
    pub curve_points_y: Vec<ShakeCurvePointSetting>,
    #[serde(default)]
    pub curve_points_z: Vec<ShakeCurvePointSetting>,

    #[serde(default)]
    pub next_curve_point_id: u64,
}

fn default_shake_distance() -> f32 {
    ContainerShakeConfig::default().distance
}
fn default_shake_speed() -> f32 {
    ContainerShakeConfig::default().speed
}
fn default_shake_speed_fine() -> f32 {
    ContainerShakeConfig::default().speed_fine
}
fn default_shake_duration_seconds() -> f32 {
    ContainerShakeConfig::default().duration_seconds
}

impl Default for ShakeConfigSetting {
    fn default() -> Self {
        Self::from_runtime(&ContainerShakeConfig::default())
    }
}

impl ShakeConfigSetting {
    pub fn from_runtime(cfg: &ContainerShakeConfig) -> Self {
        Self {
            distance: cfg.distance,
            speed: cfg.speed,
            speed_fine: cfg.speed_fine,
            duration_seconds: cfg.duration_seconds,
            curve_points_x: cfg
                .curve_points_x
                .iter()
                .map(ShakeCurvePointSetting::from_runtime)
                .collect(),
            curve_points_y: cfg
                .curve_points_y
                .iter()
                .map(ShakeCurvePointSetting::from_runtime)
                .collect(),
            curve_points_z: cfg
                .curve_points_z
                .iter()
                .map(ShakeCurvePointSetting::from_runtime)
                .collect(),
            next_curve_point_id: cfg.next_curve_point_id,
        }
    }

    pub fn to_runtime(&self) -> ContainerShakeConfig {
        ContainerShakeConfig {
            distance: self.distance,
            speed: self.speed,
            speed_fine: self.speed_fine,
            duration_seconds: self.duration_seconds,
            curve_points_x: self.curve_points_x.iter().map(|p| p.to_runtime()).collect(),
            curve_points_y: self.curve_points_y.iter().map(|p| p.to_runtime()).collect(),
            curve_points_z: self.curve_points_z.iter().map(|p| p.to_runtime()).collect(),
            next_curve_point_id: self.next_curve_point_id,
        }
    }
}

/// Dice type setting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiceTypeSetting {
    #[serde(rename = "d4")]
    D4,
    #[serde(rename = "d6")]
    D6,
    #[serde(rename = "d8")]
    D8,
    #[serde(rename = "d10")]
    D10,
    #[serde(rename = "d12")]
    D12,
    #[serde(rename = "d20")]
    D20,
}

impl Default for DiceTypeSetting {
    fn default() -> Self {
        Self::D20
    }
}

impl DiceTypeSetting {
    pub fn to_dice_type(self) -> DiceType {
        match self {
            Self::D4 => DiceType::D4,
            Self::D6 => DiceType::D6,
            Self::D8 => DiceType::D8,
            Self::D10 => DiceType::D10,
            Self::D12 => DiceType::D12,
            Self::D20 => DiceType::D20,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::D4 => "d4",
            Self::D6 => "d6",
            Self::D8 => "d8",
            Self::D10 => "d10",
            Self::D12 => "d12",
            Self::D20 => "d20",
        }
    }
}

/// Simple serializable RGBA color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSetting {
    #[serde(default)]
    pub a: f32,
    #[serde(default)]
    pub r: f32,
    #[serde(default)]
    pub g: f32,
    #[serde(default)]
    pub b: f32,
}

impl Default for ColorSetting {
    fn default() -> Self {
        // Slightly off-black by default.
        Self {
            a: 1.0,
            r: 0.05,
            g: 0.05,
            b: 0.05,
        }
    }
}

impl ColorSetting {
    pub fn to_color(&self) -> Color {
        Color::srgba(self.r, self.g, self.b, self.a)
    }

    pub fn from_color(color: Color) -> Self {
        let srgba = color.to_srgba();
        Self {
            a: srgba.alpha,
            r: srgba.red,
            g: srgba.green,
            b: srgba.blue,
        }
    }

    /// Parse from various string formats:
    /// - "A:1.0 R:0.5 G:0.3 B:0.2"
    /// - "1.0,0.5,0.3,0.2" (ARGB order)
    /// - "#FF8844" or "#AAFF8844" (hex)
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        // Try hex format first
        if input.starts_with('#') {
            return Self::parse_hex(input);
        }

        // Try "A:value R:value G:value B:value" format
        if input.contains(':') {
            return Self::parse_labeled(input);
        }

        // Try comma-separated format (A,R,G,B)
        if input.contains(',') {
            return Self::parse_csv(input);
        }

        None
    }

    fn parse_hex(input: &str) -> Option<Self> {
        let hex = input.trim_start_matches('#');

        match hex.len() {
            6 => {
                // RGB format: #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
                Some(Self { a: 1.0, r, g, b })
            }
            8 => {
                // ARGB format: #AARRGGBB
                let a = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
                let r = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
                Some(Self { a, r, g, b })
            }
            _ => None,
        }
    }

    fn parse_labeled(input: &str) -> Option<Self> {
        let mut a = 1.0f32;
        let mut r = 0.0f32;
        let mut g = 0.0f32;
        let mut b = 0.0f32;

        for part in input.split_whitespace() {
            if let Some((label, value)) = part.split_once(':') {
                let val: f32 = value.trim_end_matches(',').parse().ok()?;
                match label.to_uppercase().as_str() {
                    "A" => a = val.clamp(0.0, 1.0),
                    "R" => r = val.clamp(0.0, 1.0),
                    "G" => g = val.clamp(0.0, 1.0),
                    "B" => b = val.clamp(0.0, 1.0),
                    _ => {}
                }
            }
        }

        Some(Self { a, r, g, b })
    }

    fn parse_csv(input: &str) -> Option<Self> {
        let parts: Vec<&str> = input.split(',').collect();
        if parts.len() == 4 {
            // ARGB order
            let a: f32 = parts[0].trim().parse().ok()?;
            let r: f32 = parts[1].trim().parse().ok()?;
            let g: f32 = parts[2].trim().parse().ok()?;
            let b: f32 = parts[3].trim().parse().ok()?;
            Some(Self {
                a: a.clamp(0.0, 1.0),
                r: r.clamp(0.0, 1.0),
                g: g.clamp(0.0, 1.0),
                b: b.clamp(0.0, 1.0),
            })
        } else if parts.len() == 3 {
            // RGB order, alpha = 1.0
            let r: f32 = parts[0].trim().parse().ok()?;
            let g: f32 = parts[1].trim().parse().ok()?;
            let b: f32 = parts[2].trim().parse().ok()?;
            Some(Self {
                a: 1.0,
                r: r.clamp(0.0, 1.0),
                g: g.clamp(0.0, 1.0),
                b: b.clamp(0.0, 1.0),
            })
        } else {
            None
        }
    }

    pub fn to_hex(&self) -> String {
        let a = (self.a * 255.0) as u8;
        let r = (self.r * 255.0) as u8;
        let g = (self.g * 255.0) as u8;
        let b = (self.b * 255.0) as u8;
        format!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b)
    }
}

/// Application settings (persisted to SQLite).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub background_color: ColorSetting,

    #[serde(default = "default_dice_box_highlight_color")]
    pub dice_box_highlight_color: ColorSetting,

    /// Location of the draggable slider group panel (logical pixels, from top-left).
    #[serde(default)]
    pub slider_group_position: UiPositionSetting,

    /// Location of the draggable Quick Rolls panel (logical pixels, from top-left).
    #[serde(default = "default_quick_roll_panel_position")]
    pub quick_roll_panel_position: UiPositionSetting,

    /// Location of the draggable Command History panel (logical pixels, from top-left).
    #[serde(default = "default_command_history_panel_position")]
    pub command_history_panel_position: UiPositionSetting,

    /// Location of the draggable Results panel (logical pixels, from top-left).
    #[serde(default = "default_results_panel_position")]
    pub results_panel_position: UiPositionSetting,

    /// Location of the draggable Dice Box Controls panel (logical pixels, from top-left).
    #[serde(default = "default_dice_box_controls_panel_position")]
    pub dice_box_controls_panel_position: UiPositionSetting,

    /// Default die type for character-sheet dice icon rolls.
    #[serde(default)]
    pub character_sheet_default_die: DiceTypeSetting,

    /// Default die type for Quick Rolls (dice view).
    #[serde(default)]
    pub quick_roll_default_die: DiceTypeSetting,

    /// If enabled, any new roll will default to using the container shake action
    /// instead of the directional throw.
    #[serde(default)]
    pub default_roll_uses_shake: bool,

    /// Saved container shake curve/settings.
    #[serde(default)]
    pub shake_config: ShakeConfigSetting,
}

/// Serializable UI position (logical pixels, top-left origin).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiPositionSetting {
    pub x: f32,
    pub y: f32,
}

impl Default for UiPositionSetting {
    fn default() -> Self {
        // Left-middle-ish by default; avoids the command input card at the bottom.
        Self { x: 20.0, y: 220.0 }
    }
}

fn default_dice_box_highlight_color() -> ColorSetting {
    // A bright cyan-ish highlight by default so it is visible on the crystal floor.
    ColorSetting {
        a: 1.0,
        r: 0.15,
        g: 0.85,
        b: 1.0,
    }
}

fn default_quick_roll_panel_position() -> UiPositionSetting {
    // Right side by default.
    UiPositionSetting { x: 1070.0, y: 50.0 }
}

fn default_command_history_panel_position() -> UiPositionSetting {
    // To the left of Quick Rolls by default.
    UiPositionSetting { x: 860.0, y: 50.0 }
}

fn default_results_panel_position() -> UiPositionSetting {
    // Top-left below the tab bar by default.
    UiPositionSetting { x: 10.0, y: 50.0 }
}

fn default_dice_box_controls_panel_position() -> UiPositionSetting {
    // Near the slider group by default.
    UiPositionSetting { x: 20.0, y: 510.0 }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            background_color: ColorSetting::default(),
            dice_box_highlight_color: default_dice_box_highlight_color(),
            slider_group_position: UiPositionSetting::default(),
            quick_roll_panel_position: default_quick_roll_panel_position(),
            command_history_panel_position: default_command_history_panel_position(),
            results_panel_position: default_results_panel_position(),
            dice_box_controls_panel_position: default_dice_box_controls_panel_position(),
            character_sheet_default_die: DiceTypeSetting::default(),
            quick_roll_default_die: DiceTypeSetting::default(),
            default_roll_uses_shake: false,
            shake_config: ShakeConfigSetting::default(),
        }
    }
}

impl AppSettings {
    const SETTINGS_DB_KEY: &'static str = "app_settings";

    /// Load settings from SurrealDB.
    pub fn load() -> Self {
        if let Ok(db) = CharacterDatabase::open() {
            if let Ok(Some(settings)) = db.get_setting::<AppSettings>(Self::SETTINGS_DB_KEY) {
                info!("Loaded settings from SurrealDB");
                return settings;
            }

            return Self::default();
        }

        // If the DB cannot be opened (or isn't writable), fall back to defaults.
        // We intentionally do not read/write any JSON files for persistence.
        Self::default()
    }

    /// Save settings to SurrealDB.
    pub fn save(&self) -> Result<(), String> {
        let db = CharacterDatabase::open()?;
        self.save_to_db(&db)
    }

    pub fn save_to_db(&self, db: &CharacterDatabase) -> Result<(), String> {
        db.set_setting(Self::SETTINGS_DB_KEY, self.clone())?;

        Ok(())
    }
}

/// Tracks which modal dialog is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModalKind {
    None,
    DiceRollerSettings,
    CharacterSheetDiceSettings,
}

/// Resource for runtime settings state
#[derive(Resource)]
pub struct SettingsState {
    pub settings: AppSettings,
    pub is_modified: bool,
    pub show_modal: bool,
    pub modal_kind: ActiveModalKind,
    /// Temporary color being edited in the modal
    pub editing_color: ColorSetting,
    /// Temporary highlight color being edited in the modal
    pub editing_highlight_color: ColorSetting,
    /// Text input for color
    pub color_input_text: String,
    /// Text input for highlight color
    pub highlight_input_text: String,

    /// Editing value for character-sheet dice settings
    pub character_sheet_editing_die: DiceTypeSetting,

    /// Editing value for Quick Rolls die settings
    pub quick_roll_editing_die: DiceTypeSetting,

    /// Editing value for the "default roll uses shake" setting.
    pub default_roll_uses_shake_editing: bool,

    /// Editing value for the dice container shake curve/settings (applied on OK).
    pub editing_shake_config: ContainerShakeConfig,

    /// Selected curve point id in the curve editor (if any).
    pub selected_shake_curve_point_id: Option<u64>,

    /// Currently dragged curve point id in the curve editor (if any).
    pub dragging_shake_curve_point_id: Option<u64>,

    /// Currently dragged Bezier handle (point id + handle kind), if any.
    pub dragging_shake_curve_bezier: Option<(u64, ShakeCurveBezierHandleKind)>,

    /// Shake curve editor mode (None/Add/Delete).
    pub shake_curve_edit_mode: ShakeCurveEditMode,

    /// Axis selection for Add mode.
    pub shake_curve_add_x: bool,
    pub shake_curve_add_y: bool,
    pub shake_curve_add_z: bool,

    /// Text buffer for shake duration input (seconds).
    pub shake_duration_input_text: String,

    /// Snapshot of the last saved shake config (used to avoid saving every frame).
    pub last_saved_shake_config: ShakeConfigSetting,
}

impl Default for SettingsState {
    fn default() -> Self {
        let settings = AppSettings::load();
        let character_sheet_editing_die = settings.character_sheet_default_die;
        let quick_roll_editing_die = settings.quick_roll_default_die;
        let default_roll_uses_shake_editing = settings.default_roll_uses_shake;
        let editing_color = settings.background_color.clone();
        let editing_highlight_color = settings.dice_box_highlight_color.clone();
        let editing_shake_config = settings.shake_config.to_runtime();
        let last_saved_shake_config = settings.shake_config.clone();

        Self {
            settings,
            is_modified: false,
            show_modal: false,
            modal_kind: ActiveModalKind::None,
            editing_color,
            editing_highlight_color,
            color_input_text: String::new(),
            highlight_input_text: String::new(),
            character_sheet_editing_die,
            quick_roll_editing_die,
            default_roll_uses_shake_editing,
            editing_shake_config,
            selected_shake_curve_point_id: None,
            dragging_shake_curve_point_id: None,
            dragging_shake_curve_bezier: None,
            shake_curve_edit_mode: ShakeCurveEditMode::None,
            shake_curve_add_x: true,
            shake_curve_add_y: false,
            shake_curve_add_z: false,
            shake_duration_input_text: "1.0".to_string(),
            last_saved_shake_config,
        }
    }
}

/// Color component for slider interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorComponent {
    Alpha,
    Red,
    Green,
    Blue,
}

// ============================================================================
// Settings UI Components
// ============================================================================

/// Marker for the settings button
#[derive(Component)]
pub struct SettingsButton;

/// Marker for the settings modal overlay
#[derive(Component)]
pub struct SettingsModalOverlay;

/// Marker for the settings modal window
#[derive(Component)]
pub struct SettingsModal;

/// Marker for color preview box
#[derive(Component)]
pub struct ColorPreview;

/// Marker for dice box highlight color preview box
#[derive(Component)]
pub struct HighlightColorPreview;

/// Marker for color slider
#[derive(Component)]
pub struct ColorSlider {
    pub component: ColorComponent,
}

/// Marker for color text input
#[derive(Component)]
pub struct ColorTextInput;

/// Marker for dice box highlight color text input
#[derive(Component)]
pub struct HighlightColorTextInput;

/// Marker for the shake duration (seconds) text input in the shake curve tab.
#[derive(Component)]
pub struct ShakeDurationTextInput;

/// Marker for settings OK button
#[derive(Component)]
pub struct SettingsOkButton;

/// Marker for settings Cancel button
#[derive(Component)]
pub struct SettingsCancelButton;

/// Marker for the switch that controls "default roll uses shake" in the Dice tab.
#[derive(Component)]
pub struct DefaultRollUsesShakeSwitch;

/// Marker for settings Reset Layout button
#[derive(Component)]
pub struct SettingsResetLayoutButton;

// ============================================================================
// Character Sheet Dice Settings UI Components
// ============================================================================

/// Marker for the Character Sheet settings button (gear)
#[derive(Component)]
pub struct CharacterSheetSettingsButton;

/// Marker for the character sheet dice settings modal overlay
#[derive(Component)]
pub struct CharacterSheetSettingsModalOverlay;

/// Marker for the character sheet dice settings modal window
#[derive(Component)]
pub struct CharacterSheetSettingsModal;

/// Marker for the die type select control in the character sheet modal
#[derive(Component)]
pub struct CharacterSheetDieTypeSelect;

/// Marker for the character sheet settings Save button
#[derive(Component)]
pub struct CharacterSheetSettingsSaveButton;

/// Marker for the character sheet settings Cancel button
#[derive(Component)]
pub struct CharacterSheetSettingsCancelButton;

/// Marker for color value labels
#[derive(Component)]
pub struct ColorValueLabel {
    pub component: ColorComponent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_setting_parse_hex_rgb() {
        let color = ColorSetting::parse("#FF8844").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.533).abs() < 0.01);
        assert!((color.b - 0.267).abs() < 0.01);
        assert!((color.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_setting_parse_hex_argb() {
        let color = ColorSetting::parse("#80FF8844").unwrap();
        assert!((color.a - 0.502).abs() < 0.01);
        assert!((color.r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_setting_parse_labeled() {
        let color = ColorSetting::parse("A:0.5 R:1.0 G:0.5 B:0.25").unwrap();
        assert!((color.a - 0.5).abs() < 0.01);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_color_setting_parse_csv() {
        let color = ColorSetting::parse("0.5,1.0,0.5,0.25").unwrap();
        assert!((color.a - 0.5).abs() < 0.01);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_color_setting_to_hex() {
        let color = ColorSetting {
            a: 1.0,
            r: 1.0,
            g: 0.5,
            b: 0.0,
        };
        assert_eq!(color.to_hex(), "#FFFF7F00");
    }
}
