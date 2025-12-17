//! Systems for dice box hover highlighting.

use bevy::prelude::*;
use bevy::pbr::MeshMaterial3d;

use crate::dice3d::box_highlight::{DiceBoxFloor, DiceBoxHighlightMaterial};
use crate::dice3d::throw_control::ThrowControlState;
use crate::dice3d::types::{AppTab, SettingsState, UiState};

/// Update the dice box floor material based on hover state and settings.
pub fn update_dice_box_highlight(
    ui_state: Res<UiState>,
    throw_state: Res<ThrowControlState>,
    settings_state: Res<SettingsState>,
    floor_query: Query<&MeshMaterial3d<DiceBoxHighlightMaterial>, With<DiceBoxFloor>>,
    mut materials: ResMut<Assets<DiceBoxHighlightMaterial>>,
) {
    if ui_state.active_tab != AppTab::DiceRoller {
        return;
    }

    let hovered = if settings_state.show_modal {
        0.0
    } else if throw_state.mouse_over_box {
        1.0
    } else {
        0.0
    };
    let highlight_color = settings_state.settings.dice_box_highlight_color.to_color();

    for mat_handle in &floor_query {
        let Some(mat) = materials.get_mut(mat_handle) else {
            continue;
        };

        mat.extension.params.hovered = hovered;
        mat.extension.params.strength = 1.0;
        mat.extension.params.set_highlight_color(highlight_color);
    }
}
