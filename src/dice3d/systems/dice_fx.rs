use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy_hanabi::prelude::*;

use crate::dice3d::dice_fx::{DiceFxEffectAssets, DiceFxRollingTracker};
use crate::dice3d::embedded_assets::{
    DICE_FX_ELECTRICITY_SFX_PATH, DICE_FX_EXPLOSION_SFX_PATH, DICE_FX_FIREWORKS_SFX_PATH,
};
use crate::dice3d::types::*;

#[derive(Resource, Clone)]
pub struct DiceFxSfxAssets {
    pub electricity: Handle<AudioSource>,
    pub explosion: Handle<AudioSource>,
    pub fireworks: Handle<AudioSource>,
}

pub fn init_dice_fx_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DiceFxSfxAssets {
        electricity: asset_server.load(DICE_FX_ELECTRICITY_SFX_PATH),
        explosion: asset_server.load(DICE_FX_EXPLOSION_SFX_PATH),
        fireworks: asset_server.load(DICE_FX_FIREWORKS_SFX_PATH),
    });
}

#[derive(Component)]
pub struct DelayedFxBurst {
    timer: Timer,
}

#[derive(Component)]
pub struct DiceFxElectricityLoopSfx;

#[derive(Component)]
pub struct DelayedSfx {
    timer: Timer,
    sound: Handle<AudioSource>,
    volume: f32,
    speed: f32,
    spatial: bool,
    looped: bool,
}

pub fn tick_delayed_sfx(
    mut commands: Commands,
    time: Res<Time>,
    audio_sources: Res<Assets<AudioSource>>,
    mut q: Query<(Entity, &mut DelayedSfx, Option<&Transform>), Without<AudioPlayer>>,
) {
    for (entity, mut delayed, transform) in q.iter_mut() {
        delayed.timer.tick(time.delta());
        if !delayed.timer.is_finished() {
            continue;
        }

        // If the asset isn't loaded yet, keep the component and retry next frame.
        if audio_sources.get(&delayed.sound).is_none() {
            continue;
        }

        // Ensure the entity has a transform if we want spatial audio.
        if delayed.spatial && transform.is_none() {
            commands
                .entity(entity)
                .insert((Transform::default(), GlobalTransform::default()));
        }

        let base = if delayed.looped {
            PlaybackSettings::LOOP
        } else {
            PlaybackSettings::DESPAWN
        };

        let settings = base
            .with_spatial(delayed.spatial)
            .with_volume(Volume::Linear(delayed.volume))
            .with_speed(delayed.speed);

        commands
            .entity(entity)
            .insert((AudioPlayer(delayed.sound.clone()), settings))
            .remove::<DelayedSfx>();
    }
}

pub fn tick_delayed_fx_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut EffectSpawner, &mut DelayedFxBurst)>,
) {
    for (entity, mut spawner, mut delayed) in q.iter_mut() {
        delayed.timer.tick(time.delta());
        if delayed.timer.is_finished() {
            // For `once()` spawners created with `emit_on_start = false`, reset
            // triggers a burst on the next spawner tick (PostUpdate).
            spawner.reset();
            commands.entity(entity).remove::<DelayedFxBurst>();
        }
    }
}

fn effective_roll_effects_setting(settings_state: &SettingsState) -> &DiceFxRollEffectsSetting {
    if settings_state.show_modal && settings_state.modal_kind == ActiveModalKind::DiceRollerSettings {
        &settings_state.editing_dice_fx_roll_effects
    } else {
        &settings_state.settings.dice_fx_roll_effects
    }
}

fn effective_dice_fx_surface_opacity(settings_state: &SettingsState) -> f32 {
    if settings_state.show_modal && settings_state.modal_kind == ActiveModalKind::DiceRollerSettings {
        settings_state.editing_dice_fx_surface_opacity
    } else {
        settings_state.settings.dice_fx_surface_opacity
    }
    .clamp(0.0, 1.0)
}

fn effective_dice_fx_plume_height_multiplier(settings_state: &SettingsState) -> f32 {
    if settings_state.show_modal && settings_state.modal_kind == ActiveModalKind::DiceRollerSettings {
        settings_state.editing_dice_fx_plume_height_multiplier
    } else {
        settings_state.settings.dice_fx_plume_height_multiplier
    }
    .clamp(0.25, 3.0)
}

fn effective_dice_fx_plume_radius_multiplier(settings_state: &SettingsState) -> f32 {
    if settings_state.show_modal && settings_state.modal_kind == ActiveModalKind::DiceRollerSettings {
        settings_state.editing_dice_fx_plume_radius_multiplier
    } else {
        settings_state.settings.dice_fx_plume_radius_multiplier
    }
    .clamp(0.25, 3.0)
}

fn despawn_all_fx_children(
    commands: &mut Commands,
    children: Option<&Children>,
    fire: &Query<(), With<DiceFxFireEffect>>,
    lightning: &Query<(), With<DiceFxLightningEffect>>,
    firework: &Query<(), With<DiceFxFireworkEffect>>,
    explosion: &Query<(), With<DiceFxExplosionEffect>>,
    electricity_loop: &Query<(), With<DiceFxElectricityLoopSfx>>,
    delayed_sfx: &Query<(), With<DelayedSfx>>,
) {
    let Some(children) = children else {
        return;
    };

    for child in children.iter() {
        if fire.get(child).is_ok()
            || lightning.get(child).is_ok()
            || firework.get(child).is_ok()
            || explosion.get(child).is_ok()
            || electricity_loop.get(child).is_ok()
            || delayed_sfx.get(child).is_ok()
        {
            commands.entity(child).despawn();
        }
    }
}

pub fn clear_dice_fx_on_roll_start(
    mut commands: Commands,
    roll_state: Res<RollState>,
    mut tracker: ResMut<DiceFxRollingTracker>,
    dice_query: Query<(Entity, Option<&Children>), With<Die>>,
    fire: Query<(), With<DiceFxFireEffect>>,
    lightning: Query<(), With<DiceFxLightningEffect>>,
    firework: Query<(), With<DiceFxFireworkEffect>>,
    explosion: Query<(), With<DiceFxExplosionEffect>>,
    electricity_loop: Query<(), With<DiceFxElectricityLoopSfx>>,
    delayed_sfx: Query<(), With<DelayedSfx>>,
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
            &fire,
            &lightning,
            &firework,
            &explosion,
            &electricity_loop,
            &delayed_sfx,
        );
    }
}

pub fn apply_dice_fx_from_roll_complete(
    mut commands: Commands,
    mut ev: MessageReader<DiceRollCompletedEvent>,
    settings_state: Res<SettingsState>,
    sfx: Res<DiceFxSfxAssets>,
    _die_gt: Query<&GlobalTransform, With<Die>>,
) {
    let mapping = effective_roll_effects_setting(&settings_state);

    for event in ev.read() {
        // If any die triggers Explosion, play the explosion sound once.
        let mut any_explosion = false;

        for r in &event.results {
            let effect = mapping.effect_for(r.die_type, r.value);
            if effect == DiceFxEffectKind::Explosion {
                any_explosion = true;
            }
            if effect != DiceFxEffectKind::None {
                bevy::log::info!(
                    "Dice FX roll complete: die={:?} value={} -> {:?}",
                    r.die_type,
                    r.value,
                    effect
                );
            }
            commands.entity(r.entity).insert(DieLastRoll { value: r.value });
            commands.entity(r.entity).insert(DiceFxState { effect });
        }

        if any_explosion {
            // Play once globally per roll result set.
            // Use non-spatial audio for reliability / audibility.
            commands.spawn(DelayedSfx {
                timer: Timer::from_seconds(0.0, TimerMode::Once),
                sound: sfx.explosion.clone(),
                volume: 1.35,
                speed: 1.0,
                spatial: false,
                looped: false,
            });
        }
    }
}

pub fn sync_dice_fx_visuals(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings_state: Res<SettingsState>,
    fx_effects: Res<DiceFxEffectAssets>,
    sfx: Res<DiceFxSfxAssets>,
    dice_query: Query<(Entity, &DiceFxState, &Transform, Option<&Children>), With<Die>>,
    fire: Query<(), With<DiceFxFireEffect>>,
    lightning: Query<(), With<DiceFxLightningEffect>>,
    firework: Query<(), With<DiceFxFireworkEffect>>,
    explosion: Query<(), With<DiceFxExplosionEffect>>,
    electricity_loop: Query<(), With<DiceFxElectricityLoopSfx>>,
    delayed_sfx: Query<(), With<DelayedSfx>>,
) {
    let _surface_opacity = effective_dice_fx_surface_opacity(&settings_state);
    let plume_h_mul = effective_dice_fx_plume_height_multiplier(&settings_state);
    let plume_r_mul = effective_dice_fx_plume_radius_multiplier(&settings_state);

    if !fx_effects.is_ready() {
        return;
    }

    for (die_entity, state, die_transform, children) in dice_query.iter() {
        let desired = state.effect;

        let mut has_fire = false;
        let mut has_lightning = false;
        let mut has_firework = false;
        let mut has_explosion = false;

        if let Some(children) = children {
            for c in children.iter() {
                has_fire |= fire.get(c).is_ok();
                has_lightning |= lightning.get(c).is_ok();
                has_firework |= firework.get(c).is_ok();
                has_explosion |= explosion.get(c).is_ok();
            }
        }

        let has_desired = match desired {
            DiceFxEffectKind::None => false,
            DiceFxEffectKind::Fire => has_fire,
            DiceFxEffectKind::Lightning => has_lightning,
            DiceFxEffectKind::Firework => has_firework,
            DiceFxEffectKind::Explosion => has_explosion,
        };

        if desired == DiceFxEffectKind::None {
            despawn_all_fx_children(
                &mut commands,
                children,
                &fire,
                &lightning,
                &firework,
                &explosion,
                &electricity_loop,
                &delayed_sfx,
            );
            continue;
        }

        let has_any = has_fire || has_lightning || has_firework || has_explosion;
        if has_any && !has_desired {
            despawn_all_fx_children(
                &mut commands,
                children,
                &fire,
                &lightning,
                &firework,
                &explosion,
                &electricity_loop,
                &delayed_sfx,
            );
        }

        if has_desired {
            continue;
        }

        let die_scale = die_transform
            .scale
            .x
            .max(die_transform.scale.y)
            .max(die_transform.scale.z);

        match desired {
            DiceFxEffectKind::Fire => {
                let height = 1.2 * die_scale * plume_h_mul;
                let radius = 0.40 * die_scale * plume_r_mul;
                let transform = Transform::from_translation(Vec3::ZERO)
                    .with_scale(Vec3::new(radius, height, radius));

                let noise = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_NOISE_PATH);
                let ramp = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_RAMP_PATH);
                let mask = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_MASK_PATH);

                commands.entity(die_entity).with_children(|parent| {
                    parent.spawn((
                        ParticleEffect::new(fx_effects.fire.clone()),
                        EffectSpawner::new(&SpawnerSettings::rate(380.0.into())),
                        EffectMaterial {
                            images: vec![noise, ramp, mask],
                        },
                        EffectProperties::default(),
                        transform,
                        Visibility::Visible,
                        DiceFxFireEffect,
                    ));
                });
            }
            DiceFxEffectKind::Lightning => {
                let r = 1.05 * die_scale;
                let visible = Visibility::Visible;

                let noise = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_NOISE_PATH);
                let ramp = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_RAMP_PATH);
                let mask = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_MASK_PATH);

                // Spawn multiple rotated instances to get "Faraday cage"-like
                // arcs in several planes instead of a single ring.
                let spawner = EffectSpawner::new(&SpawnerSettings::rate(240.0.into()));
                let mat = EffectMaterial {
                    images: vec![noise, ramp, mask],
                };

                commands.entity(die_entity).with_children(|parent| {
                    for rot in [
                        Quat::IDENTITY,
                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                    ] {
                        let transform = Transform::from_translation(Vec3::ZERO)
                            .with_rotation(rot)
                            .with_scale(Vec3::splat(r));

                        parent.spawn((
                            ParticleEffect::new(fx_effects.lightning.clone()),
                            spawner.clone(),
                            mat.clone(),
                            EffectProperties::default(),
                            transform,
                            visible,
                            DiceFxLightningEffect,
                        ));
                    }

                    // Loop electricity SFX while lightning is active on this die.
                    // Use non-spatial audio to ensure audibility.
                    parent.spawn((
                        DelayedSfx {
                            timer: Timer::from_seconds(0.0, TimerMode::Once),
                            sound: sfx.electricity.clone(),
                            volume: 1.0,
                            speed: 1.0,
                            spatial: false,
                            looped: true,
                        },
                        DiceFxElectricityLoopSfx,
                    ));
                });
            }
            DiceFxEffectKind::Firework => {
                let r = 1.85 * die_scale;
                // Three staggered bursts with varied size and sound.

                // Firework needs a strong opacity mask; the "fire" set works better here.
                let noise = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_NOISE_PATH);
                let ramp = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_RAMP_PATH);
                let mask = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_MASK_PATH);

                let seed = (die_entity.to_bits() as u32) ^ 0xA53C_19E7;
                let r01 = |s: u32| {
                    let mut x = s.wrapping_add(0x9E37_79B9);
                    x ^= x >> 16;
                    x = x.wrapping_mul(0x7FEB_352D);
                    x ^= x >> 15;
                    x = x.wrapping_mul(0x846C_A68B);
                    x ^= x >> 16;
                    (x as f32) / (u32::MAX as f32)
                };

                commands.entity(die_entity).with_children(|parent| {
                    let mat = EffectMaterial {
                        images: vec![noise, ramp, mask],
                    };

                    let stages = [
                        (0.00_f32, 0.85_f32, 220.0_f32),
                        (0.16_f32, 1.00_f32, 260.0_f32),
                        (0.34_f32, 1.25_f32, 320.0_f32),
                    ];

                    // One sound per firework effect, with slight per-die stagger and variation.
                    let sfx_delay = 0.04 * r01(seed ^ 0x3333_3333);
                    let sfx_vol = (1.05 + 0.35 * r01(seed ^ 0x4444_4444)).clamp(0.25, 1.6);
                    let sfx_speed = (0.92 + 0.18 * r01(seed ^ 0x5555_5555)).clamp(0.75, 1.25);
                    parent.spawn((
                        DelayedSfx {
                            timer: Timer::from_seconds(sfx_delay, TimerMode::Once),
                            sound: sfx.fireworks.clone(),
                            volume: sfx_vol,
                            speed: sfx_speed,
                            spatial: false,
                            looped: false,
                        },
                        Transform::default(),
                        GlobalTransform::default(),
                    ));

                    for (i, (delay_s, scale_mul, count)) in stages.into_iter().enumerate() {
                        let _stage_seed = seed ^ ((i as u32) * 0x1F12_3BB5);
                        let transform = Transform::from_translation(Vec3::ZERO)
                            .with_scale(Vec3::splat(r * scale_mul));

                        let settings = if delay_s <= 0.0 {
                            SpawnerSettings::once(count.into())
                        } else {
                            SpawnerSettings::once(count.into()).with_emit_on_start(false)
                        };

                        let mut entity_cmd = parent.spawn((
                            ParticleEffect::new(fx_effects.firework.clone()),
                            EffectSpawner::new(&settings),
                            mat.clone(),
                            EffectProperties::default(),
                            transform,
                            Visibility::Visible,
                            DiceFxFireworkEffect,
                        ));

                        if delay_s > 0.0 {
                            entity_cmd.insert(DelayedFxBurst {
                                timer: Timer::from_seconds(delay_s, TimerMode::Once),
                            });
                        }
                    }
                });
            }
            DiceFxEffectKind::Explosion => {
                let r = 2.80 * die_scale;

                let noise = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_NOISE_PATH);
                let ramp = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_RAMP_PATH);
                let mask = asset_server.load(crate::dice3d::embedded_assets::DICE_FX_FIRE_MASK_PATH);

                commands.entity(die_entity).with_children(|parent| {
                    // Three staged bursts (different sizes) fired one after the other.
                    // Stage 0 emits immediately; stage 1/2 use delayed reset().
                    let spawner0 = EffectSpawner::new(&SpawnerSettings::once(220.0.into()));
                    let spawner1 = EffectSpawner::new(
                        &SpawnerSettings::once(260.0.into()).with_emit_on_start(false),
                    );
                    let spawner2 = EffectSpawner::new(
                        &SpawnerSettings::once(320.0.into()).with_emit_on_start(false),
                    );
                    let mat = EffectMaterial {
                        images: vec![noise, ramp, mask],
                    };

                    // Small -> medium -> big, with slight offsets, to read as
                    // multiple mushroom clouds building up.
                    parent.spawn((
                        ParticleEffect::new(fx_effects.explosion.clone()),
                        spawner0,
                        mat.clone(),
                        EffectProperties::default(),
                        Transform::from_translation(Vec3::new(0.00, 0.04, 0.00))
                            .with_scale(Vec3::splat(r * 0.90)),
                        Visibility::Visible,
                        DiceFxExplosionEffect,
                    ));

                    parent.spawn((
                        ParticleEffect::new(fx_effects.explosion.clone()),
                        spawner1,
                        mat.clone(),
                        EffectProperties::default(),
                        Transform::from_translation(Vec3::new(0.12, 0.10, -0.10))
                            .with_scale(Vec3::splat(r * 1.05)),
                        Visibility::Visible,
                        DiceFxExplosionEffect,
                        DelayedFxBurst {
                            timer: Timer::from_seconds(0.18, TimerMode::Once),
                        },
                    ));

                    parent.spawn((
                        ParticleEffect::new(fx_effects.explosion.clone()),
                        spawner2,
                        mat,
                        EffectProperties::default(),
                        Transform::from_translation(Vec3::new(-0.14, 0.16, 0.12))
                            .with_scale(Vec3::splat(r * 1.22)),
                        Visibility::Visible,
                        DiceFxExplosionEffect,
                        DelayedFxBurst {
                            timer: Timer::from_seconds(0.42, TimerMode::Once),
                        },
                    ));
                });
            }
            DiceFxEffectKind::None => {}
        }
    }
}
/*
NOTE: Legacy/custom Dice FX code (curves, custom images, texture processing)
was intentionally removed during the pivot to hard-coded built-in effects.
    let mut count = 0u64;
    for px in data.chunks_exact(4) {
        let r = srgb_to_linear(px[0] as f32 / 255.0);
        let g = srgb_to_linear(px[1] as f32 / 255.0);
        let b = srgb_to_linear(px[2] as f32 / 255.0);
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        sum += luma as f64;
        count += 1;
    }
    if count == 0 {
        return None;
    }

    let avg = (sum / count as f64) as f32;
    let avg = avg.clamp(0.0, 1.0);
    cache.by_image.insert(id, avg);
    Some(avg)
}

fn curve_value_to_multiplier(v: f32) -> f32 {
    // Spec:
    // - v > 0 increases dramatically
    // - v == 0 => no particles
    // - v < 0 dramatically decreases
    if v.abs() < 1e-6 {
        return 0.0;
    }
    if v > 0.0 {
        (1.0 + v * 6.0).clamp(0.0, 50.0)
    } else {
        (10.0f32).powf(v).clamp(0.0, 1.0)
    }
}


Legacy/custom Dice FX code disabled.

fn sort_points(points: &mut [FxCurvePointSetting]) {
    points.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
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

fn sample_curve_unclamped(points: &[FxCurvePointSetting], t: f32) -> f32 {
    if points.is_empty() {
        return t.clamp(0.0, 1.0);
    }
    if points.len() == 1 {
        return points[0].value;
    }

    let t = t.clamp(0.0, 1.0);

    let mut points_sorted: std::borrow::Cow<'_, [FxCurvePointSetting]> =
        std::borrow::Cow::Borrowed(points);
    if !points.windows(2).all(|w| w[0].t <= w[1].t) {
        let mut tmp = points.to_vec();
        sort_points(&mut tmp);
        points_sorted = std::borrow::Cow::Owned(tmp);
    }
    let points = points_sorted.as_ref();

    if t <= points[0].t {
        return points[0].value;
    }
    if t >= points[points.len() - 1].t {
        return points[points.len() - 1].value;
    }

    for w in points.windows(2) {
        let a = &w[0];
        let b = &w[1];
        if t >= a.t && t <= b.t {
            let dt = (b.t - a.t).max(0.0001);
            let initial_u = ((t - a.t) / dt).clamp(0.0, 1.0);

            let mut p1 = a
                .out_handle
                .map(|h| Vec2::new(h[0], h[1]))
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 1.0 / 3.0), a.value));
            let mut p2 = b
                .in_handle
                .map(|h| Vec2::new(h[0], h[1]))
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 2.0 / 3.0), b.value));

            p1.x = p1.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p2.x = p2.x.clamp(a.t.min(b.t), a.t.max(b.t));

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

    t
}

fn bevy_image_from_rgba8(width: u32, height: u32, rgba: Vec<u8>) -> Image {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                | bevy::render::render_resource::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);
    image.data = Some(rgba);
    image
}

fn load_image_from_disk(path: &Path) -> Result<Image, String> {
    let dyn_img = image::ImageReader::open(path)
        .map_err(|e| format!("Failed to open image {path:?}: {e}"))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess image format {path:?}: {e}"))?
        .decode()
        .map_err(|e| format!("Failed to decode image {path:?}: {e}"))?;

    let rgba = dyn_img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(bevy_image_from_rgba8(w, h, rgba.into_raw()))
}

fn ensure_custom_fx_textures_loaded(
    custom_cfg: Option<&CustomDiceFxSetting>,
    cache: &mut CustomDiceFxTextures,
    images: &mut Assets<Image>,
) {
    let Some(cfg) = custom_cfg else {
        cache.source = None;
        cache.noise = None;
        cache.ramp = None;
        cache.mask = None;
        cache.source_path = None;
        cache.noise_path = None;
        cache.ramp_path = None;
        cache.mask_path = None;
        return;
    };

    if !cfg.enabled {
        cache.source = None;
        cache.noise = None;
        cache.ramp = None;
        cache.mask = None;
        cache.source_path = None;
        cache.noise_path = None;
        cache.ramp_path = None;
        cache.mask_path = None;
        return;
    }

    let (Some(source_path), Some(noise_path), Some(ramp_path), Some(mask_path)) = (
        cfg.source_image_path.as_ref(),
        cfg.noise_image_path.as_ref(),
        cfg.ramp_image_path.as_ref(),
        cfg.mask_image_path.as_ref(),
    ) else {
        return;
    };

    let changed = cache.source_path.as_deref() != Some(source_path)
        || cache.noise_path.as_deref() != Some(noise_path)
        || cache.ramp_path.as_deref() != Some(ramp_path)
        || cache.mask_path.as_deref() != Some(mask_path);

    if changed {
        cache.source = None;
        cache.noise = None;
        cache.ramp = None;
        cache.mask = None;
        cache.source_path = Some(source_path.clone());
        cache.noise_path = Some(noise_path.clone());
        cache.ramp_path = Some(ramp_path.clone());
        cache.mask_path = Some(mask_path.clone());
    }

    if cache.source.is_none() {
        if let Ok(img) = load_image_from_disk(Path::new(source_path)) {
            cache.source = Some(images.add(img));
        }
    }

    if cache.noise.is_none() {
        if let Ok(img) = load_image_from_disk(Path::new(noise_path)) {
            cache.noise = Some(images.add(img));
        }
    }
    if cache.ramp.is_none() {
        if let Ok(img) = load_image_from_disk(Path::new(ramp_path)) {
            cache.ramp = Some(images.add(img));
        }
    }
    if cache.mask.is_none() {
        if let Ok(img) = load_image_from_disk(Path::new(mask_path)) {
            cache.mask = Some(images.add(img));
        }
    }
}


fn effective_dice_fx_surface_opacity(settings_state: &SettingsState) -> f32 {
    if settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings
    {
        settings_state.editing_dice_fx_surface_opacity
    } else {
        settings_state.settings.dice_fx_surface_opacity
    }
    .clamp(0.0, 1.0)
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

fn trigger_custom_fx(cfg: &CustomDiceFxSetting, results: &[DieRollOutcome]) -> bool {
    if !cfg.enabled {
        return false;
    }

    match cfg.trigger_kind {
        CustomDiceFxTriggerKind::TotalAtLeast => {
            let total: u32 = results.iter().map(|r| r.value).sum();
            total >= cfg.trigger_value
        }
        CustomDiceFxTriggerKind::AllMax => {
            !results.is_empty() && results.iter().all(|r| r.value == r.die_type.max_value())
        }
        CustomDiceFxTriggerKind::AnyDieEquals => results.iter().any(|r| r.value == cfg.trigger_value),
    }
}

fn despawn_all_fx_children(
    commands: &mut Commands,
    children: Option<&Children>,
    fire: &Query<(), With<DiceFxFireEffect>>,
    atomic: &Query<(), With<DiceFxAtomicEffect>>,
    electric: &Query<(), With<DiceFxElectricEffect>>,
    custom: &Query<(), With<DiceFxCustomEffect>>,
) {
    let Some(children) = children else {
        return;
    };

    for child in children.iter() {
        if fire.get(child).is_ok()
            || atomic.get(child).is_ok()
            || electric.get(child).is_ok()
            || custom.get(child).is_ok()
        {
            commands.entity(child).despawn();
        }
    }
}

pub fn clear_dice_fx_on_roll_start(
    mut commands: Commands,
    roll_state: Res<RollState>,
    mut tracker: ResMut<DiceFxRollingTracker>,
    dice_query: Query<(Entity, Option<&Children>), With<Die>>,
    fire: Query<(), With<DiceFxFireEffect>>,
    atomic: Query<(), With<DiceFxAtomicEffect>>,
    electric: Query<(), With<DiceFxElectricEffect>>,
    custom: Query<(), With<DiceFxCustomEffect>>,
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
            &fire,
            &atomic,
            &electric,
            &custom,
        );
    }
}

// (legacy code removed)

pub fn expire_custom_dice_fx(
    time: Res<Time>,
    mut dice: Query<&mut DiceFxState, With<Die>>,
) {
    let t = time.elapsed_secs();
    for mut state in dice.iter_mut() {
        if !state.custom {
            continue;
        }
        let dur = state.custom_duration.max(0.0);
        if dur <= 0.0 {
            state.custom = false;
            continue;
        }
        if t - state.custom_started_at >= dur {
            state.custom = false;
        }
    }
}

pub fn apply_dice_fx_density_from_images_and_curves(
    time: Res<Time>,
    settings_state: Res<SettingsState>,
    images: Res<Assets<Image>>,
    mut luma_cache: ResMut<DiceFxImageLumaCache>,
    dice_state: Query<&DiceFxState, With<Die>>,
    mut fx_query: Query<(
        &DiceFxOwner,
        &DiceFxBaseScale,
        &DiceFxBaseRate,
        &DiceFxSpawnedAt,
        &EffectMaterial,
        &mut EffectSpawner,
        &mut EffectProperties,
        &mut Transform,
        Option<&DiceFxFireEffect>,
        Option<&DiceFxAtomicEffect>,
        Option<&DiceFxElectricEffect>,
        Option<&DiceFxCustomEffect>,
    )>,
) {
    let cfg = effective_custom_dice_fx_cfg(&settings_state);

    let now = time.elapsed_secs();

    for (
        owner,
        base_scale,
        base_rate,
        spawned_at,
        material,
        mut spawner,
        mut props,
        mut transform,
        is_fire,
        is_atomic,
        is_electric,
        is_custom,
    ) in fx_query.iter_mut() {
        // Only touch our dice FX particle entities (ones with at least one marker).
        if is_fire.is_none() && is_atomic.is_none() && is_electric.is_none() && is_custom.is_none() {
            continue;
        }

        // We expect images = [noise, ramp, mask] (+ optional source for custom)
        let Some(noise_handle) = material.images.get(0) else {
            continue;
        };
        let Some(ramp_handle) = material.images.get(1) else {
            continue;
        };
        let Some(mask_handle) = material.images.get(2) else {
            continue;
        };

        let Some(l_ramp) = avg_image_luma01(ramp_handle, &images, &mut luma_cache) else {
            continue;
        };
        let Some(l_mask) = avg_image_luma01(mask_handle, &images, &mut luma_cache) else {
            continue;
        };
        let Some(l_noise) = avg_image_luma01(noise_handle, &images, &mut luma_cache) else {
            continue;
        };

        // Curves apply as a dramatic multiplier.
        // - If custom FX is active on this die, we sample over its duration.
        // - Otherwise we sample over a fixed time window from spawn.
        let (
            t01,
            opacity_curve_points,
            plume_h_curve_points,
            plume_r_curve_points,
            ramp_curve_points,
            mask_curve_points,
            noise_curve_points,
        ) = if let Some(cfg) = cfg {
            // Prefer per-die custom duration when custom is active.
            let t01 = dice_state
                .get(owner.0)
                .ok()
                .and_then(|s| {
                    if !s.custom {
                        return None;
                    }
                    let dur = s.custom_duration.max(0.0001);
                    Some(((now - s.custom_started_at) / dur).clamp(0.0, 1.0))
                })
                .unwrap_or_else(|| ((now - spawned_at.0) / cfg.duration_seconds.max(0.0001)).clamp(0.0, 1.0));
            (
                t01,
                Some(&cfg.curve_points_opacity),
                Some(&cfg.curve_points_plume_height),
                Some(&cfg.curve_points_plume_radius),
                Some(&cfg.curve_points_ramp),
                Some(&cfg.curve_points_mask),
                Some(&cfg.curve_points_noise),
            )
        } else {
            (0.0, None, None, None, None, None, None)
        };

        let curve_mul = if let Some(points) = opacity_curve_points {
            curve_value_to_multiplier(sample_curve_unclamped(points, t01))
        } else {
            1.0
        };

        let ramp_time_mul = ramp_curve_points
            .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, t01)))
            .unwrap_or(1.0);
        let mask_time_mul = mask_curve_points
            .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, t01)))
            .unwrap_or(1.0);
        let noise_time_mul = noise_curve_points
            .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, t01)))
            .unwrap_or(1.0);

        // Image brightness drives quantity and size.
        // - ramp controls primary intensity/color structure
        // - mask controls how much of the effect is visible
        // - noise controls how "busy" it is
        let image_qty_mul = (l_ramp.powf(1.8) * l_mask.powf(1.4) * (0.35 + 0.65 * l_noise)).clamp(0.0, 5.0);
        let image_size_mul = (0.20 + 2.8 * l_ramp).clamp(0.0, 6.0);

        // Curves can further multiply based on each texture's brightness.
        // These are dramatic and follow the same sign rules:
        // 0 => off, negative => crush, positive => boost.
        let ramp_curve_mul = ramp_curve_points
            .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, l_ramp)))
            .unwrap_or(1.0);
        let mask_curve_mul = mask_curve_points
            .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, l_mask)))
            .unwrap_or(1.0);
        let noise_curve_mul = noise_curve_points
            .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, l_noise)))
            .unwrap_or(1.0);

        let ramp_mul = ramp_curve_mul * ramp_time_mul;
        let mask_mul = mask_curve_mul * mask_time_mul;
        let noise_mul = noise_curve_mul * noise_time_mul;

        let qty_mul = image_qty_mul * curve_mul * ramp_mul * mask_mul * noise_mul;
        let size_mul = image_size_mul * curve_mul * ramp_mul;

        let final_rate = (base_rate.0 * qty_mul).clamp(0.0, 50_000.0);
        spawner.settings = SpawnerSettings::rate(final_rate.into());
        spawner.active = final_rate > 0.01;

        // Size scaling:
        // - Fire/Atomic are anisotropic plumes; allow separate height/radius curves.
        // - Electric/Custom keep uniform scaling.
        if is_fire.is_some() || is_atomic.is_some() {
            let plume_h_mul = plume_h_curve_points
                .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, t01)))
                .unwrap_or(1.0);
            let plume_r_mul = plume_r_curve_points
                .map(|p| curve_value_to_multiplier(sample_curve_unclamped(p, t01)))
                .unwrap_or(1.0);
            let s = size_mul.clamp(0.0, 10.0);
            let r = (s * plume_r_mul).clamp(0.0, 10.0);
            let h = (s * plume_h_mul).clamp(0.0, 10.0);
            transform.scale = Vec3::new(base_scale.0.x * r, base_scale.0.y * h, base_scale.0.z * r);
        } else {
            let s = size_mul.clamp(0.0, 10.0);
            transform.scale = base_scale.0 * s;
        }

        // Fully animatable Hanabi properties (declared in the effect modules).
        // Clamp to keep values in a sane range even with dramatic curves.
        let ramp_b = ramp_curve_mul.clamp(0.0, 5.0);
        let ramp_t = ramp_time_mul.clamp(0.0, 5.0);
        let mask_b = mask_curve_mul.clamp(0.0, 5.0);
        let mask_t = mask_time_mul.clamp(0.0, 5.0);
        let noise_b = noise_curve_mul.clamp(0.0, 5.0);
        let noise_t = noise_time_mul.clamp(0.0, 5.0);

        let spawn_radius = ((0.35 + 1.65 * l_ramp) * ramp_b * ramp_t).clamp(0.0, 10.0);
        let speed_mul = ((0.20 + 1.80 * l_noise) * noise_b * noise_t).clamp(0.0, 10.0);
        let lifetime_mul = ((0.20 + 1.80 * l_mask) * mask_b * mask_t).clamp(0.0, 10.0);
        let drag_mul = ((0.30 + 1.70 * (1.0 - l_noise)) * mask_b * mask_t).clamp(0.0, 10.0);
        let accel_mul = ((0.20 + 1.80 * l_ramp) * ramp_b * ramp_t).clamp(0.0, 10.0);

        props = EffectProperties::set_if_changed(props, "spawn_radius", spawn_radius.into());
        props = EffectProperties::set_if_changed(props, "speed_mul", speed_mul.into());
        props = EffectProperties::set_if_changed(props, "lifetime_mul", lifetime_mul.into());
        props = EffectProperties::set_if_changed(props, "drag_mul", drag_mul.into());
        EffectProperties::set_if_changed(props, "accel_mul", accel_mul.into());
    }
}

*/
