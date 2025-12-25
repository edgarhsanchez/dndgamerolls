//! Application settings types and persistence
//!
//! This module handles loading and saving application settings.

use super::DiceType;
use bevy::log::info;
use bevy::prelude::*;
use csscolorparser;
use serde::{Deserialize, Serialize};

use super::database::CharacterDatabase;
use super::ui::{
    ContainerShakeConfig, ShakeCurveBezierHandleKind, ShakeCurveEditMode, ShakeCurvePoint,
};

// ============================================================================
// Dice Roll FX Mapping (hardcoded effects, no customization)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiceRollFxKind {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "fire")]
    Fire,
    #[serde(rename = "electricity")]
    Electricity,
    #[serde(rename = "fireworks")]
    Fireworks,
    #[serde(rename = "explosion")]
    Explosion,
    #[serde(rename = "plasma")]
    Plasma,
}

impl Default for DiceRollFxKind {
    fn default() -> Self {
        Self::None
    }
}

impl DiceRollFxKind {
    pub fn label(&self) -> &'static str {
        match self {
            DiceRollFxKind::None => "None",
            DiceRollFxKind::Fire => "Fire",
            DiceRollFxKind::Electricity => "Electricity",
            DiceRollFxKind::Fireworks => "Fireworks",
            DiceRollFxKind::Explosion => "Explosion",
            DiceRollFxKind::Plasma => "Plasma",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiceRollFxMapping {
    pub die_type: DiceType,
    /// Index by rolled value. Entry 0 is unused (kept for convenience).
    #[serde(default)]
    pub effects_by_value: Vec<DiceRollFxKind>,
}

impl DiceRollFxMapping {
    pub fn new(die_type: DiceType) -> Self {
        let mut effects_by_value = vec![DiceRollFxKind::None; (die_type.max_value() + 1) as usize];
        effects_by_value[0] = DiceRollFxKind::None;
        Self { die_type, effects_by_value }
    }

    pub fn get(&self, value: u32) -> DiceRollFxKind {
        let idx = value as usize;
        self.effects_by_value
            .get(idx)
            .copied()
            .unwrap_or(DiceRollFxKind::None)
    }

    pub fn set(&mut self, value: u32, kind: DiceRollFxKind) {
        let idx = value as usize;
        if idx == 0 {
            return;
        }
        if idx >= self.effects_by_value.len() {
            self.effects_by_value.resize(idx + 1, DiceRollFxKind::None);
        }
        self.effects_by_value[idx] = kind;
    }

    pub fn normalize_len(&mut self) {
        let want = (self.die_type.max_value() + 1) as usize;
        if self.effects_by_value.len() != want {
            self.effects_by_value.resize(want, DiceRollFxKind::None);
        }
        if !self.effects_by_value.is_empty() {
            self.effects_by_value[0] = DiceRollFxKind::None;
        }
    }
}

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

        // Allow simple named colors for convenience (case-insensitive).
        // Intended primarily for theme seed input (e.g. "red", "blue").
        if !input.is_empty()
            && !input.starts_with('#')
            && !input.contains(':')
            && !input.contains(',')
        {
            if let Some(named) = Self::parse_named(input) {
                return Some(named);
            }
        }

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

    fn parse_named(input: &str) -> Option<Self> {
        // Support standard CSS color keywords (e.g. "rebeccapurple", "cornflowerblue",
        // "lightgray", etc.) via csscolorparser.
        //
        // Also accept common user formatting like spaces/underscores/hyphens:
        // "Light Gray" / "light-gray" / "light_gray" -> "lightgray".
        let cleaned: String = input
            .trim()
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '_' && *c != '-')
            .collect::<String>()
            .to_ascii_lowercase();

        let parsed = csscolorparser::Color::from_html(cleaned.as_str()).ok()?;

        Some(Self {
            a: parsed.a,
            r: parsed.r,
            g: parsed.g,
            b: parsed.b,
        })
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

    /// Optional theme seed override (hex string like "#FFAABBCC").
    ///
    /// When `None`, the app uses the default `MaterialTheme`.
    #[serde(default)]
    pub theme_seed_hex: Option<String>,

    /// Recently used theme seeds (canonical hex strings like "#FFAABBCC").
    #[serde(default)]
    pub recent_theme_seeds: Vec<String>,

    /// Per-die visual/physics scale overrides.
    ///
    /// Defaults match the built-in dice scale values so the current sizing behavior remains
    /// unchanged unless the user adjusts sliders.
    #[serde(default)]
    pub dice_scales: DiceScaleSettings,

    /// Per-die/per-face mapping for which hardcoded FX should play on a specific roll value.
    ///
    /// Entries are optional; missing dice types default to "None" for all faces.
    #[serde(default)]
    pub dice_roll_fx_mappings: Vec<DiceRollFxMapping>,

    /// Opacity for the dice surface FX shell (0..1).
    ///
    /// This affects the translucent shader surface used for electric/fire/atomic/custom FX.
    #[serde(default = "default_dice_fx_surface_opacity")]
    pub dice_fx_surface_opacity: f32,

    /// Multiplier for the plume FX height (fire/atomic).
    #[serde(default = "default_dice_fx_plume_height_multiplier")]
    pub dice_fx_plume_height_multiplier: f32,

    /// Multiplier for the plume FX radius (fire/atomic).
    #[serde(default = "default_dice_fx_plume_radius_multiplier")]
    pub dice_fx_plume_radius_multiplier: f32,
}

fn default_dice_fx_surface_opacity() -> f32 {
    0.45
}

fn default_dice_fx_plume_height_multiplier() -> f32 {
    1.25
}

fn default_dice_fx_plume_radius_multiplier() -> f32 {
    1.15
}

/// Per-die scale settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiceScaleSettings {
    #[serde(default = "default_d4_scale")]
    pub d4: f32,
    #[serde(default = "default_d6_scale")]
    pub d6: f32,
    #[serde(default = "default_d8_scale")]
    pub d8: f32,
    #[serde(default = "default_d10_scale")]
    pub d10: f32,
    #[serde(default = "default_d12_scale")]
    pub d12: f32,
    #[serde(default = "default_d20_scale")]
    pub d20: f32,
}

fn default_d4_scale() -> f32 {
    DiceType::D4.scale()
}
fn default_d6_scale() -> f32 {
    DiceType::D6.scale()
}
fn default_d8_scale() -> f32 {
    DiceType::D8.scale()
}
fn default_d10_scale() -> f32 {
    DiceType::D10.scale()
}
fn default_d12_scale() -> f32 {
    DiceType::D12.scale()
}
fn default_d20_scale() -> f32 {
    DiceType::D20.scale()
}

impl Default for DiceScaleSettings {
    fn default() -> Self {
        Self {
            d4: default_d4_scale(),
            d6: default_d6_scale(),
            d8: default_d8_scale(),
            d10: default_d10_scale(),
            d12: default_d12_scale(),
            d20: default_d20_scale(),
        }
    }
}

impl DiceScaleSettings {
    /// Global min/max for the slider values.
    ///
    /// These values are absolute world scales applied to each die type.
    /// This ensures every die type shares the same slider range and can reach the same sizes.
    pub const MIN_SCALE: f32 = 0.50;
    pub const MAX_SCALE: f32 = 2.00;

    /// Returns the stored world scale for a die type.
    pub fn scale_for(&self, die_type: DiceType) -> f32 {
        match die_type {
            DiceType::D4 => self.d4,
            DiceType::D6 => self.d6,
            DiceType::D8 => self.d8,
            DiceType::D10 => self.d10,
            DiceType::D12 => self.d12,
            DiceType::D20 => self.d20,
        }
        .clamp(Self::MIN_SCALE, Self::MAX_SCALE)
    }

    /// Sets the stored world scale for a die type.
    pub fn set_scale_for(&mut self, die_type: DiceType, value: f32) {
        let value = value.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
        match die_type {
            DiceType::D4 => self.d4 = value,
            DiceType::D6 => self.d6 = value,
            DiceType::D8 => self.d8 = value,
            DiceType::D10 => self.d10 = value,
            DiceType::D12 => self.d12 = value,
            DiceType::D20 => self.d20 = value,
        }
    }

    /// Enforce the invariant that D4 is the smallest die and D20 is the largest.
    ///
    /// Other dice are clamped into the inclusive range [d4..d20].
    pub fn normalize(&mut self) {
        // Keep values clamped to the global slider range.
        self.d4 = self.d4.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
        self.d6 = self.d6.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
        self.d8 = self.d8.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
        self.d10 = self.d10.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
        self.d12 = self.d12.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
        self.d20 = self.d20.clamp(Self::MIN_SCALE, Self::MAX_SCALE);
    }
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
            theme_seed_hex: None,
            recent_theme_seeds: Vec::new(),
            dice_scales: DiceScaleSettings::default(),

            dice_roll_fx_mappings: Vec::new(),
            dice_fx_surface_opacity: default_dice_fx_surface_opacity(),
            dice_fx_plume_height_multiplier: default_dice_fx_plume_height_multiplier(),
            dice_fx_plume_radius_multiplier: default_dice_fx_plume_radius_multiplier(),
        }
    }
}

impl AppSettings {
    const SETTINGS_DB_KEY: &'static str = "app_settings";

    pub fn roll_fx_for(&self, die_type: DiceType, value: u32) -> DiceRollFxKind {
        if value == 0 {
            return DiceRollFxKind::None;
        }

        self.dice_roll_fx_mappings
            .iter()
            .find(|m| m.die_type == die_type)
            .map(|m| m.get(value))
            .unwrap_or(DiceRollFxKind::None)
    }

    pub fn ensure_roll_fx_mapping_mut(&mut self, die_type: DiceType) -> &mut DiceRollFxMapping {
        if let Some(idx) = self.dice_roll_fx_mappings.iter().position(|m| m.die_type == die_type) {
            self.dice_roll_fx_mappings[idx].normalize_len();
            return &mut self.dice_roll_fx_mappings[idx];
        }
        self.dice_roll_fx_mappings.push(DiceRollFxMapping::new(die_type));
        let idx = self.dice_roll_fx_mappings.len() - 1;
        &mut self.dice_roll_fx_mappings[idx]
    }

    /// Load settings from SurrealDB.
    pub fn load() -> Self {
        match CharacterDatabase::open() {
            Ok(db) => match db.get_setting::<AppSettings>(Self::SETTINGS_DB_KEY) {
                Ok(Some(settings)) => {
                    info!(
                        "Loaded settings from SurrealDB at {:?} (background={})",
                        db.db_path,
                        settings.background_color.to_hex()
                    );
                    return settings;
                }
                Ok(None) => {
                    info!(
                        "No persisted settings found in SurrealDB at {:?}; using defaults",
                        db.db_path
                    );
                    return Self::default();
                }
                Err(e) => {
                    warn!(
                        "Failed to load settings from SurrealDB at {:?}: {}; using defaults",
                        db.db_path, e
                    );
                    return Self::default();
                }
            },
            Err(e) => {
                warn!(
                    "Failed to open SurrealDB for settings ({}); using defaults",
                    e
                );
            }
        }

        // If the DB cannot be opened (or isn't writable), fall back to defaults.
        // We intentionally do not read/write any JSON files for persistence.
        Self::default()
    }

    /// Load settings from an already-open database.
    pub fn load_from_db(db: &CharacterDatabase) -> Result<Option<Self>, String> {
        db.get_setting::<AppSettings>(Self::SETTINGS_DB_KEY)
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

    /// Theme seed hex input ("#AARRGGBB"). Empty means "use default theme".
    pub theme_seed_input_text: String,

    /// Parsed theme seed override derived from `theme_seed_input_text`.
    pub editing_theme_seed_override: Option<ColorSetting>,

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

    /// Temporary per-die scales being edited in the modal.
    pub editing_dice_scales: DiceScaleSettings,

    /// Editing values for per-die/per-face roll FX mappings (applied on OK).
    pub editing_dice_roll_fx_mappings: Vec<DiceRollFxMapping>,

    /// Editing values for global Dice FX visuals (applied on OK).
    pub editing_dice_fx_surface_opacity: f32,
    pub editing_dice_fx_plume_height_multiplier: f32,
    pub editing_dice_fx_plume_radius_multiplier: f32,
}

impl Default for SettingsState {
    fn default() -> Self {
        // Avoid doing any database I/O in `Default`.
        // Settings are loaded during Startup once the DB resource is initialized.
        let settings = AppSettings::default();
        let character_sheet_editing_die = settings.character_sheet_default_die;
        let quick_roll_editing_die = settings.quick_roll_default_die;
        let default_roll_uses_shake_editing = settings.default_roll_uses_shake;
        let editing_color = settings.background_color.clone();
        let editing_highlight_color = settings.dice_box_highlight_color.clone();
        let editing_shake_config = settings.shake_config.to_runtime();
        let last_saved_shake_config = settings.shake_config.clone();
        let editing_dice_scales = settings.dice_scales.clone();

        let editing_dice_roll_fx_mappings = settings.dice_roll_fx_mappings.clone();

        let editing_dice_fx_surface_opacity = settings.dice_fx_surface_opacity;
        let editing_dice_fx_plume_height_multiplier = settings.dice_fx_plume_height_multiplier;
        let editing_dice_fx_plume_radius_multiplier = settings.dice_fx_plume_radius_multiplier;

        Self {
            settings,
            is_modified: false,
            show_modal: false,
            modal_kind: ActiveModalKind::None,
            editing_color,
            editing_highlight_color,
            color_input_text: String::new(),
            highlight_input_text: String::new(),
            theme_seed_input_text: String::new(),
            editing_theme_seed_override: None,
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
            editing_dice_scales,

            editing_dice_roll_fx_mappings,
            editing_dice_fx_surface_opacity,
            editing_dice_fx_plume_height_multiplier,
            editing_dice_fx_plume_radius_multiplier,
        }
    }
}

/// Which Dice FX parameter a slider/label controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiceFxParamKind {
    SurfaceOpacity,
    PlumeHeight,
    PlumeRadius,
}

/// Marker for Dice FX parameter sliders.
#[derive(Component, Clone, Copy)]
pub struct DiceFxParamSlider {
    pub kind: DiceFxParamKind,
}

/// Marker for Dice FX parameter value labels.
#[derive(Component, Clone, Copy)]
pub struct DiceFxParamValueLabel {
    pub kind: DiceFxParamKind,
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

/// Marker for theme seed text input
#[derive(Component)]
pub struct ThemeSeedTextInput;

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

/// Marker for a per-die/per-face roll-FX mapping select.
#[derive(Component, Clone, Copy)]
pub struct DiceRollFxMappingSelect {
    pub die_type: DiceType,
    pub value: u32,
}

/// Marker for dice scale slider.
#[derive(Component, Clone, Copy)]
pub struct DiceScaleSlider {
    pub die_type: DiceType,
}

/// Marker for dice scale value labels.
#[derive(Component, Clone, Copy)]
pub struct DiceScaleValueLabel {
    pub die_type: DiceType,
}

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

    fn u8f(v: u8) -> f32 {
        v as f32 / 255.0
    }

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

    #[test]
    fn test_color_setting_parse_named_color_css() {
        let color = ColorSetting::parse("rebeccapurple").unwrap();
        assert!((color.a - 1.0).abs() < 0.01);
        // RebeccaPurple is #663399
        assert!((color.r - (0x66 as f32 / 255.0)).abs() < 0.02);
        assert!((color.g - (0x33 as f32 / 255.0)).abs() < 0.02);
        assert!((color.b - (0x99 as f32 / 255.0)).abs() < 0.02);
    }

    #[test]
    fn test_color_setting_parse_named_color_with_spaces() {
        let color = ColorSetting::parse("Light Gray").unwrap();
        assert!((color.a - 1.0).abs() < 0.01);
        // Just sanity-check it's a light-ish gray.
        assert!(color.r > 0.6 && color.g > 0.6 && color.b > 0.6);
    }

    #[test]
    fn test_color_setting_hex_to_color() {
        let setting = ColorSetting::parse("#80FF8844").unwrap();
        let color = setting.to_color();
        let srgba = color.to_srgba();

        assert!((srgba.alpha - u8f(0x80)).abs() < 0.000_001);
        assert!((srgba.red - u8f(0xFF)).abs() < 0.000_001);
        assert!((srgba.green - u8f(0x88)).abs() < 0.000_001);
        assert!((srgba.blue - u8f(0x44)).abs() < 0.000_001);
    }

    #[test]
    fn test_color_setting_color_to_hex() {
        let color = Color::srgba(u8f(0xFF), u8f(0x88), u8f(0x44), u8f(0x80));
        let setting = ColorSetting::from_color(color);
        assert_eq!(setting.to_hex(), "#80FF8844");
    }
}
