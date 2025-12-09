//! Throw Control Systems
//!
//! Contains systems for tracking mouse position, raycasting to the box floor,
//! updating the 3D arrow indicator, and handling the strength slider.

use super::state::*;
use bevy::prelude::*;

/// System to track mouse position and raycast to find target point on box floor
///
/// This system casts a ray from the camera through the mouse cursor position
/// to find where it intersects with the box floor (Y = BOX_FLOOR_Y plane).
pub fn update_throw_from_mouse(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::dice3d::types::MainCamera>>,
    mut throw_state: ResMut<ThrowControlState>,
    command_input: Res<crate::dice3d::types::CommandInput>,
    ui_state: Res<crate::dice3d::types::UiState>,
) {
    // Don't update when in command input mode or not on dice roller tab
    if command_input.active || ui_state.active_tab != crate::dice3d::types::AppTab::DiceRoller {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        throw_state.mouse_over_box = false;
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    // Cast ray from camera through cursor position
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Find intersection with the box floor plane (Y = BOX_FLOOR_Y)
    // Ray: P = origin + t * direction
    // Plane: Y = BOX_FLOOR_Y
    // Solve: origin.y + t * direction.y = BOX_FLOOR_Y
    // t = (BOX_FLOOR_Y - origin.y) / direction.y

    if ray.direction.y.abs() < 0.0001 {
        // Ray is parallel to floor, no intersection
        throw_state.mouse_over_box = false;
        return;
    }

    let t = (BOX_FLOOR_Y - ray.origin.y) / ray.direction.y;

    if t < 0.0 {
        // Intersection is behind the camera
        throw_state.mouse_over_box = false;
        return;
    }

    // Calculate intersection point
    let intersection = ray.origin + ray.direction * t;

    // Check if intersection is within or near the box
    let is_in_box = ThrowControlState::is_point_in_box(intersection);

    // Clamp to box boundaries for target point
    let target = ThrowControlState::clamp_to_box_floor(intersection);

    // Update state
    throw_state.target_point = target;
    throw_state.mouse_over_box = is_in_box;
    throw_state.throw_strength = ThrowControlState::calculate_strength_from_distance(target);
}

/// System to update the 3D arrow indicator position and rotation
pub fn update_throw_arrow(
    throw_state: Res<ThrowControlState>,
    mut arrow_query: Query<(&mut Transform, &mut Visibility), With<ThrowDirectionArrow>>,
    ui_state: Res<crate::dice3d::types::UiState>,
) {
    for (mut transform, mut visibility) in arrow_query.iter_mut() {
        // Only show arrow on dice roller tab
        if ui_state.active_tab != crate::dice3d::types::AppTab::DiceRoller {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;

        // Position arrow at target point, slightly above floor
        transform.translation = Vec3::new(
            throw_state.target_point.x,
            BOX_FLOOR_Y + 0.1,
            throw_state.target_point.z,
        );

        // Rotate arrow to point from center toward target
        let direction = throw_state.target_point - BOX_CENTER;
        if direction.length() > 0.01 {
            let angle = direction.z.atan2(direction.x);
            // Arrow mesh points in +X by default, rotate to point toward target
            transform.rotation = Quat::from_rotation_y(-angle + std::f32::consts::FRAC_PI_2);
        }

        // Scale arrow based on strength
        let scale = 0.3 + throw_state.throw_strength * 0.5;
        transform.scale = Vec3::new(scale, scale, scale);
    }
}

/// Spawn the 3D arrow indicator mesh
pub fn spawn_throw_arrow(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Create a simple arrow mesh (cone + cylinder)
    let arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.3, 0.1, 0.8),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Arrow body (cylinder pointing in +Z direction)
    let body_mesh = meshes.add(Cylinder::new(0.05, 0.4));

    // Arrow head (cone)
    let head_mesh = meshes.add(Cone {
        radius: 0.12,
        height: 0.25,
    });

    // Spawn arrow as a parent entity with children
    commands
        .spawn((
            PbrBundle {
                transform: Transform::from_translation(Vec3::new(0.0, BOX_FLOOR_Y + 0.1, 0.0)),
                visibility: Visibility::Visible,
                ..default()
            },
            ThrowDirectionArrow,
        ))
        .with_children(|parent| {
            // Arrow body - rotated to point in +Z and positioned
            parent.spawn(PbrBundle {
                mesh: body_mesh,
                material: arrow_material.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.2))
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ..default()
            });

            // Arrow head - at the tip
            parent.spawn(PbrBundle {
                mesh: head_mesh,
                material: arrow_material,
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.5))
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ..default()
            });
        });
}

/// Handle mouse interaction with the strength slider
pub fn handle_strength_slider(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut throw_state: ResMut<ThrowControlState>,
    track_query: Query<(&Node, &GlobalTransform), With<StrengthSliderTrack>>,
    mut handle_query: Query<&mut Style, With<StrengthSliderHandle>>,
    windows: Query<&Window>,
) {
    // Only handle when left mouse button is pressed
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Check if cursor is within the slider track area
    for (node, global_transform) in track_query.iter() {
        let track_rect = node.logical_rect(global_transform);

        // Expand the click area horizontally for easier interaction
        let expanded_rect = bevy::math::Rect {
            min: Vec2::new(track_rect.min.x - 15.0, track_rect.min.y),
            max: Vec2::new(track_rect.max.x + 15.0, track_rect.max.y),
        };

        if expanded_rect.contains(cursor_position) {
            // Calculate level from cursor Y position within track
            let relative_y = cursor_position.y - track_rect.min.y;
            let track_height = track_rect.height();
            let normalized = (relative_y / track_height).clamp(0.0, 1.0);

            // Invert so top = max strength (1.0), bottom = min strength (0.0)
            let inverted = 1.0 - normalized;

            // Map to max_strength range (1.0 to 15.0)
            throw_state.max_strength = 1.0 + inverted * 14.0;

            // Update handle position
            for mut style in handle_query.iter_mut() {
                style.top = Val::Percent(normalized * 100.0);
            }
        }
    }
}
