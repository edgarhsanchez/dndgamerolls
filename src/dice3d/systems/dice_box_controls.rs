//! Dice container controls panel (draggable) and actions.

use bevy::prelude::*;
use bevy_material_ui::prelude::{
    IconButtonClickEvent, MaterialIcon, MaterialTheme, SliderChangeEvent,
};
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::dice3d::systems::DiceSpawnPointsApplied;
use crate::dice3d::throw_control::{
    BOX_HALF_EXTENT, BOX_WALL_HEIGHT, CUP_RADIUS, ORIGINAL_BOX_HALF_EXTENT,
};
use crate::dice3d::types::*;

/// Start/refresh a container shake animation using the current shake settings.
///
use crate::dice3d::{BOX_MODEL_SCENE_PATH, CUP_MODEL_SCENE_PATH};
/// Returns `true` if shaking was started.
pub fn start_container_shake(
    shake_state: &ShakeState,
    shake_config: &ContainerShakeConfig,
    shake_anim: &mut ContainerShakeAnimation,
    container_query: &Query<(Entity, &Transform), With<DiceBox>>,
) -> bool {
    let strength = shake_state.strength.clamp(0.0, 1.0);
    if strength <= 0.001 {
        return false;
    }

    // Start/refresh a quick left/right shake of the *container*.
    // Use continuous motion so Rapier sees sustained kinematic velocity
    // and the dice get pushed by moving walls/floor.
    shake_anim.active = true;
    shake_anim.elapsed = 0.0;
    shake_anim.phase = 0.0;
    shake_anim.amplitude = shake_config.distance.max(0.0) * strength;

    // With the curve editor, offset is:
    //   p(t) = (A) * curve(progress), where progress is [0..1] over duration.
    // Max speed is approximately:
    //   vmax ≈ A * max|d curve / d(progress)| / duration
    const MAX_CONTAINER_SHAKE_SPEED: f32 = 12.0;

    let amplitude = shake_anim.amplitude.max(0.0);

    // Duration is explicitly configured in the shake curve settings.
    let duration_from_ui = shake_config.duration_seconds.max(0.01);

    // Cap max linear speed of the container based on the steepest curve segment.
    let curve_slope = max_abs_curve_slope(&shake_config.curve_points_x)
        .max(max_abs_curve_slope(&shake_config.curve_points_y))
        .max(max_abs_curve_slope(&shake_config.curve_points_z));
    let min_duration_from_speed_cap = if amplitude > 0.0001 {
        (amplitude * curve_slope / MAX_CONTAINER_SHAKE_SPEED).max(0.0)
    } else {
        0.0
    };

    shake_anim.duration = duration_from_ui.max(min_duration_from_speed_cap);
    shake_anim.base_positions.clear();

    for (entity, transform) in container_query.iter() {
        shake_anim
            .base_positions
            .insert(entity, transform.translation);
    }

    true
}

fn max_abs_curve_slope(points: &[ShakeCurvePoint]) -> f32 {
    if points.len() < 2 {
        return 0.0001;
    }

    // Numeric slope estimate to account for Bezier handles.
    let steps: usize = 128;
    let dt = 1.0 / (steps as f32);

    let mut max_slope: f32 = 0.0;
    let mut prev = sample_curve(points, 0.0);
    for i in 1..=steps {
        let t = (i as f32) * dt;
        let v = sample_curve(points, t);
        let slope = ((v - prev) / dt.max(0.0001)).abs();
        max_slope = max_slope.max(slope);
        prev = v;
    }

    max_slope.max(0.0001)
}

fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let omt = 1.0 - u;
    (omt * omt * omt) * p0
        + (3.0 * omt * omt * u) * p1
        + (3.0 * omt * u * u) * p2
        + (u * u * u) * p3
}

fn cubic_bezier_derivative(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let omt = 1.0 - u;
    3.0 * omt * omt * (p1 - p0) + 6.0 * omt * u * (p2 - p1) + 3.0 * u * u * (p3 - p2)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn sample_curve(points: &[ShakeCurvePoint], t01: f32) -> f32 {
    if points.is_empty() {
        return 0.0;
    }
    if points.len() == 1 {
        return points[0].value;
    }

    // Non-looping: t01 maps directly to [0..1].
    let t = t01.clamp(0.0, 1.0);
    let mut tmp: Vec<ShakeCurvePoint> = points.to_vec();
    tmp.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));

    if t <= tmp[0].t {
        return tmp[0].value;
    }
    if t >= tmp[tmp.len() - 1].t {
        return tmp[tmp.len() - 1].value;
    }

    for w in tmp.windows(2) {
        let a = w[0];
        let b = w[1];
        if t >= a.t && t <= b.t {
            let dt = (b.t - a.t).max(0.0001);
            let initial_u = ((t - a.t) / dt).clamp(0.0, 1.0);

            let mut p1 = a
                .out_handle
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 1.0 / 3.0), a.value));
            let mut p2 = b
                .in_handle
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 2.0 / 3.0), b.value));

            p1.x = p1.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p2.x = p2.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p1.y = p1.y.clamp(-1.0, 1.0);
            p2.y = p2.y.clamp(-1.0, 1.0);

            let mut u = initial_u;
            for _ in 0..8 {
                let x = cubic_bezier(a.t, p1.x, p2.x, b.t, u);
                let dx = cubic_bezier_derivative(a.t, p1.x, p2.x, b.t, u);
                if dx.abs() < 1e-5 {
                    break;
                }
                u = (u - (x - t) / dx).clamp(0.0, 1.0);
            }

            return cubic_bezier(a.value, p1.y, p2.y, b.value, u);
        }
    }

    tmp[0].value
}

fn save_controls_panel_position(settings_state: &mut SettingsState, x: f32, y: f32) {
    let slot = &mut settings_state.settings.dice_box_controls_panel_position;
    if (slot.x - x).abs() < 0.5 && (slot.y - y).abs() < 0.5 {
        return;
    }

    slot.x = x;
    slot.y = y;

    settings_state.is_modified = true;
}

/// Drag the dice container controls panel around by grabbing its handle.
pub fn handle_dice_box_controls_panel_drag(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    ui_state: Res<UiState>,
    mut settings_state: ResMut<SettingsState>,
    app_tab_bar: Query<&ComputedNode, With<AppTabBar>>,
    handle_interaction: Query<
        (&Interaction, &ChildOf),
        (With<DiceBoxControlsPanelHandle>, Changed<Interaction>),
    >,
    mut panel_query: Query<
        (&mut Node, &mut DiceBoxControlsPanelDragState, &ComputedNode),
        With<DiceBoxControlsPanelRoot>,
    >,
) {
    if ui_state.active_tab != AppTab::DiceRoller {
        return;
    }

    if settings_state.show_modal {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let tab_bar_height = app_tab_bar
        .iter()
        .next()
        .map(|c| c.size().y.max(0.0))
        .unwrap_or(48.0);

    // Start dragging when the handle is pressed.
    for (interaction, child_of) in handle_interaction.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if !mouse.just_pressed(MouseButton::Left) {
            continue;
        }

        let parent_entity = child_of.parent();
        let Ok((node, mut drag_state, _computed)) = panel_query.get_mut(parent_entity) else {
            continue;
        };

        let left = match node.left {
            Val::Px(v) => v,
            _ => 0.0,
        };
        let top = match node.top {
            Val::Px(v) => v,
            _ => tab_bar_height,
        };

        drag_state.dragging = true;
        drag_state.grab_offset = cursor_position - Vec2::new(left, top);
    }

    // Continue dragging.
    if mouse.pressed(MouseButton::Left) {
        for (mut node, drag_state, computed) in panel_query.iter_mut() {
            if !drag_state.dragging {
                continue;
            }

            let mut new_left = cursor_position.x - drag_state.grab_offset.x;
            let mut new_top = cursor_position.y - drag_state.grab_offset.y;

            let win_w = window.resolution.width();
            let win_h = window.resolution.height();
            let panel_w = computed.size().x.max(1.0);
            let panel_h = computed.size().y.max(1.0);

            new_left = new_left.clamp(0.0, (win_w - panel_w).max(0.0));
            new_top = new_top.clamp(tab_bar_height, (win_h - panel_h).max(tab_bar_height));

            node.left = Val::Px(new_left);
            node.top = Val::Px(new_top);

            save_controls_panel_position(&mut settings_state, new_left, new_top);
        }
    }

    // Stop dragging.
    if mouse.just_released(MouseButton::Left) {
        for (_node, mut drag_state, _computed) in panel_query.iter_mut() {
            drag_state.dragging = false;
        }
    }
}

/// Update the Mode label text to match the current container style.
pub fn sync_dice_container_mode_text(
    style: Res<DiceContainerStyle>,
    theme: Res<MaterialTheme>,
    mut texts: Query<(&mut Text, &mut TextColor), With<DiceBoxContainerModeText>>,
) {
    if !style.is_changed() && !theme.is_changed() {
        return;
    }

    let label = match *style {
        DiceContainerStyle::Box => "Mode: Box",
        DiceContainerStyle::Cup => "Mode: Cup",
    };

    for (mut text, mut color) in texts.iter_mut() {
        **text = label.to_string();
        *color = TextColor(theme.primary);
    }
}

/// Update the toggle-container button's icon glyph to match the current style.
///
/// When in Box mode, show the cup icon (U+EA1B) to indicate switching to Cup.
pub fn sync_dice_container_toggle_icon(
    style: Res<DiceContainerStyle>,
    mut icons: Query<&mut MaterialIcon, With<DiceBoxToggleContainerIconText>>,
) {
    if !style.is_changed() {
        return;
    }

    let new_icon = match *style {
        DiceContainerStyle::Box => MaterialIcon::from_name("emoji_food_beverage")
            .or_else(|| MaterialIcon::from_name("local_bar"))
            .or_else(|| MaterialIcon::from_name("coffee"))
            .or_else(|| MaterialIcon::from_name("swap_horiz")),
        DiceContainerStyle::Cup => MaterialIcon::from_name("swap_horiz")
            .or_else(|| MaterialIcon::from_name("swap_horizontal_circle"))
            .or_else(|| MaterialIcon::from_name("swap_horiz")),
    };

    let Some(new_icon) = new_icon else {
        return;
    };

    for mut icon in icons.iter_mut() {
        icon.id = new_icon.id;
    }
}

/// Apply shake strength when the Material slider changes.
pub fn handle_shake_slider_changes(
    settings_state: Res<SettingsState>,
    mut events: MessageReader<SliderChangeEvent>,
    mut shake_state: ResMut<ShakeState>,
    mut shake_config: ResMut<ContainerShakeConfig>,
    strength_slider_query: Query<(), With<ShakeSlider>>,
    distance_slider_query: Query<(), With<ShakeDistanceSlider>>,
    speed_slider_query: Query<(), With<ShakeSpeedSlider>>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in events.read() {
        if strength_slider_query.get(event.entity).is_ok() {
            shake_state.strength = event.value.clamp(0.0, 1.0);
            continue;
        }

        if distance_slider_query.get(event.entity).is_ok() {
            shake_config.distance = event.value.clamp(0.2, 1.6);
            continue;
        }

        if speed_slider_query.get(event.entity).is_ok() {
            shake_config.speed = event.value.clamp(0.0, 1.0);
            continue;
        }
    }
}

/// Rotate the camera around the origin (single direction).
pub fn handle_dice_box_rotate_click(
    ui_state: Res<UiState>,
    settings_state: Res<SettingsState>,
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<(), With<DiceBoxControlsPanelRotateButton>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if ui_state.active_tab != AppTab::DiceRoller {
        return;
    }
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        if buttons.get(event.entity).is_err() {
            continue;
        }

        // Rotate by 22.5 degrees each click.
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_8);
        for mut transform in camera_query.iter_mut() {
            let pos = transform.translation;
            transform.translation = rotation * pos;
            *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

/// Apply a quick impulse to dice so they bounce around.
pub fn handle_dice_box_shake_box_click(
    ui_state: Res<UiState>,
    settings_state: Res<SettingsState>,
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<(), With<DiceBoxShakeBoxButton>>,
    shake_state: Res<ShakeState>,
    shake_config: Res<ContainerShakeConfig>,
    mut shake_anim: ResMut<ContainerShakeAnimation>,
    container_query: Query<(Entity, &Transform), With<DiceBox>>,
) {
    if ui_state.active_tab != AppTab::DiceRoller {
        return;
    }
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        if buttons.get(event.entity).is_err() {
            continue;
        }

        let _started = start_container_shake(
            &shake_state,
            &shake_config,
            &mut shake_anim,
            &container_query,
        );
    }
}

/// Animate the dice container shake (moves walls/floor/ceiling left-right rapidly).
pub fn animate_container_shake(
    time: Res<Time>,
    mut shake_anim: ResMut<ContainerShakeAnimation>,
    shake_config: Res<ContainerShakeConfig>,
    mut container_query: Query<(Entity, &mut Transform), With<DiceBox>>,
) {
    if !shake_anim.active {
        return;
    }

    let dt = time.delta_secs();
    shake_anim.elapsed += dt;

    if shake_anim.elapsed >= shake_anim.duration {
        shake_anim.active = false;
    }

    if !shake_anim.active {
        // Restore base positions.
        for (entity, mut transform) in container_query.iter_mut() {
            if let Some(base) = shake_anim.base_positions.get(&entity) {
                transform.translation = *base;
            }
        }
        shake_anim.base_positions.clear();
        return;
    }

    // Envelope: ramp in quickly, sustain, then ramp out (prevents a sudden "snap" stop).
    let progress = (shake_anim.elapsed / shake_anim.duration.max(0.001)).clamp(0.0, 1.0);

    let smoothstep01 = |t: f32| t * t * (3.0 - 2.0 * t);

    // First 35% ramps in.
    let ramp_in = smoothstep01((progress / 0.35).clamp(0.0, 1.0));
    // Last 15% ramps out.
    let ramp_out = smoothstep01(((1.0 - progress) / 0.15).clamp(0.0, 1.0));
    let envelope = (ramp_in * ramp_out).clamp(0.0, 1.0);

    // Frequency ramps up with the ramp-in.
    // Slightly amplify toward the end of ramp-in for a more "kick" feel.
    let amp = shake_anim.amplitude * envelope * (0.85 + 0.15 * ramp_in);

    // Non-looping: progress maps directly to curve t in [0..1].
    let t01 = progress;
    let curve_x = sample_curve(&shake_config.curve_points_x, t01).clamp(-1.0, 1.0);
    let curve_y = sample_curve(&shake_config.curve_points_y, t01).clamp(-1.0, 1.0);
    let curve_z = sample_curve(&shake_config.curve_points_z, t01).clamp(-1.0, 1.0);
    let offset = Vec3::new(curve_x, curve_y, curve_z) * amp;

    for (entity, mut transform) in container_query.iter_mut() {
        if let Some(base) = shake_anim.base_positions.get(&entity) {
            transform.translation = *base + offset;
        }
    }
}

/// Toggle between box and cup wall geometry.
pub fn handle_dice_box_toggle_container_click(
    mut commands: Commands,
    ui_state: Res<UiState>,
    settings_state: Res<SettingsState>,
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<(), With<DiceBoxToggleContainerButton>>,
    mut style: ResMut<DiceContainerStyle>,
    _materials: Res<DiceContainerMaterials>,
    asset_server: Res<AssetServer>,
    _meshes: ResMut<Assets<Mesh>>,
    walls: Query<Entity, With<DiceBoxWall>>,
    floors: Query<Entity, With<DiceBoxFloorCollider>>,
    ceilings: Query<Entity, With<DiceBoxCeiling>>,
    container_root: Query<Entity, With<DiceBox>>,
    mut dice_query: Query<(&mut Transform, &mut Velocity), With<Die>>,
    mut shake_anim: ResMut<ContainerShakeAnimation>,
    mut spawn_points_applied: ResMut<DiceSpawnPointsApplied>,
) {
    if ui_state.active_tab != AppTab::DiceRoller {
        return;
    }
    if settings_state.show_modal {
        return;
    }

    let mut toggled = false;
    for event in click_events.read() {
        if buttons.get(event.entity).is_err() {
            continue;
        }
        toggled = true;
    }
    if !toggled {
        return;
    }

    // Cancel any in-progress shake animation (walls/entities are about to be rebuilt).
    shake_anim.active = false;
    shake_anim.base_positions.clear();

    // Remove existing walls.
    for e in walls.iter() {
        commands.entity(e).despawn();
    }

    // Rebuild floor/ceiling so colliders match the new container style closely.
    for e in floors.iter() {
        commands.entity(e).despawn();
    }
    for e in ceilings.iter() {
        commands.entity(e).despawn();
    }

    // Toggle style.
    *style = match *style {
        DiceContainerStyle::Box => DiceContainerStyle::Cup,
        DiceContainerStyle::Cup => DiceContainerStyle::Box,
    };

    // Allow spawn point placement to re-run for the new style.
    spawn_points_applied.box_applied = false;
    spawn_points_applied.cup_applied = false;

    let mut root_iter = container_root.iter();
    let Some(container_root) = root_iter.next() else {
        return;
    };
    if root_iter.next().is_some() {
        return;
    }

    // Respawn walls/colliders in the new style (as children of the container root).
    let wall_height = BOX_WALL_HEIGHT;
    let wall_thickness = 0.15;

    // Floor collider
    let floor_thickness = 0.30;
    let floor_half_height = floor_thickness / 2.0;
    commands
        .entity(container_root)
        .with_children(|parent| match *style {
            DiceContainerStyle::Box => {
                parent.spawn((
                    Transform::from_xyz(0.0, -floor_half_height, 0.0),
                    Collider::cuboid(BOX_HALF_EXTENT, floor_half_height, BOX_HALF_EXTENT),
                    Restitution::coefficient(0.2),
                    Friction::coefficient(0.8),
                    DiceBoxFloorCollider,
                    DiceContainerProceduralCollider,
                ));
            }
            DiceContainerStyle::Cup => {
                parent.spawn((
                    Transform::from_xyz(0.0, -floor_half_height, 0.0),
                    Collider::cylinder(floor_half_height, CUP_RADIUS),
                    Restitution::coefficient(0.2),
                    Friction::coefficient(0.8),
                    DiceBoxFloorCollider,
                    DiceContainerProceduralCollider,
                ));
            }
        });

    // Ceiling collider
    let ceiling_thickness = 0.10;
    let ceiling_half_height = ceiling_thickness / 2.0;
    commands
        .entity(container_root)
        .with_children(|parent| match *style {
            DiceContainerStyle::Box => {
                let ceiling_size = 2.0 * BOX_HALF_EXTENT + wall_thickness * 2.0;
                parent.spawn((
                    Transform::from_xyz(0.0, wall_height + ceiling_half_height, 0.0),
                    Collider::cuboid(ceiling_size / 2.0, ceiling_half_height, ceiling_size / 2.0),
                    Restitution::coefficient(0.05),
                    Friction::coefficient(0.3),
                    DiceBoxCeiling,
                    DiceContainerProceduralCollider,
                ));
            }
            DiceContainerStyle::Cup => {
                parent.spawn((
                    Transform::from_xyz(0.0, wall_height + ceiling_half_height, 0.0),
                    Collider::cylinder(ceiling_half_height, CUP_RADIUS + wall_thickness),
                    Restitution::coefficient(0.05),
                    Friction::coefficient(0.3),
                    DiceBoxCeiling,
                    DiceContainerProceduralCollider,
                ));
            }
        });

    match *style {
        DiceContainerStyle::Box => {
            let box_size = BOX_HALF_EXTENT;

            commands.entity(container_root).with_children(|parent| {
                // Visual box model (embedded glTF scene)
                let box_scene: Handle<Scene> = asset_server.load(BOX_MODEL_SCENE_PATH);
                let scale = (BOX_HALF_EXTENT / ORIGINAL_BOX_HALF_EXTENT).max(0.0001);
                parent.spawn((
                    SceneRoot(box_scene),
                    Transform::from_xyz(0.0, wall_height / 2.0, 0.0).with_scale(Vec3::splat(scale)),
                    DiceBoxWall,
                    DiceContainerVisualRoot,
                    DiceBoxVisualSceneRoot,
                ));

                for (pos, size) in [
                    (
                        Vec3::new(0.0, wall_height / 2.0, -box_size),
                        Vec3::new(
                            2.0 * box_size + wall_thickness * 2.0,
                            wall_height,
                            wall_thickness,
                        ),
                    ),
                    (
                        Vec3::new(0.0, wall_height / 2.0, box_size),
                        Vec3::new(
                            2.0 * box_size + wall_thickness * 2.0,
                            wall_height,
                            wall_thickness,
                        ),
                    ),
                    (
                        Vec3::new(-box_size, wall_height / 2.0, 0.0),
                        Vec3::new(
                            wall_thickness,
                            wall_height,
                            2.0 * box_size + wall_thickness * 2.0,
                        ),
                    ),
                    (
                        Vec3::new(box_size, wall_height / 2.0, 0.0),
                        Vec3::new(
                            wall_thickness,
                            wall_height,
                            2.0 * box_size + wall_thickness * 2.0,
                        ),
                    ),
                ] {
                    parent.spawn((
                        Transform::from_translation(pos),
                        Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
                        Restitution::coefficient(0.2),
                        Friction::coefficient(0.8),
                        DiceBoxWall,
                        DiceContainerProceduralCollider,
                    ));
                }
            });
        }
        DiceContainerStyle::Cup => {
            let radius: f32 = CUP_RADIUS;

            commands.entity(container_root).with_children(|parent| {
                // Visual cup model (embedded glTF scene)
                let cup_scene: Handle<Scene> = asset_server.load(CUP_MODEL_SCENE_PATH);
                parent.spawn((
                    SceneRoot(cup_scene),
                    Transform::from_xyz(0.0, wall_height / 2.0, 0.0),
                    DiceBoxWall,
                    DiceContainerVisualRoot,
                ));

                // Invisible collider ring
                let segments: usize = 24;
                let segment_length: f32 = (std::f32::consts::TAU * radius) / segments as f32;
                let wall_depth: f32 = wall_thickness;

                for i in 0..segments {
                    let t = i as f32 / segments as f32;
                    let angle = t * std::f32::consts::TAU;
                    let x = angle.cos() * radius;
                    let z = angle.sin() * radius;
                    let pos = Vec3::new(x, wall_height / 2.0, z);
                    let rot = Quat::from_rotation_y(-angle);
                    let size = Vec3::new(segment_length + wall_thickness, wall_height, wall_depth);

                    parent.spawn((
                        Transform::from_translation(pos).with_rotation(rot),
                        Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
                        Restitution::coefficient(0.2),
                        Friction::coefficient(0.8),
                        DiceBoxWall,
                        DiceContainerProceduralCollider,
                    ));
                }
            });
        }
    }

    // Re-drop dice into the middle of the new container style.
    // Put them above the floor so gravity drops them naturally.
    let mut rng = rand::rng();
    let spawn_radius = 0.30;
    for (mut transform, mut velocity) in dice_query.iter_mut() {
        transform.translation = Vec3::new(
            rng.random_range(-spawn_radius..spawn_radius),
            1.25,
            rng.random_range(-spawn_radius..spawn_radius),
        );
        // Give a small downward velocity so they're guaranteed to "re-drop".
        velocity.linvel = Vec3::new(0.0, -0.5, 0.0);
        velocity.angvel = Vec3::new(
            rng.random_range(-1.5..1.5),
            rng.random_range(-1.5..1.5),
            rng.random_range(-1.5..1.5),
        );
    }
}
