//! UI-related types and components
//!
//! This module contains all UI components: text displays, command input,
//! command history, and zoom slider.

use bevy::prelude::*;

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
