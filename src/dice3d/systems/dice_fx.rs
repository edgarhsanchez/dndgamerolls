use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy_hanabi::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::dice3d::dice_fx::DiceFxRollingTracker;
use crate::dice3d::embedded_assets::{
    DICE_FX_ELECTRICITY_SFX_PATH, DICE_FX_EXPLOSION_SFX_PATH, DICE_FX_FIREWORKS_SFX_PATH,
    DICE_FX_FIRE_SFX_PATH, DICE_FX_PLASMABALL_SFX_PATH,
};
use crate::dice3d::hanabi_fx::{fx_handles_for_kind, DiceHanabiFxAssets, DiceHanabiFxInstance};
use crate::dice3d::types::*;
use crate::dice3d::throw_control::{BOX_FLOOR_Y, BOX_HALF_EXTENT, BOX_TOP_Y, CUP_RADIUS};

#[derive(Component, Clone, Copy, Debug)]
pub struct DiceFxOwner(pub Entity);

#[derive(Component, Clone, Copy, Debug)]
pub struct ElectricBoltEmitter {
    pub next_shot_at: f32,
    pub rate_scale: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct FxDespawnAt(pub f32);

#[derive(Component, Clone, Copy, Debug)]
pub struct DiceFxElectricityLoopSfx;

#[derive(Component, Clone, Copy, Debug)]
pub struct DiceFxPlasmaLoopSfx;

#[derive(Component, Clone, Copy, Debug)]
pub struct DiceFxFireLoopSfx;

fn effective_dice_fx_plume_height_multiplier(settings_state: &SettingsState) -> f32 {
    if settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings
    {
        settings_state.editing_dice_fx_plume_height_multiplier
    } else {
        settings_state.settings.dice_fx_plume_height_multiplier
    }
    .clamp(0.25, 3.0)
}

fn effective_dice_fx_plume_radius_multiplier(settings_state: &SettingsState) -> f32 {
    if settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings
    {
        settings_state.editing_dice_fx_plume_radius_multiplier
    } else {
        settings_state.settings.dice_fx_plume_radius_multiplier
    }
    .clamp(0.25, 3.0)
}

fn despawn_all_fx_children(
    commands: &mut Commands,
    children: Option<&Children>,
    fx: &Query<(), With<DiceHanabiFxInstance>>,
) {
    let Some(children) = children else {
        return;
    };

    for child in children.iter() {
        if fx.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

pub fn clear_dice_fx_on_roll_start(
    mut commands: Commands,
    roll_state: Res<RollState>,
    mut tracker: ResMut<DiceFxRollingTracker>,
    dice_query: Query<(Entity, Option<&Children>), With<Die>>,
    fx: Query<(), With<DiceHanabiFxInstance>>,
) {
    let now_rolling = roll_state.rolling;
    let started = now_rolling && !tracker.was_rolling;
    tracker.was_rolling = now_rolling;

    if !started {
        return;
    }

    for (die_entity, children) in dice_query.iter() {
        commands.entity(die_entity).remove::<DiceFxState>();
        commands.entity(die_entity).remove::<DieLastRoll>();
        commands.entity(die_entity).remove::<ElectricBoltEmitter>();

        despawn_all_fx_children(
            &mut commands,
            children,
            &fx,
        );
    }
}

pub fn apply_dice_fx_from_roll_complete(
    mut commands: Commands,
    mut ev: MessageReader<DiceRollCompletedEvent>,
    time: Res<Time>,
    settings_state: Res<SettingsState>,
    hanabi_fx: Res<DiceHanabiFxAssets>,
    asset_server: Res<AssetServer>,
    global_transforms: Query<&GlobalTransform>,
) {
    for event in ev.read() {
        let started_at = time.elapsed_secs();
        // FX should persist until the next roll.
        let duration = 0.0_f32;

        let mut rng = rand::thread_rng();

        for r in &event.results {
            let kind = settings_state.settings.roll_fx_for(r.die_type, r.value);
            let (fire, electric, fireworks, explosion, plasma) = match kind {
                DiceRollFxKind::Fire => (true, false, false, false, false),
                DiceRollFxKind::Electricity => (false, true, false, false, false),
                DiceRollFxKind::Fireworks => (false, false, true, false, false),
                DiceRollFxKind::Explosion => (false, false, false, true, false),
                DiceRollFxKind::Plasma => (false, false, false, false, true),
                DiceRollFxKind::None => (false, false, false, false, false),
            };

            commands.entity(r.entity).insert(DieLastRoll { value: r.value });

            if fire || electric || fireworks || explosion || plasma {
                commands.entity(r.entity).insert(DiceFxState {
                    fire,
                    electric,
                    plasma,
                    fireworks,
                    explosion,
                    started_at,
                    duration,
                });

                // Spawn Hanabi particle effect instances as children of the die.
                let mut scale = Vec3::ONE;
                let h_mul = effective_dice_fx_plume_height_multiplier(&settings_state);
                let r_mul = effective_dice_fx_plume_radius_multiplier(&settings_state);
                scale.x *= r_mul;
                scale.z *= r_mul;
                scale.y *= h_mul;

                for effect_handle in fx_handles_for_kind(&hanabi_fx, kind) {
                    commands.entity(r.entity).with_children(|parent| {
                        parent.spawn((
                            ParticleEffect::new(effect_handle),
                            Transform::from_translation(Vec3::ZERO).with_scale(scale),
                            GlobalTransform::default(),
                            Visibility::Visible,
                            DiceHanabiFxInstance { kind },
                            DiceFxOwner(r.entity),
                        ));
                    });
                }

                // Electricity also emits intermittent jagged bolts toward other dice / hidden targets.
                if electric {
                    // Give each die a persistent cadence bias so bolts don't sync up.
                    let rate_scale = rng.gen_range(0.55..1.85);
                    // Wide initial phase so dice don't start emitting at the same time.
                    let next = started_at + rng.gen_range(0.05..1.25) * rate_scale;
                    commands.entity(r.entity).insert(ElectricBoltEmitter {
                        next_shot_at: next,
                        rate_scale,
                    });
                } else {
                    commands.entity(r.entity).remove::<ElectricBoltEmitter>();
                }

                // Fireworks and explosion SFX: play every time the effect fires (per die).
                if fireworks {
                    let sound: Handle<AudioSource> = asset_server.load(DICE_FX_FIREWORKS_SFX_PATH);
                    commands.spawn((
                        AudioPlayer(sound),
                        PlaybackSettings::DESPAWN
                            .with_spatial(false)
                            .with_volume(Volume::Linear(1.0)),
                    ));
                }
                if explosion {
                    if let Ok(gt) = global_transforms.get(r.entity) {
                        let sound: Handle<AudioSource> = asset_server.load(DICE_FX_EXPLOSION_SFX_PATH);
                        commands.spawn((
                            AudioPlayer(sound),
                            PlaybackSettings::DESPAWN
                                .with_spatial(true)
                                .with_volume(Volume::Linear(0.8)),
                            Transform::from_translation(gt.translation()),
                            GlobalTransform::default(),
                        ));
                    }
                }
            } else {
                commands.entity(r.entity).remove::<DiceFxState>();
                commands.entity(r.entity).remove::<ElectricBoltEmitter>();
            }
        }
    }
}

pub fn update_dice_fx_loop_sfx(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dice_fx: Query<&DiceFxState, With<Die>>,
    electricity_loop: Query<Entity, With<DiceFxElectricityLoopSfx>>,
    plasma_loop: Query<Entity, With<DiceFxPlasmaLoopSfx>>,
    fire_loop: Query<Entity, With<DiceFxFireLoopSfx>>,
) {
    let any_electric = dice_fx.iter().any(|s| s.electric);
    let any_plasma = dice_fx.iter().any(|s| s.plasma);
    let any_fire = dice_fx.iter().any(|s| s.fire);

    if any_electric {
        if electricity_loop.is_empty() {
            let sound: Handle<AudioSource> = asset_server.load(DICE_FX_ELECTRICITY_SFX_PATH);
            commands.spawn((
                DiceFxElectricityLoopSfx,
                AudioPlayer(sound),
                PlaybackSettings::LOOP
                    .with_spatial(false)
                    .with_volume(Volume::Linear(0.7)),
            ));
        }
    } else {
        for e in electricity_loop.iter() {
            commands.entity(e).despawn();
        }
    }

    if any_plasma {
        if plasma_loop.is_empty() {
            let sound: Handle<AudioSource> = asset_server.load(DICE_FX_PLASMABALL_SFX_PATH);
            commands.spawn((
                DiceFxPlasmaLoopSfx,
                AudioPlayer(sound),
                PlaybackSettings::LOOP
                    .with_spatial(false)
                    .with_volume(Volume::Linear(0.7)),
            ));
        }
    } else {
        for e in plasma_loop.iter() {
            commands.entity(e).despawn();
        }
    }

    if any_fire {
        if fire_loop.is_empty() {
            let sound: Handle<AudioSource> = asset_server.load(DICE_FX_FIRE_SFX_PATH);
            commands.spawn((
                DiceFxFireLoopSfx,
                AudioPlayer(sound),
                PlaybackSettings::LOOP
                    .with_spatial(false)
                    .with_volume(Volume::Linear(0.7)),
            ));
        }
    } else {
        for e in fire_loop.iter() {
            commands.entity(e).despawn();
        }
    }
}

fn hidden_electric_targets(style: DiceContainerStyle) -> Vec<Vec3> {
    match style {
        DiceContainerStyle::Box => {
            let r = BOX_HALF_EXTENT * 0.92;
            let y1 = (BOX_FLOOR_Y + 0.25).max(0.18);
            let y2 = (BOX_TOP_Y - 0.20).max(y1 + 0.05);
            vec![
                Vec3::new(-r, y1, -r),
                Vec3::new(-r, y1, r),
                Vec3::new(r, y1, -r),
                Vec3::new(r, y1, r),
                Vec3::new(-r, y2, 0.0),
                Vec3::new(r, y2, 0.0),
                Vec3::new(0.0, y2, -r),
                Vec3::new(0.0, y2, r),
            ]
        }
        DiceContainerStyle::Cup => {
            let r = CUP_RADIUS * 0.88;
            let y1 = (BOX_FLOOR_Y + 0.25).max(0.18);
            let y2 = (BOX_TOP_Y - 0.25).max(y1 + 0.05);
            vec![
                Vec3::new(r, y1, 0.0),
                Vec3::new(-r, y1, 0.0),
                Vec3::new(0.0, y1, r),
                Vec3::new(0.0, y1, -r),
                Vec3::new(r * 0.7, y2, r * 0.7),
                Vec3::new(-r * 0.7, y2, r * 0.7),
                Vec3::new(r * 0.7, y2, -r * 0.7),
                Vec3::new(-r * 0.7, y2, -r * 0.7),
            ]
        }
    }
}

pub fn spawn_electric_bolts(
    mut commands: Commands,
    time: Res<Time>,
    container_style: Res<DiceContainerStyle>,
    hanabi_fx: Res<DiceHanabiFxAssets>,
    dice_box: Query<&GlobalTransform, With<DiceBox>>,
    dice_positions: Query<(Entity, &GlobalTransform), With<Die>>,
    mut dice_emitters: Query<(
        Entity,
        &GlobalTransform,
        &Die,
        &DiceFxState,
        Option<&mut ElectricBoltEmitter>,
    ), With<Die>>,
) {
    let now = time.elapsed_secs();
    let mut rng = rand::thread_rng();

    let box_gt = dice_box.iter().next();

    // Precompute hidden targets in world space if we can.
    let hidden_local = hidden_electric_targets(*container_style);
    let hidden_world: Vec<Vec3> = if let Some(gt) = box_gt {
        hidden_local
            .into_iter()
            .map(|p| gt.transform_point(p))
            .collect()
    } else {
        hidden_local
    };

    // Snapshot other dice positions.
    let mut other_dice: Vec<(Entity, Vec3)> = Vec::new();
    for (e, gt) in dice_positions.iter() {
        other_dice.push((e, gt.translation()));
    }

    for (die_entity, die_gt, die, fx_state, maybe_emitter) in dice_emitters.iter_mut() {
        if !fx_state.electric {
            continue;
        }

        let Some(mut emitter) = maybe_emitter else {
            // If we somehow missed inserting it, create it and let next frame handle.
            commands.entity(die_entity).insert(ElectricBoltEmitter {
                next_shot_at: now + 0.20,
                rate_scale: rng.gen_range(0.75..1.6),
            });
            continue;
        };

        if now < emitter.next_shot_at {
            continue;
        }

        // Randomize next time we shoot.
        // Mostly quick flickers, sometimes longer pauses.
        emitter.next_shot_at = if rng.gen_bool(0.25) {
            now + rng.gen_range(0.22..0.75)
        } else {
            now + rng.gen_range(0.05..0.25)
        };

        let source_world = die_gt.translation();
        let die_rot_world = die_gt.compute_transform().rotation;

        // Origin: a random die vertex (in die-local space), slightly pushed outward.
        let origin_local = random_die_vertex_local(die.die_type, &mut rng);
        let origin_local = origin_local + origin_local.normalize_or_zero() * 0.02;
        let origin_world = source_world + die_rot_world * origin_local;

        // Occasionally "fizzle" (no visible bolt) so the timing feels less periodic.
        // Still reschedule below.
        let should_emit = !rng.gen_bool(0.22);

        // Spawn 2–6 overlapping bolts at a time.
        let bolt_count = rng.gen_range(2..=6);

        // Helper: choose a direction/target per bolt.
        // - 70%: arc to other dice (when any exist)
        // - 20%: pure random direction
        // - 10%: hidden anchors (if available)
        // Falls back sensibly if a category isn't available.
        let candidates: Vec<Vec3> = other_dice
            .iter()
            .filter_map(|(e, p)| (*e != die_entity).then_some(*p))
            .collect();
        let has_other_dice = !candidates.is_empty();
        let has_hidden = !hidden_world.is_empty();

        let choose_target_world = |rng: &mut rand::rngs::ThreadRng| {
            // Pure random direction (bias away from near-vertical so it doesn't read as "shooting up").
            let random_dir_target = |rng: &mut rand::rngs::ThreadRng| {
                let mut d = Vec3::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-0.35..0.65),
                    rng.gen_range(-1.0..1.0),
                )
                .normalize_or_zero();
                if d == Vec3::ZERO {
                    d = Vec3::X;
                }
                let len = rng.gen_range(0.25..1.25);
                source_world + d * len
            };

            let roll = rng.gen_range(0.0..1.0);

            if roll < 0.70 {
                if has_other_dice {
                    *candidates.choose(rng).unwrap()
                } else if has_hidden {
                    *hidden_world.choose(rng).unwrap()
                } else {
                    random_dir_target(rng)
                }
            } else if roll < 0.90 {
                random_dir_target(rng)
            } else {
                if has_hidden {
                    *hidden_world.choose(rng).unwrap()
                } else if has_other_dice {
                    *candidates.choose(rng).unwrap()
                } else {
                    random_dir_target(rng)
                }
            }
        };

        // Randomize next time we shoot.
        // Mix short jittery bursts with occasional long quiet gaps.
        let base_dt = if rng.gen_bool(0.12) {
            rng.gen_range(0.85..2.20)
        } else if rng.gen_bool(0.35) {
            rng.gen_range(0.20..0.85)
        } else {
            rng.gen_range(0.04..0.32)
        };
        // Add extra randomness per event and apply a per-die cadence bias.
        emitter.next_shot_at = now + base_dt * emitter.rate_scale * rng.gen_range(0.85..1.25);

        if !should_emit {
            continue;
        }

        for _ in 0..bolt_count {
            let target_world = choose_target_world(&mut rng);

            let mut world_dir = target_world - origin_world;
            let base_distance = world_dir.length().clamp(0.15, 2.0);
            world_dir = world_dir.normalize_or_zero();
            if world_dir == Vec3::ZERO {
                continue;
            }

            // Randomize bolt length so not every arc reaches its target exactly.
            // Keep it within a sane range so it still reads as "between dice".
            let distance = (base_distance * rng.gen_range(0.65..1.15)).clamp(0.12, 2.0);

            // Convert direction into die-local space so we can spawn as child.
            let local_dir = (die_rot_world.inverse() * world_dir).normalize_or_zero();
            if local_dir == Vec3::ZERO {
                continue;
            }

            // Kinked bolt: chain 2–4 segments, each rotated a bit differently.
            let segment_count = rng.gen_range(2..=4);
            let weights: Vec<f32> = (0..segment_count).map(|_| rng.gen_range(0.6..1.4)).collect();
            let sum_w: f32 = weights.iter().sum();
            let mut seg_lengths: Vec<f32> = weights
                .into_iter()
                .map(|w| (distance * (w / sum_w)).max(0.06))
                .collect();
            // Renormalize in case max() bumped some segments.
            let sum_len: f32 = seg_lengths.iter().sum();
            if sum_len > 0.0 {
                for l in &mut seg_lengths {
                    *l = *l * (distance / sum_len);
                }
            }

            let ttl = rng.gen_range(0.12..0.42);

            let mut seg_dir = local_dir;
            let mut seg_origin = origin_local;

            for (i, seg_len) in seg_lengths.into_iter().enumerate() {
                let base_rot = Quat::from_rotation_arc(Vec3::Y, seg_dir);
                let twist = Quat::from_axis_angle(seg_dir, rng.gen_range(-3.14..3.14));
                let seg_rot = base_rot * twist;

                commands.entity(die_entity).with_children(|parent| {
                    parent.spawn((
                        ParticleEffect::new(hanabi_fx.electricity_bolt.clone()),
                        Transform::from_translation(seg_origin)
                            .with_rotation(seg_rot)
                            .with_scale(Vec3::new(1.0, seg_len, 1.0)),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        DiceHanabiFxInstance {
                            kind: DiceRollFxKind::Electricity,
                        },
                        DiceFxOwner(die_entity),
                        FxDespawnAt(now + ttl),
                    ));
                });

                // Advance origin along the current segment.
                seg_origin += seg_dir * seg_len;

                // Change direction for the next segment (kink), except after the last.
                if i + 1 < segment_count {
                    let mut axis = Vec3::new(
                        rng.gen_range(-1.0..1.0),
                        rng.gen_range(-1.0..1.0),
                        rng.gen_range(-1.0..1.0),
                    )
                    .normalize_or_zero();
                    if axis == Vec3::ZERO {
                        axis = seg_dir.any_orthonormal_vector();
                    }
                    axis = (axis - seg_dir * axis.dot(seg_dir)).normalize_or_zero();
                    if axis == Vec3::ZERO {
                        axis = seg_dir.any_orthonormal_vector();
                    }

                    let kink = rng.gen_range(-0.95..0.95); // ~55 degrees
                    seg_dir = (Quat::from_axis_angle(axis, kink) * seg_dir).normalize_or_zero();
                    if seg_dir == Vec3::ZERO {
                        seg_dir = local_dir;
                    }
                }
            }
        }
    }
}

fn random_die_vertex_local(die_type: DiceType, rng: &mut rand::rngs::ThreadRng) -> Vec3 {
    let vertices = die_vertices_local(die_type);
    *vertices
        .choose(rng)
        .unwrap_or(&Vec3::ZERO)
}

fn die_vertices_local(die_type: DiceType) -> Vec<Vec3> {
    match die_type {
        DiceType::D4 => {
            let size = 0.5;
            let a = size;
            let h = a * (2.0_f32 / 3.0_f32).sqrt();
            let base_y = -h / 4.0;
            let apex_y = 3.0 * h / 4.0;
            let base_r = a / (3.0_f32).sqrt();
            let v0 = Vec3::new(0.0, apex_y, 0.0);
            let v1 = Vec3::new(0.0, base_y, base_r);
            let v2 = Vec3::new(base_r * 0.866, base_y, -base_r * 0.5);
            let v3 = Vec3::new(-base_r * 0.866, base_y, -base_r * 0.5);
            vec![v0, v1, v2, v3]
        }
        DiceType::D6 => {
            let size = 0.6;
            let h = size / 2.0;
            vec![
                Vec3::new(-h, -h, -h),
                Vec3::new(-h, -h, h),
                Vec3::new(-h, h, -h),
                Vec3::new(-h, h, h),
                Vec3::new(h, -h, -h),
                Vec3::new(h, -h, h),
                Vec3::new(h, h, -h),
                Vec3::new(h, h, h),
            ]
        }
        DiceType::D8 => {
            let size = 0.5;
            vec![
                Vec3::new(0.0, size, 0.0),
                Vec3::new(0.0, -size, 0.0),
                Vec3::new(size, 0.0, 0.0),
                Vec3::new(-size, 0.0, 0.0),
                Vec3::new(0.0, 0.0, size),
                Vec3::new(0.0, 0.0, -size),
            ]
        }
        DiceType::D10 => {
            let size = 0.5;
            let angle = std::f32::consts::PI / 5.0;
            let mut vertices = Vec::new();
            let top = Vec3::new(0.0, size * 0.9, 0.0);
            let bottom = Vec3::new(0.0, -size * 0.9, 0.0);
            vertices.push(top);
            vertices.push(bottom);
            for i in 0..5 {
                let a = i as f32 * angle * 2.0;
                vertices.push(Vec3::new(a.cos() * size * 0.7, size * 0.3, a.sin() * size * 0.7));
            }
            for i in 0..5 {
                let a = (i as f32 + 0.5) * angle * 2.0;
                vertices.push(Vec3::new(
                    a.cos() * size * 0.7,
                    -size * 0.3,
                    a.sin() * size * 0.7,
                ));
            }
            vertices
        }
        DiceType::D12 => {
            let size = 0.5;
            let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
            let s = size * 0.35;
            vec![
                Vec3::new(-1.0, -1.0, -1.0) * s,
                Vec3::new(-1.0, -1.0, 1.0) * s,
                Vec3::new(-1.0, 1.0, -1.0) * s,
                Vec3::new(-1.0, 1.0, 1.0) * s,
                Vec3::new(1.0, -1.0, -1.0) * s,
                Vec3::new(1.0, -1.0, 1.0) * s,
                Vec3::new(1.0, 1.0, -1.0) * s,
                Vec3::new(1.0, 1.0, 1.0) * s,
                Vec3::new(0.0, -1.0 / phi, -phi) * s,
                Vec3::new(0.0, -1.0 / phi, phi) * s,
                Vec3::new(0.0, 1.0 / phi, -phi) * s,
                Vec3::new(0.0, 1.0 / phi, phi) * s,
                Vec3::new(-1.0 / phi, -phi, 0.0) * s,
                Vec3::new(-1.0 / phi, phi, 0.0) * s,
                Vec3::new(1.0 / phi, -phi, 0.0) * s,
                Vec3::new(1.0 / phi, phi, 0.0) * s,
                Vec3::new(-phi, 0.0, -1.0 / phi) * s,
                Vec3::new(-phi, 0.0, 1.0 / phi) * s,
                Vec3::new(phi, 0.0, -1.0 / phi) * s,
                Vec3::new(phi, 0.0, 1.0 / phi) * s,
            ]
        }
        DiceType::D20 => {
            let size = 0.5;
            let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
            let s = size * 0.35;
            vec![
                Vec3::new(0.0, 1.0, phi) * s,
                Vec3::new(0.0, -1.0, phi) * s,
                Vec3::new(0.0, 1.0, -phi) * s,
                Vec3::new(0.0, -1.0, -phi) * s,
                Vec3::new(1.0, phi, 0.0) * s,
                Vec3::new(-1.0, phi, 0.0) * s,
                Vec3::new(1.0, -phi, 0.0) * s,
                Vec3::new(-1.0, -phi, 0.0) * s,
                Vec3::new(phi, 0.0, 1.0) * s,
                Vec3::new(-phi, 0.0, 1.0) * s,
                Vec3::new(phi, 0.0, -1.0) * s,
                Vec3::new(-phi, 0.0, -1.0) * s,
            ]
        }
    }
}

pub fn despawn_temporary_fx(
    mut commands: Commands,
    time: Res<Time>,
    q: Query<(Entity, &FxDespawnAt)>,
) {
    let now = time.elapsed_secs();
    for (e, at) in q.iter() {
        if now >= at.0 {
            commands.entity(e).despawn();
        }
    }
}
