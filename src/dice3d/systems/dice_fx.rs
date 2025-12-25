use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy_hanabi::prelude::*;
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
pub struct DiceElectricityWander {
    pub velocity: Vec3,
}

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
    dice_box: Query<(Entity, Option<&Children>), With<DiceBox>>,
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

        despawn_all_fx_children(
            &mut commands,
            children,
            &fx,
        );
    }

    for (box_entity, children) in dice_box.iter() {
        despawn_all_fx_children(&mut commands, children, &fx);
        // Wander component is on the FX child, so clearing children is enough.
        commands.entity(box_entity).remove::<DiceFxState>();
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
    dice_box: Query<Entity, With<DiceBox>>,
) {
    for event in ev.read() {
        let started_at = time.elapsed_secs();
        // FX should persist until the next roll.
        let duration = 0.0_f32;

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

            // Electricity is a container-wide effect (spawned separately below).
            if fire || fireworks || explosion || plasma {
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
            } else {
                commands.entity(r.entity).remove::<DiceFxState>();
            }
        }

        // Container-wide Electricity: a wandering emitter inside the box/cup.
        if any_electric {
            let Some(box_entity) = dice_box.iter().next() else {
                continue;
            };

            let mut rng = rand::thread_rng();
            let speed = rng.gen_range(2.0..6.0);
            let dir = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-0.2..0.8), rng.gen_range(-1.0..1.0))
                .normalize_or_zero();

            // Start near the center, slightly above the floor.
            let start_pos = Vec3::new(0.0, (BOX_TOP_Y * 0.45).max(0.35), 0.0);

            commands.entity(box_entity).with_children(|parent| {
                parent.spawn((
                    ParticleEffect::new(hanabi_fx.electricity.clone()),
                    Transform::from_translation(start_pos),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    DiceHanabiFxInstance {
                        kind: DiceRollFxKind::Electricity,
                    },
                    DiceElectricityWander {
                        velocity: dir * speed,
                    },
                    // owner is the container
                    DiceFxOwner(box_entity),
                ));
            });

            // Make sure DiceFxState reflects that electricity is active.
            commands.entity(box_entity).insert(DiceFxState {
                fire: false,
                electric: true,
                fireworks: false,
                explosion: false,
                started_at,
                duration,
            });
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

pub fn update_electricity_wander(
    time: Res<Time>,
    container_style: Res<DiceContainerStyle>,
    mut q: Query<(&mut Transform, &mut DiceElectricityWander)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut wander) in q.iter_mut() {
        let mut pos = transform.translation;
        pos += wander.velocity * dt;

        // Keep it inside the container, "bouncing" off walls.
        match *container_style {
            DiceContainerStyle::Box => {
                let min_x = -BOX_HALF_EXTENT * 0.92;
                let max_x = BOX_HALF_EXTENT * 0.92;
                let min_z = -BOX_HALF_EXTENT * 0.92;
                let max_z = BOX_HALF_EXTENT * 0.92;
                let min_y = BOX_FLOOR_Y + 0.15;
                let max_y = BOX_TOP_Y - 0.15;

                if pos.x < min_x {
                    pos.x = min_x;
                    wander.velocity.x = wander.velocity.x.abs();
                } else if pos.x > max_x {
                    pos.x = max_x;
                    wander.velocity.x = -wander.velocity.x.abs();
                }

                if pos.z < min_z {
                    pos.z = min_z;
                    wander.velocity.z = wander.velocity.z.abs();
                } else if pos.z > max_z {
                    pos.z = max_z;
                    wander.velocity.z = -wander.velocity.z.abs();
                }

                if pos.y < min_y {
                    pos.y = min_y;
                    wander.velocity.y = wander.velocity.y.abs();
                } else if pos.y > max_y {
                    pos.y = max_y;
                    wander.velocity.y = -wander.velocity.y.abs();
                }
            }
            DiceContainerStyle::Cup => {
                let min_y = BOX_FLOOR_Y + 0.15;
                let max_y = BOX_TOP_Y - 0.10;

                let r = CUP_RADIUS * 0.88;
                let xz = Vec2::new(pos.x, pos.z);
                let len = xz.length();
                if len > r {
                    let n = xz / len;
                    // Reflect velocity around the normal in XZ plane.
                    let v_xz = Vec2::new(wander.velocity.x, wander.velocity.z);
                    let reflected = v_xz - 2.0 * v_xz.dot(n) * n;
                    wander.velocity.x = reflected.x;
                    wander.velocity.z = reflected.y;
                    pos.x = n.x * r;
                    pos.z = n.y * r;
                }

                if pos.y < min_y {
                    pos.y = min_y;
                    wander.velocity.y = wander.velocity.y.abs();
                } else if pos.y > max_y {
                    pos.y = max_y;
                    wander.velocity.y = -wander.velocity.y.abs();
                }
            }
        }

        transform.translation = pos;
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
