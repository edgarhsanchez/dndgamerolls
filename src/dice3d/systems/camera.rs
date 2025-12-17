//! Camera control systems
//!
//! This module contains systems for camera rotation, zoom controls,
//! and zoom slider interaction.

use bevy::prelude::*;

use crate::dice3d::types::*;
use bevy_material_ui::prelude::{MaterialSlider, SliderChangeEvent};

/// System to handle camera rotation and keyboard zoom
pub fn rotate_camera(
    settings_state: Res<SettingsState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut zoom_state: ResMut<ZoomState>,
    mut zoom_slider_query: Query<&mut MaterialSlider, With<ZoomSlider>>,
) {
    if settings_state.show_modal {
        return;
    }

    let rotation_speed = 1.0;
    let zoom_speed = 0.5;

    let mut zoom_changed = false;
    for mut transform in camera_query.iter_mut() {
        let mut angle = 0.0;

        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            angle += rotation_speed * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            angle -= rotation_speed * time.delta_secs();
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
            zoom_state.level = (zoom_state.level - zoom_speed * time.delta_secs()).max(0.0);
            zoom_changed = true;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            zoom_state.level = (zoom_state.level + zoom_speed * time.delta_secs()).min(1.0);
            zoom_changed = true;
        }

        // Apply zoom to camera
        let target_distance = zoom_state.get_distance();
        let current_dir = transform.translation.normalize();
        transform.translation = current_dir * target_distance;
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);

        // Keep the UI slider value in sync with keyboard zoom.
        if zoom_changed {
            for mut slider in zoom_slider_query.iter_mut() {
                slider.value = zoom_state.level.clamp(slider.min, slider.max);
            }
        }
    }
}

/// Apply zoom when the Material slider changes.
pub fn handle_zoom_slider_changes(
    settings_state: Res<SettingsState>,
    mut events: MessageReader<SliderChangeEvent>,
    mut zoom_state: ResMut<ZoomState>,
    slider_query: Query<(), With<ZoomSlider>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in events.read() {
        if slider_query.get(event.entity).is_err() {
            continue;
        }

        zoom_state.level = event.value.clamp(0.0, 1.0);

        for mut cam_transform in camera_query.iter_mut() {
            let target_distance = zoom_state.get_distance();
            let current_dir = cam_transform.translation.normalize();
            cam_transform.translation = current_dir * target_distance;
            *cam_transform = cam_transform.looking_at(Vec3::ZERO, Vec3::Y);
        }
    }
}
