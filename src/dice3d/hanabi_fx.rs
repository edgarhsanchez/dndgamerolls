use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::dice3d::types::DiceRollFxKind;

#[derive(Resource, Clone)]
pub struct DiceHanabiFxAssets {
    pub fire_core: Handle<EffectAsset>,
    pub fire_smoke: Handle<EffectAsset>,
    pub electricity: Handle<EffectAsset>,
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
    let fire_smoke = effects.add(make_fire_smoke_fx());
    let electricity = effects.add(make_electric_wander_fx());
    let fireworks = effects.add(make_fireworks_fx());
    let explosion = effects.add(make_explosion_fx());
    let plasma_core = effects.add(make_plasma_core_fx());
    let plasma_filaments = effects.add(make_plasma_filaments_fx());

    commands.insert_resource(DiceHanabiFxAssets {
        fire_core,
        fire_smoke,
        electricity,
        fireworks,
        explosion,
        plasma_core,
        plasma_filaments,
    });
}

pub fn fx_handles_for_kind(assets: &DiceHanabiFxAssets, kind: DiceRollFxKind) -> Vec<Handle<EffectAsset>> {
    match kind {
        DiceRollFxKind::None => vec![],
        DiceRollFxKind::Fire => vec![assets.fire_core.clone(), assets.fire_smoke.clone()],
        DiceRollFxKind::Electricity => vec![assets.electricity.clone()],
        DiceRollFxKind::Fireworks => vec![assets.fireworks.clone()],
        DiceRollFxKind::Explosion => vec![assets.explosion.clone()],
        DiceRollFxKind::Plasma => vec![assets.plasma_core.clone(), assets.plasma_filaments.clone()],
    }
}

fn make_fire_core_fx() -> EffectAsset {
    let mut color = bevy_hanabi::Gradient::new();
    // HDR-bright colors to drive bloom.
    color.add_key(0.0, Vec4::new(10.0, 2.5, 0.2, 1.0));
    color.add_key(0.35, Vec4::new(10.0, 3.5, 0.6, 0.85));
    color.add_key(1.0, Vec4::new(0.3, 0.1, 0.02, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.09));
    size.add_key(0.4, Vec3::splat(0.13));
    size.add_key(1.0, Vec3::splat(0.01));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::new(0.0, 0.12, 0.0)).expr();
    let axis = w.lit(Vec3::Y).expr();
    let radius = w.lit(0.12).expr();

    // Lifetime in [0.35:0.95]
    let lifetime = w
        .lit(0.35)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.60)));

    // Speed in [0.8:2.4]
    let speed = w
        .lit(0.80)
        .add(w.rand(ScalarType::Float).mul(w.lit(1.60)));

    // Use a sphere velocity from a point slightly below the die so the
    // resulting direction is biased upward.
    let vel_center = w.lit(Vec3::new(0.0, -0.08, 0.0)).expr();

    // Mild upward acceleration to keep flame feeling "alive".
    let accel = w.lit(Vec3::new(0.0, 1.2, 0.0)).expr();
    let drag = w.lit(2.2).expr();

    let module = w.finish();

    EffectAsset::new(4096, SpawnerSettings::rate(260.0.into()), module)
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
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(10.0, 10.0, 10.0, 1.0));
    color.add_key(0.6, Vec4::new(6.0, 5.0, 2.0, 0.7));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.03));
    size.add_key(1.0, Vec3::splat(0.01));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::new(0.0, 0.15, 0.0)).expr();
    let radius = w.lit(0.04).expr();

    // Speed in [2.5:6.5]
    let speed = w
        .lit(2.5)
        .add(w.rand(ScalarType::Float).mul(w.lit(4.0)));

    // Lifetime in [0.55:1.05]
    let lifetime = w
        .lit(0.55)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.50)));

    let gravity = w.lit(Vec3::new(0.0, -6.0, 0.0)).expr();
    let drag = w.lit(1.2).expr();
    let zero = w.lit(Vec3::ZERO).expr();

    let module = w.finish();

    EffectAsset::new(12000, SpawnerSettings::once(1600.0.into()), module)
        .with_name("dice_fireworks")
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
    // Bright, short-lived "arc particles" emitted from a moving emitter.
    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(2.0, 8.0, 18.0, 1.0));
    color.add_key(0.4, Vec4::new(1.0, 6.0, 16.0, 0.9));
    color.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.035));
    size.add_key(1.0, Vec3::splat(0.010));

    let w = ExprWriter::new();
    let center = w.lit(Vec3::ZERO).expr();
    let radius = w.lit(0.25).expr();

    // Speed in [2.5:9.5]
    let speed = w
        .lit(2.5)
        .add(w.rand(ScalarType::Float).mul(w.lit(7.0)));

    // Lifetime in [0.08:0.22]
    let lifetime = w
        .lit(0.08)
        .add(w.rand(ScalarType::Float).mul(w.lit(0.14)));

    let drag = w.lit(4.0).expr();
    let module = w.finish();

    EffectAsset::new(14000, SpawnerSettings::rate(900.0.into()), module)
        .with_name("dice_electric_wander")
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
        .render(OrientModifier::new(OrientMode::AlongVelocity))
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
