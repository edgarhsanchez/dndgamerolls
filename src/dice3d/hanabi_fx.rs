use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::dice3d::types::DiceRollFxKind;

#[derive(Resource, Clone)]
pub struct DiceHanabiFxAssets {
    pub fire_core: Handle<EffectAsset>,
    pub fire_sparks_small: Handle<EffectAsset>,
    pub fire_sparks_large: Handle<EffectAsset>,
    pub fire_smoke: Handle<EffectAsset>,
    pub electricity: Handle<EffectAsset>,
    pub electricity_bolt: Handle<EffectAsset>,
    pub fireworks_rocket: Handle<EffectAsset>,
    pub fireworks: Handle<EffectAsset>,
    pub explosion: Handle<EffectAsset>,
    pub plasma_core: Handle<EffectAsset>,
    pub plasma_filaments: Handle<EffectAsset>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct DiceHanabiFxInstance {
    pub kind: DiceRollFxKind,
}

pub fn init_dice_hanabi_fx_assets(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let fire_core = effects.add(make_fire_core_fx());
    let fire_sparks_small = effects.add(make_fire_sparks_small_fx());
    let fire_sparks_large = effects.add(make_fire_sparks_large_fx());
    let fire_smoke = effects.add(make_fire_smoke_fx());
    let electricity = effects.add(make_electric_wander_fx());
    let electricity_bolt = effects.add(make_electric_bolt_fx());
    let fireworks_rocket = effects.add(make_fireworks_rocket_fx());
    let fireworks = effects.add(make_fireworks_fx());
    let explosion = effects.add(make_explosion_fx());
    let plasma_core = effects.add(make_plasma_core_fx());
    let plasma_filaments = effects.add(make_plasma_filaments_fx());

    commands.insert_resource(DiceHanabiFxAssets {
        fire_core,
        fire_sparks_small,
        fire_sparks_large,
        fire_smoke,
        electricity,
        electricity_bolt,
        fireworks_rocket,
        fireworks,
        explosion,
        plasma_core,
        plasma_filaments,
    });
}

pub fn fx_handles_for_kind(assets: &DiceHanabiFxAssets, kind: DiceRollFxKind) -> Vec<Handle<EffectAsset>> {
    match kind {
        DiceRollFxKind::None => vec![],
        DiceRollFxKind::Fire => vec![
            assets.fire_core.clone(),
            assets.fire_sparks_small.clone(),
            assets.fire_sparks_large.clone(),
            assets.fire_smoke.clone(),
        ],
        DiceRollFxKind::Electricity => vec![assets.electricity.clone()],
        DiceRollFxKind::Fireworks => vec![assets.fireworks_rocket.clone(), assets.fireworks.clone()],
        DiceRollFxKind::Explosion => vec![assets.explosion.clone()],
        DiceRollFxKind::Plasma => vec![assets.plasma_core.clone(), assets.plasma_filaments.clone()],
    }
}

fn make_fire_core_fx() -> EffectAsset {
    let mut color = bevy_hanabi::Gradient::new();
    // HDR-bright colors to drive bloom.
    color.add_key(0.0, Vec4::new(18.0, 5.0, 0.35, 1.0));
    color.add_key(0.25, Vec4::new(12.0, 4.0, 0.7, 0.90));
    color.add_key(1.0, Vec4::new(0.3, 0.1, 0.02, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    // Smaller core particles; size variation comes from the separate sparks layer.
    size.add_key(0.0, Vec3::splat(0.045));
    size.add_key(0.4, Vec3::splat(0.075));
    size.add_key(1.0, Vec3::splat(0.01));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::new(0.0, 0.12, 0.0)).expr();
    let axis = w.lit(Vec3::Y).expr();
    let radius = w.lit(0.10).expr();

    // Lifetime in [0.30:0.80]
    let lifetime = w
        .lit(0.30)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.50)));

    // Speed in [0.9:2.6]
    let speed = w
        .lit(0.90)
        .add(w.rand(ScalarType::Float).mul(w.lit(1.70)));

    // Use a sphere velocity from a point slightly below the die so the
    // resulting direction is biased upward.
    let vel_center = w.lit(Vec3::new(0.0, -0.08, 0.0)).expr();

    // Stronger upward acceleration to feel like hot combustion.
    let accel = w.lit(Vec3::new(0.0, 1.9, 0.0)).expr();
    let drag = w.lit(2.6).expr();

    let module = w.finish();

    EffectAsset::new(4096, SpawnerSettings::rate(300.0.into()), module)
        .with_name("dice_fire_core")
        .init(SetPositionCircleModifier {
            center,
            axis,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .init(SetVelocitySphereModifier {
            center: vel_center,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(AccelModifier::new(accel))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

fn make_fire_sparks_small_fx() -> EffectAsset {
    // Smaller, brighter, more varied particles which provide the "hot center" feel.
    // This layer intentionally spans a wider size range than the core.
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(26.0, 9.0, 1.2, 1.0));
    color.add_key(0.2, Vec4::new(18.0, 6.0, 0.8, 0.95));
    color.add_key(0.7, Vec4::new(10.0, 2.0, 0.2, 0.45));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.012));
    size.add_key(0.25, Vec3::splat(0.022));
    size.add_key(1.0, Vec3::splat(0.005));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::new(0.0, 0.11, 0.0)).expr();
    let axis = w.lit(Vec3::Y).expr();
    let radius = w.lit(0.08).expr();

    // Lifetime in [0.18:0.55]
    let lifetime = w
        .lit(0.18)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.37)));

    // Speed in [1.6:5.8]
    let speed = w
        .lit(1.6)
        .add(w.rand(ScalarType::Float).mul(w.lit(4.2)));

    // Bias velocity upward.
    let vel_center = w.lit(Vec3::new(0.0, -0.12, 0.0)).expr();
    let accel = w.lit(Vec3::new(0.0, 2.8, 0.0)).expr();
    let drag = w.lit(3.6).expr();

    let module = w.finish();

    EffectAsset::new(4096, SpawnerSettings::rate(650.0.into()), module)
        .with_name("dice_fire_sparks_small")
        .init(SetPositionCircleModifier {
            center,
            axis,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .init(SetVelocitySphereModifier {
            center: vel_center,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(AccelModifier::new(accel))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn make_fire_sparks_large_fx() -> EffectAsset {
    // Larger/softer sparks mixed in at a lower rate to avoid uniform particle sizes.
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(18.0, 6.0, 0.6, 0.95));
    color.add_key(0.25, Vec4::new(12.0, 4.0, 0.35, 0.80));
    color.add_key(0.8, Vec4::new(5.0, 1.0, 0.08, 0.25));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.024));
    size.add_key(0.25, Vec3::splat(0.040));
    size.add_key(1.0, Vec3::splat(0.008));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::new(0.0, 0.11, 0.0)).expr();
    let axis = w.lit(Vec3::Y).expr();
    let radius = w.lit(0.11).expr();

    // Lifetime in [0.20:0.60]
    let lifetime = w
        .lit(0.20)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.40)));

    // Speed in [1.2:3.6]
    let speed = w
        .lit(1.2)
        .add(w.rand(ScalarType::Float).mul(w.lit(2.4)));

    let vel_center = w.lit(Vec3::new(0.0, -0.10, 0.0)).expr();
    let accel = w.lit(Vec3::new(0.0, 2.0, 0.0)).expr();
    let drag = w.lit(2.8).expr();

    let module = w.finish();

    EffectAsset::new(4096, SpawnerSettings::rate(220.0.into()), module)
        .with_name("dice_fire_sparks_large")
        .init(SetPositionCircleModifier {
            center,
            axis,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .init(SetVelocitySphereModifier {
            center: vel_center,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(AccelModifier::new(accel))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn make_fire_smoke_fx() -> EffectAsset {
    let mut color = bevy_hanabi::Gradient::new();
    // Smoke: subtle HDR (still blooms slightly) and fades out.
    color.add_key(0.0, Vec4::new(0.5, 0.5, 0.5, 0.30));
    color.add_key(0.6, Vec4::new(0.25, 0.25, 0.25, 0.18));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.10));
    size.add_key(1.0, Vec3::splat(0.22));

    let w = ExprWriter::new();
    // Spawn mostly at the edges of the fire footprint.
    let center = w.lit(Vec3::new(0.0, 0.10, 0.0)).expr();
    let axis = w.lit(Vec3::Y).expr();
    let radius = w.lit(0.18).expr();

    // Lifetime in [1.2:2.2]
    let lifetime = w
        .lit(1.2)
        .add(w.rand(ScalarType::Float).mul(w.lit(1.0)));

    // Slow rise, little drift.
    let speed = w
        .lit(0.25)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.45)));
    let vel_center = w.lit(Vec3::new(0.0, -0.08, 0.0)).expr();
    let accel = w.lit(Vec3::new(0.0, 0.55, 0.0)).expr();
    let drag = w.lit(1.0).expr();

    let module = w.finish();

    EffectAsset::new(2048, SpawnerSettings::rate(55.0.into()), module)
        .with_name("dice_fire_smoke")
        .init(SetPositionCircleModifier {
            center,
            axis,
            radius,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: vel_center,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(AccelModifier::new(accel))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

fn make_fireworks_fx() -> EffectAsset {
    // Burst that appears after a brief delay (alpha=0 at t=0), so it reads as
    // "rocket up" then "explode" when combined with the rocket trail effect.
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 0.0));
    color.add_key(0.25, Vec4::new(18.0, 16.0, 10.0, 1.0));
    color.add_key(0.6, Vec4::new(8.0, 6.0, 2.0, 0.75));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.020));
    size.add_key(0.35, Vec3::splat(0.030));
    size.add_key(1.0, Vec3::splat(0.008));

    let w = ExprWriter::new();

    // Spawn the burst at a random height above the die.
    let x = w.rand(ScalarType::Float).mul(w.lit(0.20)).sub(w.lit(0.10));
    let z = w.rand(ScalarType::Float).mul(w.lit(0.20)).sub(w.lit(0.10));
    let y = w.lit(0.65).add(w.rand(ScalarType::Float).mul(w.lit(0.75)));
    let pos = w
        .lit(Vec3::X)
        .mul(x)
        .add(w.lit(Vec3::Y).mul(y))
        .add(w.lit(Vec3::Z).mul(z));

    // Speed in [3.0:8.5]
    let speed = w
        .lit(3.0)
        .add(w.rand(ScalarType::Float).mul(w.lit(5.5)));

    // Lifetime in [0.75:1.35]
    let lifetime = w
        .lit(0.75)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.60)));

    let gravity = w.lit(Vec3::new(0.0, -8.5, 0.0)).expr();
    let drag = w.lit(1.0).expr();
    let zero = w.lit(Vec3::ZERO).expr();

    let module = w.finish();

    EffectAsset::new(12000, SpawnerSettings::once(1100.0.into()), module)
        .with_name("dice_fireworks")
        .init(SetAttributeModifier::new(Attribute::POSITION, pos.expr()))
        .init(SetVelocitySphereModifier {
            center: zero,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(AccelModifier::new(gravity))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn make_fireworks_rocket_fx() -> EffectAsset {
    // "Rocket" streaks that shoot upward from near the die. The burst itself is
    // handled by make_fireworks_fx() and appears slightly later via a color delay.
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(20.0, 20.0, 20.0, 1.0));
    color.add_key(0.6, Vec4::new(10.0, 8.0, 4.0, 0.55));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    // Long, thin streaks (oriented along velocity).
    size.add_key(0.0, Vec3::new(0.010, 0.18, 0.010));
    size.add_key(1.0, Vec3::new(0.006, 0.06, 0.006));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::new(0.0, 0.08, 0.0)).expr();
    let radius = w.lit(0.05).expr();

    // Upward-biased velocity with slight spread.
    let vx = w.rand(ScalarType::Float).mul(w.lit(0.8)).sub(w.lit(0.4));
    let vz = w.rand(ScalarType::Float).mul(w.lit(0.8)).sub(w.lit(0.4));
    let vy = w.lit(5.5).add(w.rand(ScalarType::Float).mul(w.lit(4.0)));
    let vel = w
        .lit(Vec3::X)
        .mul(vx)
        .add(w.lit(Vec3::Y).mul(vy))
        .add(w.lit(Vec3::Z).mul(vz));

    // Lifetime in [0.35:0.70]
    let lifetime = w
        .lit(0.35)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.35)));

    let gravity = w.lit(Vec3::new(0.0, -5.0, 0.0)).expr();
    let drag = w.lit(0.8).expr();
    let module = w.finish();

    EffectAsset::new(2048, SpawnerSettings::once(90.0.into()), module)
        .with_name("dice_fireworks_rocket")
        .init(SetPositionSphereModifier {
            center,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .init(SetAttributeModifier::new(Attribute::VELOCITY, vel.expr()))
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(AccelModifier::new(gravity))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn make_explosion_fx() -> EffectAsset {
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(16.0, 8.0, 2.0, 1.0));
    color.add_key(0.2, Vec4::new(12.0, 3.0, 0.4, 0.9));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.06));
    size.add_key(0.35, Vec3::splat(0.09));
    size.add_key(1.0, Vec3::splat(0.01));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::new(0.0, 0.12, 0.0)).expr();
    let radius = w.lit(0.02).expr();

    // Speed in [4.0:10.0]
    let speed = w
        .lit(4.0)
        .add(w.rand(ScalarType::Float).mul(w.lit(6.0)));

    // Lifetime in [0.25:0.75]
    let lifetime = w
        .lit(0.25)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.50)));

    let gravity = w.lit(Vec3::new(0.0, -8.0, 0.0)).expr();
    let drag = w.lit(2.5).expr();
    let zero = w.lit(Vec3::ZERO).expr();

    let module = w.finish();

    EffectAsset::new(8000, SpawnerSettings::once(1200.0.into()), module)
        .with_name("dice_explosion")
        .init(SetPositionSphereModifier {
            center,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .init(SetVelocitySphereModifier {
            center: zero,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(AccelModifier::new(gravity))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn make_electric_wander_fx() -> EffectAsset {
    // Bright, short-lived arc streaks emitted around the die.
    // This is meant to read like a corona / Faraday-cage discharge, not a uniform
    // "rotating ring" around the vertical axis.
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(4.0, 14.0, 26.0, 1.0));
    color.add_key(0.3, Vec4::new(2.0, 10.0, 22.0, 0.95));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    // Shorter streaks so it reads as arcs/bolts.
    size.add_key(0.0, Vec3::new(0.010, 0.08, 0.010));
    size.add_key(1.0, Vec3::new(0.006, 0.03, 0.006));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::ZERO).expr();
    let radius = w.lit(0.22).expr();

    // Speed in [3.5:11.0]
    let speed = w
        .lit(3.5)
        .add(w.rand(ScalarType::Float).mul(w.lit(7.5)));

    // Lifetime in [0.06:0.16]
    let lifetime = w
        .lit(0.06)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.10)));

    let drag = w.lit(8.5).expr();
    let origin = w.lit(Vec3::ZERO).expr();

    // Random axis per particle -> breaks the "spinning around Y" look and produces
    // chaotic arcing paths.
    let ax = w.rand(ScalarType::Float).mul(w.lit(2.0)).sub(w.lit(1.0));
    let ay = w.rand(ScalarType::Float).mul(w.lit(2.0)).sub(w.lit(1.0));
    let az = w.rand(ScalarType::Float).mul(w.lit(2.0)).sub(w.lit(1.0));
    let axis = w
        .lit(Vec3::X)
        .mul(ax)
        .add(w.lit(Vec3::Y).mul(ay))
        .add(w.lit(Vec3::Z).mul(az))
        .expr();
    let module = w.finish();

    EffectAsset::new(14000, SpawnerSettings::rate(520.0.into()), module)
        .with_name("dice_electric_wander")
        .init(SetPositionSphereModifier {
            center,
            radius,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocityTangentModifier {
            origin,
            axis,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn make_electric_bolt_fx() -> EffectAsset {
    // A short-lived, very bright bolt "volume" along +Y (scale the effect entity's Y to set length).
    // Important: do NOT give particles a world-up velocity. Hanabi interprets velocity in simulation
    // space (not rotated by the effect entity), which makes bolts appear to shoot straight up.
    // Instead, spawn static particles in a thin column and let the entity transform aim the bolt.
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(8.0, 28.0, 48.0, 1.0));
    color.add_key(0.25, Vec4::new(6.0, 20.0, 40.0, 0.95));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    // Small points which read as a jagged bolt when densely distributed.
    size.add_key(0.0, Vec3::splat(0.018));
    size.add_key(1.0, Vec3::splat(0.010));

    let w = ExprWriter::new();

    // Position in a thin "column" with y in [0..1]. The effect entity's transform scales this
    // into world-space bolt length.
    let x = w.rand(ScalarType::Float).mul(w.lit(0.08)).sub(w.lit(0.04));
    let z = w.rand(ScalarType::Float).mul(w.lit(0.08)).sub(w.lit(0.04));
    let y = w.rand(ScalarType::Float);
    let pos = w
        .lit(Vec3::X)
        .mul(x)
        .add(w.lit(Vec3::Y).mul(y))
        .add(w.lit(Vec3::Z).mul(z));

    // Very short flash.
    let lifetime = w.lit(0.05).add(w.rand(ScalarType::Float).mul(w.lit(0.07)));
    let module = w.finish();

    EffectAsset::new(4096, SpawnerSettings::once(1400.0.into()), module)
        .with_name("dice_electric_bolt")
        .init(SetAttributeModifier::new(Attribute::POSITION, pos.expr()))
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

fn make_plasma_core_fx() -> EffectAsset {
    let mut color = bevy_hanabi::Gradient::new();
    // HDR purple/blue core glow.
    color.add_key(0.0, Vec4::new(8.0, 2.0, 10.0, 0.95));
    color.add_key(0.7, Vec4::new(4.0, 1.2, 8.0, 0.65));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.05));
    size.add_key(1.0, Vec3::splat(0.02));

    let w = ExprWriter::new();
    let lifetime = w.lit(0.18).add(w.rand(ScalarType::Float).mul(w.lit(0.18)));
    let center = w.lit(Vec3::ZERO).expr();
    let radius = w.lit(0.10).expr();
    let speed = w.lit(0.4).add(w.rand(ScalarType::Float).mul(w.lit(0.6)));
    let drag = w.lit(3.0).expr();
    let module = w.finish();

    EffectAsset::new(4096, SpawnerSettings::rate(220.0.into()), module)
        .with_name("dice_plasma_core")
        .init(SetPositionSphereModifier {
            center,
            radius,
            dimension: ShapeDimension::Volume,
        })
        .init(SetVelocitySphereModifier {
            center,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

fn make_plasma_filaments_fx() -> EffectAsset {
    let mut color = bevy_hanabi::Gradient::new();
    // Bright pink filaments which fade quickly.
    color.add_key(0.0, Vec4::new(14.0, 2.0, 12.0, 1.0));
    color.add_key(0.5, Vec4::new(8.0, 1.5, 10.0, 0.85));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.03));
    size.add_key(1.0, Vec3::splat(0.01));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::ZERO).expr();
    let radius = w.lit(0.22).expr();

    // Tangential speed in [1.8:6.5]
    let speed = w
        .lit(1.8)
        .add(w.rand(ScalarType::Float).mul(w.lit(4.7)));

    // Lifetime in [0.10:0.28]
    let lifetime = w
        .lit(0.10)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.18)));

    let drag = w.lit(5.5).expr();
    let origin = w.lit(Vec3::ZERO).expr();
    let axis = w.lit(Vec3::Y).expr();

    let module = w.finish();

    EffectAsset::new(12000, SpawnerSettings::rate(520.0.into()), module)
        .with_name("dice_plasma_filaments")
        .init(SetPositionSphereModifier {
            center,
            radius,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocityTangentModifier {
            origin,
            axis,
            speed: speed.expr(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr()))
        .update(LinearDragModifier::new(drag))
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}
