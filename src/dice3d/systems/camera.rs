//! Camera control systems
//!
//! This module contains systems for camera rotation, zoom controls,
//! and zoom slider interaction.

use bevy::prelude::*;

use crate::dice3d::types::*;

/// System to handle camera rotation and keyboard zoom
pub fn rotate_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut zoom_state: ResMut<ZoomState>,
    mut handle_query: Query<&mut Style, With<ZoomSliderHandle>>,
) {
    let rotation_speed = 1.0;
    let zoom_speed = 0.5;

    for mut transform in camera_query.iter_mut() {
        let mut angle = 0.0;

        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            angle += rotation_speed * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            angle -= rotation_speed * time.delta_seconds();
        }

        if angle != 0.0 {
            let rotation = Quat::from_rotation_y(angle);
            let pos = transform.translation;
            let new_pos = rotation * pos;
            transform.translation = new_pos;
            *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
        }

        // Keyboard zoom with updated limits
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            zoom_state.level = (zoom_state.level - zoom_speed * time.delta_seconds()).max(0.0);
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            zoom_state.level = (zoom_state.level + zoom_speed * time.delta_seconds()).min(1.0);
        }

        // Apply zoom to camera
        let target_distance = zoom_state.get_distance();
        let current_dir = transform.translation.normalize();
        transform.translation = current_dir * target_distance;
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);

        // Update slider handle position
        for mut style in handle_query.iter_mut() {
            style.top = Val::Percent(zoom_state.level * 100.0);
        }
    }
}

/// Handle mouse interaction with the zoom slider
pub fn handle_zoom_slider(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut zoom_state: ResMut<ZoomState>,
    track_query: Query<(&Node, &GlobalTransform), With<ZoomSliderTrack>>,
    mut handle_query: Query<&mut Style, With<ZoomSliderHandle>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
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
            // Calculate zoom level from cursor Y position within track
            let relative_y = cursor_position.y - track_rect.min.y;
            let track_height = track_rect.height();
            let new_level = (relative_y / track_height).clamp(0.0, 1.0);

            zoom_state.level = new_level;

            // Update handle position
            for mut style in handle_query.iter_mut() {
                style.top = Val::Percent(new_level * 100.0);
            }

            // Update camera position
            for mut transform in camera_query.iter_mut() {
                let target_distance = zoom_state.get_distance();
                let current_dir = transform.translation.normalize();
                transform.translation = current_dir * target_distance;
                *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
            }
        }
    }
}
