//! Application settings types and persistence
//!
//! This module handles loading and saving application settings from/to settings.json

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Serializable color representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSetting {
    pub a: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Default for ColorSetting {
    fn default() -> Self {
        // Default dark background
        Self {
            a: 1.0,
            r: 0.1,
            g: 0.1,
            b: 0.15,
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

    pub fn to_labeled(&self) -> String {
        format!(
            "A:{:.2} R:{:.2} G:{:.2} B:{:.2}",
            self.a, self.r, self.g, self.b
        )
    }
}

/// Application settings stored in settings.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub background_color: ColorSetting,
}

impl AppSettings {
    const SETTINGS_FILE: &'static str = "settings.json";

    /// Load settings from settings.json, or return defaults if not found
    pub fn load() -> Self {
        let path = PathBuf::from(Self::SETTINGS_FILE);

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(settings) => {
                        println!("Loaded settings from {}", Self::SETTINGS_FILE);
                        return settings;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", Self::SETTINGS_FILE, e);
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read {}: {}", Self::SETTINGS_FILE, e);
                }
            }
        }

        // Return defaults
        Self::default()
    }

    /// Save settings to settings.json
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(Self::SETTINGS_FILE, json)
            .map_err(|e| format!("Failed to write settings: {}", e))?;

        println!("Settings saved to {}", Self::SETTINGS_FILE);
        Ok(())
    }
}

/// Resource for runtime settings state
#[derive(Resource)]
pub struct SettingsState {
    pub settings: AppSettings,
    pub is_modified: bool,
    pub show_modal: bool,
    /// Temporary color being edited in the modal
    pub editing_color: ColorSetting,
    /// Text input for color
    pub color_input_text: String,
    /// Which color component slider is being dragged
    pub dragging_slider: Option<ColorComponent>,
}

impl Default for SettingsState {
    fn default() -> Self {
        let settings = AppSettings::load();
        let editing_color = settings.background_color.clone();
        Self {
            settings,
            is_modified: false,
            show_modal: false,
            editing_color,
            color_input_text: String::new(),
            dragging_slider: None,
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

/// Marker for color slider
#[derive(Component)]
pub struct ColorSlider {
    pub component: ColorComponent,
}

/// Marker for color slider handle
#[derive(Component)]
pub struct ColorSliderHandle {
    pub component: ColorComponent,
}

/// Marker for color slider track
#[derive(Component)]
pub struct ColorSliderTrack {
    pub component: ColorComponent,
}

/// Marker for color text input
#[derive(Component)]
pub struct ColorTextInput;

/// Marker for settings OK button
#[derive(Component)]
pub struct SettingsOkButton;

/// Marker for settings Cancel button
#[derive(Component)]
pub struct SettingsCancelButton;

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
