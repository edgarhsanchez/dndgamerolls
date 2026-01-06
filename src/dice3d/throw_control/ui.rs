//! Throw Control UI
//!
//! Contains functions for spawning the strength slider UI.

use super::state::*;
use bevy::prelude::*;

use bevy_material_ui::prelude::*;
use bevy_material_ui::slider::{spawn_slider_control_with, SliderDirection};

/// Spawn the strength slider UI next to the zoom slider
/// This should be called from the main setup function
pub fn spawn_strength_slider(
    commands: &mut Commands,
    throw_state: &ThrowControlState,
    theme: &MaterialTheme,
) {
    // Strength slider on the left-middle, next to zoom slider
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(60.0), // Offset from zoom slider (at 20px)
                // Keep it away from the bottom command input panel.
                top: Val::Percent(35.0),
                width: Val::Px(30.0),
                height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            crate::dice3d::types::DiceRollerRoot, // Part of dice roller view
            ZIndex(10),
        ))
        .with_children(|parent| {
            // "⚡" label at top (max strength)
            parent.spawn((
                Text::new("S"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(theme.on_surface.with_alpha(0.9)),
            ));

            parent
                .spawn(Node {
                    width: Val::Px(30.0),
                    height: Val::Px(160.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                })
                .with_children(|slot| {
                    let slider = MaterialSlider::new(1.0, 15.0)
                        .with_value(throw_state.max_strength)
                        .vertical()
                        .direction(SliderDirection::EndToStart)
                        .track_height(6.0)
                        .thumb_radius(10.0);
                    spawn_slider_control_with(slot, theme, slider, StrengthSlider);
                });

            // "s" label at bottom (min strength)
            parent.spawn((
                Text::new("s"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface.with_alpha(0.7)),
            ));
        });
}
