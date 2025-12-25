//! Systems for dice box hover highlighting.

use bevy::color::LinearRgba;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

use crate::dice3d::throw_control::ThrowControlState;
use crate::dice3d::types::{
    AppTab, ContainerOriginalEmissive, DiceContainerColliderGuide, DiceContainerVisualPart,
    DiceContainerVisualRoot, SettingsState, UiState,
};

fn collect_descendants_with_material(
    entity: Entity,
    children_query: &Query<&Children>,
    material_query: &Query<
        &MeshMaterial3d<StandardMaterial>,
        (
            Without<DiceContainerVisualPart>,
            Without<DiceContainerColliderGuide>,
        ),
    >,
    out: &mut Vec<Entity>,
) {
    if material_query.get(entity).is_ok() {
        out.push(entity);
    }

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        collect_descendants_with_material(child, children_query, material_query, out);
    }
}

/// Update the dice container (box/cup) material based on hover state and settings.
pub fn update_dice_box_highlight(
    mut commands: Commands,
    ui_state: Res<UiState>,
    throw_state: Res<ThrowControlState>,
    settings_state: Res<SettingsState>,
    roots: Query<Entity, With<DiceContainerVisualRoot>>,
    children_query: Query<&Children>,
    material_query: Query<
        &MeshMaterial3d<StandardMaterial>,
        (
            Without<DiceContainerVisualPart>,
            Without<DiceContainerColliderGuide>,
        ),
    >,
    mut part_query: Query<
        (
            Entity,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&ContainerOriginalEmissive>,
        ),
        With<DiceContainerVisualPart>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // Tag container mesh/material entities once their scenes are spawned.
    // (GLTF scene instances spawn asynchronously, so we keep trying until we find material entities.)
    for root in &roots {
        let mut candidates: Vec<Entity> = Vec::new();
        collect_descendants_with_material(root, &children_query, &material_query, &mut candidates);

        for entity in candidates {
            commands.entity(entity).insert(DiceContainerVisualPart);
        }
    }

    let highlight_color = settings_state.settings.dice_box_highlight_color.to_color();
    let highlight_emissive = LinearRgba::from(highlight_color) * 3.0;

    // Apply highlight by setting emissive on per-entity cloned StandardMaterials.
    for (entity, mut mat_handle, original) in part_query.iter_mut() {
        // Ensure each part has its own material handle so we don't mutate shared glTF materials.
        let original_emissive = if let Some(original) = original {
            original.0
        } else {
            let Some(existing) = materials.get(&mat_handle.0) else {
                continue;
            };

            let cloned = existing.clone();
            let orig = cloned.emissive;
            let new_handle = materials.add(cloned);
            *mat_handle = MeshMaterial3d(new_handle);
            commands
                .entity(entity)
                .insert(ContainerOriginalEmissive(orig));
            orig
        };

        let Some(mat) = materials.get_mut(&mat_handle.0) else {
            continue;
        };

        if hovered > 0.5 {
            mat.emissive = highlight_emissive;
        } else {
            mat.emissive = original_emissive;
        }
    }
}
