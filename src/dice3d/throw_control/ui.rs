//! Throw Control UI
//!
//! Contains functions for spawning the strength slider UI.

use super::state::*;
use bevy::prelude::*;

/// Color constants for the strength slider
const SLIDER_TRACK_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 0.7);
const SLIDER_HANDLE_COLOR: Color = Color::srgba(0.9, 0.4, 0.2, 0.9);

/// Spawn the strength slider UI next to the zoom slider
/// This should be called from the main setup function
pub fn spawn_strength_slider(commands: &mut Commands, throw_state: &ThrowControlState) {
    // Strength slider on the lower left, next to zoom slider
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(60.0), // Offset from zoom slider (at 20px)
                    bottom: Val::Px(60.0),
                    width: Val::Px(30.0),
                    height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            StrengthSliderContainer,
            crate::dice3d::types::DiceRollerRoot, // Part of dice roller view
        ))
        .with_children(|parent| {
            // "⚡" label at top (max strength)
            parent.spawn(TextBundle::from_section(
                "S",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgba(0.9, 0.6, 0.3, 0.9),
                    ..default()
                },
            ));

            // Slider track
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(8.0),
                            height: Val::Px(160.0),
                            margin: UiRect::vertical(Val::Px(5.0)),
                            ..default()
                        },
                        background_color: SLIDER_TRACK_COLOR.into(),
                        ..default()
                    },
                    StrengthSliderTrack,
                ))
                .with_children(|track| {
                    // Slider handle - position based on max_strength (inverted: top = max)
                    // max_strength ranges from 1.0 to 10.0, map to 0-100%
                    let normalized = (throw_state.max_strength - 1.0) / 9.0;
                    let handle_pos = (1.0 - normalized) * 100.0; // Invert so top = max

                    track.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                left: Val::Px(-6.0),
                                top: Val::Percent(handle_pos),
                                ..default()
                            },
                            background_color: SLIDER_HANDLE_COLOR.into(),
                            ..default()
                        },
                        StrengthSliderHandle,
                    ));
                });

            // "s" label at bottom (min strength)
            parent.spawn(TextBundle::from_section(
                "s",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgba(0.7, 0.5, 0.3, 0.7),
                    ..default()
                },
            ));
        });
}
