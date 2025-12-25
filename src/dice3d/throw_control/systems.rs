//! Throw Control Systems
//!
//! Contains systems for tracking mouse position, raycasting to the box floor,
//! updating the 3D arrow indicator, and handling the strength slider.

use super::state::*;
use bevy::prelude::*;

use bevy_material_ui::prelude::SliderChangeEvent;

use crate::dice3d::types::DiceContainerStyle;
use crate::dice3d::types::{ContainerShakeAnimation, SettingsState, UiPointerCapture};

fn ray_intersects_aabb(ray_origin: Vec3, ray_dir: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> bool {
    // Slab method.
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;

    for (origin, dir, minv, maxv) in [
        (ray_origin.x, ray_dir.x, aabb_min.x, aabb_max.x),
        (ray_origin.y, ray_dir.y, aabb_min.y, aabb_max.y),
        (ray_origin.z, ray_dir.z, aabb_min.z, aabb_max.z),
    ] {
        if dir.abs() < 1e-6 {
            // Ray parallel to slab: must be within slab to intersect.
            if origin < minv || origin > maxv {
                return false;
            }
            continue;
        }

        let inv = 1.0 / dir;
        let mut t1 = (minv - origin) * inv;
        let mut t2 = (maxv - origin) * inv;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        tmin = tmin.max(t1);
        tmax = tmax.min(t2);
        if tmin > tmax {
            return false;
        }
    }

    // Intersection must be in front of the camera.
    tmax >= 0.0
}

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
    settings_state: Res<crate::dice3d::types::SettingsState>,
    ui_pointer_capture: Res<UiPointerCapture>,
    container_style: Res<DiceContainerStyle>,
) {
    // Don't update when in command input mode or not on dice roller tab
    if command_input.active || ui_state.active_tab != crate::dice3d::types::AppTab::DiceRoller {
        return;
    }

    // Modal dialog open: treat as not hovering the box.
    if settings_state.show_modal {
        throw_state.mouse_over_box = false;
        return;
    }

    // UI is capturing pointer: prevent click-through into the 3D box.
    if ui_pointer_capture.mouse_captured {
        throw_state.mouse_over_box = false;
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        throw_state.mouse_over_box = false;
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Cast ray from camera through cursor position
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let ray_dir: Vec3 = *ray.direction;

    // First: detect whether the cursor ray intersects the dice container volume.
    // Expand slightly so wall thickness is included.
    let click_margin = 0.2;
    let (container_min, container_max) = match *container_style {
        DiceContainerStyle::Box => (
            Vec3::new(
                BOX_MIN_X - click_margin,
                BOX_FLOOR_Y,
                BOX_MIN_Z - click_margin,
            ),
            Vec3::new(
                BOX_MAX_X + click_margin,
                BOX_TOP_Y,
                BOX_MAX_Z + click_margin,
            ),
        ),
        DiceContainerStyle::Cup => (
            Vec3::new(
                -super::CUP_RADIUS - click_margin,
                BOX_FLOOR_Y,
                -super::CUP_RADIUS - click_margin,
            ),
            Vec3::new(
                super::CUP_RADIUS + click_margin,
                BOX_TOP_Y,
                super::CUP_RADIUS + click_margin,
            ),
        ),
    };
    let is_over_box_volume = ray_intersects_aabb(ray.origin, ray_dir, container_min, container_max);

    // Next: find intersection with the box floor plane (Y = BOX_FLOOR_Y)
    // Ray: P = origin + t * direction
    // Plane: Y = BOX_FLOOR_Y
    // Solve: origin.y + t * direction.y = BOX_FLOOR_Y
    // t = (BOX_FLOOR_Y - origin.y) / direction.y

    if ray.direction.y.abs() < 0.0001 {
        // Ray is parallel to floor, no intersection
        throw_state.mouse_over_box = is_over_box_volume;
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

    // Check footprint + compute target point on the floor.
    let (is_in_footprint, target, max_distance) = match *container_style {
        DiceContainerStyle::Box => {
            let is_in = ThrowControlState::is_point_in_box(intersection);
            let tgt = ThrowControlState::clamp_to_box_floor(intersection);
            (is_in, tgt, (BOX_MAX_X - BOX_MIN_X).abs() * 0.5)
        }
        DiceContainerStyle::Cup => {
            let p = Vec2::new(intersection.x, intersection.z);
            let r = super::CUP_RADIUS.max(0.0001);
            let len = p.length();
            let is_in = len <= r;
            let clamped = if len > r { (p / len) * r } else { p };
            let tgt = Vec3::new(clamped.x, BOX_FLOOR_Y, clamped.y);
            (is_in, tgt, r)
        }
    };

    // Update state
    throw_state.target_point = target;
    // Hover/click should work on the floor and the walls.
    throw_state.mouse_over_box = is_over_box_volume || is_in_footprint;
    throw_state.throw_strength =
        (Vec2::new(target.x, target.z).length() / max_distance.max(0.0001)).clamp(0.0, 1.0);
}

/// System to update the 3D arrow indicator position and rotation
pub fn update_throw_arrow(
    throw_state: Res<ThrowControlState>,
    mut arrow_query: Query<(&mut Transform, &mut Visibility), With<ThrowDirectionArrow>>,
    ui_state: Res<crate::dice3d::types::UiState>,
    shake_anim: Res<ContainerShakeAnimation>,
) {
    for (mut transform, mut visibility) in arrow_query.iter_mut() {
        // Only show arrow on dice roller tab
        if ui_state.active_tab != crate::dice3d::types::AppTab::DiceRoller {
            *visibility = Visibility::Hidden;
            continue;
        }

        // Hide arrow during container shaking so it doesn't distract / look wrong.
        if shake_anim.active {
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
            Transform::from_translation(Vec3::new(0.0, BOX_FLOOR_Y + 0.1, 0.0)),
            Visibility::Visible,
            ThrowDirectionArrow,
        ))
        .with_children(|parent| {
            // Arrow body - rotated to point in +Z and positioned
            parent.spawn((
                Mesh3d(body_mesh),
                MeshMaterial3d(arrow_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.2))
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ));

            // Arrow head - at the tip
            parent.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(arrow_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.5))
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ));
        });
}

/// Apply throw strength when the Material slider changes.
pub fn handle_strength_slider_changes(
    settings_state: Res<SettingsState>,
    mut events: MessageReader<SliderChangeEvent>,
    mut throw_state: ResMut<ThrowControlState>,
    slider_query: Query<(), With<StrengthSlider>>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in events.read() {
        if slider_query.get(event.entity).is_err() {
            continue;
        }

        throw_state.max_strength = event.value.clamp(1.0, 15.0);
    }
}
