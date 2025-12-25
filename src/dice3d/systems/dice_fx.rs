use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy_hanabi::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::dice3d::dice_fx::DiceFxRollingTracker;
use crate::dice3d::embedded_assets::{
    DICE_FX_ELECTRICITY_SFX_PATH, DICE_FX_EXPLOSION_SFX_PATH, DICE_FX_FIREWORKS_SFX_PATH,
};
use crate::dice3d::hanabi_fx::{fx_handles_for_kind, DiceHanabiFxAssets, DiceHanabiFxInstance};
use crate::dice3d::types::*;
use crate::dice3d::throw_control::{BOX_FLOOR_Y, BOX_HALF_EXTENT, BOX_TOP_Y, CUP_RADIUS};

#[derive(Component, Clone, Copy, Debug)]
pub struct DiceFxOwner(pub Entity);

#[derive(Component, Clone, Copy, Debug)]
pub struct ElectricBoltEmitter {
    pub next_bolt_at: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct FxDespawnAt(pub f32);

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

        let sfx_pos = event
            .results
            .iter()
            .find_map(|r| global_transforms.get(r.entity).ok().map(|gt| gt.translation()));

        let mut any_electric = false;
        let mut any_fireworks = false;
        let mut any_explosion = false;

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

            any_electric |= electric;
            any_fireworks |= fireworks;
            any_explosion |= explosion;

            commands.entity(r.entity).insert(DieLastRoll { value: r.value });

            if fire || electric || fireworks || explosion || plasma {
                commands.entity(r.entity).insert(DiceFxState {
                    fire,
                    electric,
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
                    let next = started_at + rng.gen_range(0.05..0.18);
                    commands.entity(r.entity).insert(ElectricBoltEmitter { next_bolt_at: next });
                } else {
                    commands.entity(r.entity).remove::<ElectricBoltEmitter>();
                }
            } else {
                commands.entity(r.entity).remove::<DiceFxState>();
                commands.entity(r.entity).remove::<ElectricBoltEmitter>();
            }
        }

        if let Some(pos) = sfx_pos {
            if any_electric {
                let sound: Handle<AudioSource> = asset_server.load(DICE_FX_ELECTRICITY_SFX_PATH);
                commands.spawn((
                    AudioPlayer(sound),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Linear(0.8)),
                    Transform::from_translation(pos),
                    GlobalTransform::default(),
                ));
            }
            if any_fireworks {
                let sound: Handle<AudioSource> = asset_server.load(DICE_FX_FIREWORKS_SFX_PATH);
                commands.spawn((
                    AudioPlayer(sound),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Linear(0.8)),
                    Transform::from_translation(pos),
                    GlobalTransform::default(),
                ));
            }
            if any_explosion {
                let sound: Handle<AudioSource> = asset_server.load(DICE_FX_EXPLOSION_SFX_PATH);
                commands.spawn((
                    AudioPlayer(sound),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Linear(0.8)),
                    Transform::from_translation(pos),
                    GlobalTransform::default(),
                ));
            }
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
    mut dice_gt: Query<(
        Entity,
        &GlobalTransform,
        Option<&DiceFxState>,
        Option<Mut<ElectricBoltEmitter>>,
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
    for (e, gt, _, _) in dice_gt.iter() {
        other_dice.push((e, gt.translation()));
    }

    for (die_entity, die_gt, maybe_fx_state, maybe_emitter) in dice_gt.iter_mut() {
        let Some(fx_state) = maybe_fx_state else {
            continue;
        };
        if !fx_state.electric {
            continue;
        }

        let Some(mut emitter) = maybe_emitter else {
            // If we somehow missed inserting it, create it and let next frame handle.
            commands
                .entity(die_entity)
                .insert(ElectricBoltEmitter { next_bolt_at: now + 0.10 });
            continue;
        };

        if now < emitter.next_bolt_at {
            continue;
        }

        // Schedule next bolt. Keep it flickery.
        emitter.next_bolt_at = now + rng.gen_range(0.09..0.22);

        let source_world = die_gt.translation();
        let die_rot_world = die_gt.compute_transform().rotation;

        // Choose a target:
        // - Prefer other dice ~70% of the time (when available)
        // - Otherwise, occasionally hit "hidden" anchors to feel unpredictable
        let target_world = {
            let mut candidates: Vec<Vec3> = Vec::new();
            for (e, p) in other_dice.iter() {
                if *e != die_entity {
                    candidates.push(*p);
                }
            }

            let prefer_other_dice = rng.gen_bool(0.70);

            if prefer_other_dice {
                if !candidates.is_empty() {
                    *candidates.choose(&mut rng).unwrap()
                } else if !hidden_world.is_empty() {
                    *hidden_world.choose(&mut rng).unwrap()
                } else {
                    source_world + Vec3::Y * 0.8
                }
            } else if !hidden_world.is_empty() {
                *hidden_world.choose(&mut rng).unwrap()
            } else if !candidates.is_empty() {
                *candidates.choose(&mut rng).unwrap()
            } else {
                source_world + Vec3::Y * 0.8
            }
        };

        let mut world_dir = target_world - source_world;
        let distance = world_dir.length().clamp(0.15, 2.0);
        world_dir = world_dir.normalize_or_zero();
        if world_dir == Vec3::ZERO {
            continue;
        }

        // Convert direction into die-local space so we can spawn as child.
        let local_dir = (die_rot_world.inverse() * world_dir).normalize_or_zero();
        let base_rot = Quat::from_rotation_arc(Vec3::Y, local_dir);

        // Spawn 2-6 bolts for branching/jaggedness.
        let bolt_count = rng.gen_range(2..=6);
        for _ in 0..bolt_count {
            let yaw = rng.gen_range(-0.55..0.55);
            let roll = rng.gen_range(-0.65..0.65);
            let twist = Quat::from_rotation_y(yaw) * Quat::from_rotation_z(roll);

            let offset = Vec3::new(
                rng.gen_range(-0.05..0.05),
                rng.gen_range(0.02..0.12),
                rng.gen_range(-0.05..0.05),
            );

            let bolt_rot = base_rot * twist;

            commands.entity(die_entity).with_children(|parent| {
                parent.spawn((
                    ParticleEffect::new(hanabi_fx.electricity_bolt.clone()),
                    Transform::from_translation(offset)
                        .with_rotation(bolt_rot)
                        .with_scale(Vec3::new(1.0, distance, 1.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    DiceHanabiFxInstance {
                        kind: DiceRollFxKind::Electricity,
                    },
                    DiceFxOwner(die_entity),
                    FxDespawnAt(now + 0.35),
                ));
            });
        }
    }
}

pub fn despawn_temporary_fx(mut commands: Commands, time: Res<Time>, q: Query<(Entity, &FxDespawnAt)>) {
    let now = time.elapsed_secs();
    for (e, at) in q.iter() {
        if now >= at.0 {
            commands.entity(e).despawn();
        }
    }
}

pub fn expire_dice_roll_fx(
    time: Res<Time>,
    mut commands: Commands,
    dice: Query<(Entity, Option<&Children>, &DiceFxState), With<Die>>,
    fx: Query<(), With<DiceHanabiFxInstance>>,
) {
    let t = time.elapsed_secs();
    for (entity, children, state) in dice.iter() {
        if state.duration > 0.0 && (t - state.started_at) >= state.duration {
            commands.entity(entity).remove::<DiceFxState>();
            despawn_all_fx_children(&mut commands, children, &fx);
        }
    }
}
