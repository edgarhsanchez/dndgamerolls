use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::dice3d::systems::dice_fx::{
    apply_dice_fx_from_roll_complete, clear_dice_fx_on_roll_start, init_dice_fx_sounds,
    sync_dice_fx_visuals, tick_delayed_sfx,
};
use crate::dice3d::systems::dice_fx::tick_delayed_fx_bursts;

#[derive(Resource, Clone)]
pub struct DiceFxEffectAssets {
    pub fire: Handle<EffectAsset>,
    pub lightning: Handle<EffectAsset>,
    pub firework: Handle<EffectAsset>,
    pub explosion: Handle<EffectAsset>,
}

impl DiceFxEffectAssets {
    pub fn is_ready(&self) -> bool {
        // Handles are always valid even if the underlying asset isn't, but
        // keeping this helper makes call sites clearer.
        true
    }
}

fn create_fire_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    // Runtime properties (animatable via EffectProperties)
    // Smaller by default to keep the plume tight around the die.
    let p_spawn_radius = writer.add_property("spawn_radius", 0.35.into());
    let p_speed_mul = writer.add_property("speed_mul", 1.0.into());
    let p_lifetime_mul = writer.add_property("lifetime_mul", 1.0.into());
    let p_drag_mul = writer.add_property("drag_mul", 1.0.into());
    let p_accel_mul = writer.add_property("accel_mul", 1.0.into());

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.prop(p_spawn_radius).expr(),
        // Fill volume so the plume reads less like a hollow shell.
        dimension: ShapeDimension::Volume,
    };

    // Center vs edge shaping.
    // - F32_0: base size (center larger)
    // - COLOR: base hue/intensity (center whiter/brighter, edge redder/dimmer)
    let radial = (writer.attr(Attribute::POSITION).length() / writer.prop(p_spawn_radius))
        .min(writer.lit(1.0))
        .max(writer.lit(0.0));
    let core = writer.lit(1.0) - radial;
    // Bias heavily toward the center so the hot core reads clearly.
    let core_t = core.clone() * core.clone();

    let base_size = writer
        .lit(0.06)
        .mix(writer.lit(0.15), core_t.clone());
    let init_base_size = SetAttributeModifier::new(Attribute::F32_0, base_size.expr());

    // Base color is LDR (packed). We use it to differentiate center/edge, then
    // apply an HDR color gradient via Modulate at render time.
    let base_edge = writer.lit(Vec3::new(1.0, 0.18, 0.06));
    let base_core = writer.lit(Vec3::new(1.0, 1.0, 1.0));
    let hue = base_edge.mix(base_core, core_t.clone());
    let intensity = writer
        .lit(0.35)
        .mix(writer.lit(1.0), core_t.clone());
    let base_rgb = hue * intensity;
    let base_col = base_rgb
        .vec4_xyz_w(writer.lit(1.0))
        .pack4x8unorm();
    let init_color = SetAttributeModifier::new(Attribute::COLOR, base_col.expr());

    // Upward-biased velocity with reduced lateral spread.
    let x = (writer.rand(ScalarType::Float) * writer.lit(2.0) - writer.lit(1.0)) * writer.lit(0.35);
    let z = (writer.rand(ScalarType::Float) * writer.lit(2.0) - writer.lit(1.0)) * writer.lit(0.35);
    let y = writer.lit(2.2).uniform(writer.lit(3.6));
    let v = (x.clone().vec3(y, z) * writer.prop(p_speed_mul)).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, v);

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, (writer.lit(1.6) * writer.prop(p_lifetime_mul)).expr());

    let update_accel = AccelModifier::new((writer.lit(Vec3::Y * 6.5) * writer.prop(p_accel_mul)).expr());
    let update_drag = LinearDragModifier::new((writer.lit(2.0) * writer.prop(p_drag_mul)).expr());

    // Use an HDR color/alpha gradient (as recommended by Hanabi demos) so fire
    // can bloom nicely when the camera has HDR+BLOOM enabled.
    let mut color_gradient = bevy_hanabi::Gradient::new();
    // White-hot -> orange -> red ember. Because we modulate by per-particle
    // base color, the center stays whiter/brighter while the edge skews red.
    color_gradient.add_key(0.0, Vec4::new(40.0, 40.0, 40.0, 0.0));
    color_gradient.add_key(0.06, Vec4::new(38.0, 30.0, 16.0, 0.95));
    color_gradient.add_key(0.35, Vec4::new(16.0, 6.0, 1.6, 0.90));
    color_gradient.add_key(0.70, Vec4::new(3.2, 0.7, 0.25, 0.55));
    color_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    // Size driven by per-particle base size (F32_0) and lifetime.
    let size_scale = (writer.lit(1.0)
        - (writer.attr(Attribute::AGE) / writer.attr(Attribute::LIFETIME))
            .min(writer.lit(1.0))
            .max(writer.lit(0.0)))
    .max(writer.lit(0.0));
    let update_size = SetAttributeModifier::new(
        Attribute::SIZE,
        (writer.attr(Attribute::F32_0) * size_scale).expr(),
    );

    let slot_noise = writer.lit(0u32).expr();
    let slot_ramp = writer.lit(1u32).expr();
    let slot_mask = writer.lit(2u32).expr();

    let mut module = writer.finish();
    module.add_texture_slot("noise");
    module.add_texture_slot("ramp");
    module.add_texture_slot("mask");

    EffectAsset::new(8192, SpawnerSettings::rate(300.0.into()), module)
        .with_name("dice_fire")
        .init(init_pos)
        .init(init_base_size)
        .init(init_color)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        .update(update_size)
        .render(ParticleTextureModifier {
            texture_slot: slot_ramp,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_mask,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_noise,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Modulate,
            mask: ColorBlendMask::RGBA,
        })
}

fn create_lightning_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let p_spawn_radius = writer.add_property("spawn_radius", 0.40.into());
    let p_speed_mul = writer.add_property("speed_mul", 1.0.into());
    let p_lifetime_mul = writer.add_property("lifetime_mul", 1.0.into());
    let p_drag_mul = writer.add_property("drag_mul", 1.0.into());
    let p_accel_mul = writer.add_property("accel_mul", 1.0.into());

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.prop(p_spawn_radius).expr(),
        dimension: ShapeDimension::Surface,
    };

    // Faraday-cage-like arcs: start tangent around the die and keep
    // particles pulled inward to curve their paths into orbiting streaks.
    let init_vel = SetVelocityTangentModifier {
        origin: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        speed: (writer.lit(12.0) * writer.prop(p_speed_mul)).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, (writer.lit(0.34) * writer.prop(p_lifetime_mul)).expr());

    // Keep streaks energetic; too much drag turns it into floating sparks.
    let update_drag = LinearDragModifier::new((writer.lit(2.2) * writer.prop(p_drag_mul)).expr());
    // Strong inward acceleration provides the curvature / arcing motion.
    let update_radial = RadialAccelModifier::new(
        writer.lit(Vec3::ZERO).expr(),
        (writer.lit(-120.0) * writer.prop(p_accel_mul)).expr(),
    );

    // HDR blue-white spark color with fast fade.
    let mut color_gradient = bevy_hanabi::Gradient::new();
    // Very bright initial strike, then taper to a dimmer glow.
    color_gradient.add_key(0.0, Vec4::new(40.0, 55.0, 80.0, 0.0));
    color_gradient.add_key(0.02, Vec4::new(40.0, 55.0, 80.0, 1.0));
    color_gradient.add_key(0.25, Vec4::new(10.0, 18.0, 40.0, 1.0));
    color_gradient.add_key(0.70, Vec4::new(2.0, 4.5, 14.0, 0.75));
    color_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    // Long thin streaks, oriented along velocity (see portal.rs).
    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::new(0.80, 0.045, 0.045));
    size_gradient.add_key(1.0, Vec3::new(0.35, 0.028, 0.028));

    let slot_noise = writer.lit(0u32).expr();
    let _slot_ramp = writer.lit(1u32).expr();
    let slot_mask = writer.lit(2u32).expr();

    let mut module = writer.finish();
    module.add_texture_slot("noise");
    module.add_texture_slot("ramp");
    module.add_texture_slot("mask");

    // Add some sustained tangential forcing so paths keep “shooting around”.
    let update_tangent = TangentAccelModifier::constant(&mut module, Vec3::ZERO, Vec3::Y, 18.0);

    EffectAsset::new(8192, SpawnerSettings::rate(520.0.into()), module)
        .with_name("dice_lightning")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_radial)
        .update(update_tangent)
        .update(update_drag)
        .render(ParticleTextureModifier {
            texture_slot: slot_mask,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_noise,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier::new(color_gradient))
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn create_firework_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.35).expr(),
        dimension: ShapeDimension::Surface,
    };

    // Upward-biased burst.
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::new(0.0, -1.5, 0.0)).expr(),
        speed: writer.lit(7.0).expr(),
    };

    // Random bright color per particle (packed u32).
    let rgb = writer.rand(VectorType::VEC3F) * writer.lit(Vec3::splat(0.6)) + writer.lit(Vec3::splat(0.4));
    let col = rgb.vec4_xyz_w(writer.lit(1.0)).pack4x8unorm();
    let init_color = SetAttributeModifier::new(Attribute::COLOR, col.expr());

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.85).expr());

    let update_accel = AccelModifier::new(writer.lit(Vec3::NEG_Y * 9.5).expr());
    let update_drag = LinearDragModifier::new(writer.lit(0.7).expr());

    let mut alpha_gradient = bevy_hanabi::Gradient::new();
    alpha_gradient.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 0.0));
    alpha_gradient.add_key(0.05, Vec4::new(0.0, 0.0, 0.0, 1.0));
    alpha_gradient.add_key(0.75, Vec4::new(0.0, 0.0, 0.0, 1.0));
    alpha_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut sparkle_gradient = bevy_hanabi::Gradient::new();
    // HDR sparkle boost so it can bloom.
    sparkle_gradient.add_key(0.0, Vec4::new(18.0, 18.0, 18.0, 0.0));
    sparkle_gradient.add_key(0.20, Vec4::new(10.0, 10.0, 10.0, 0.0));
    sparkle_gradient.add_key(0.55, Vec4::new(4.0, 4.0, 4.0, 0.0));
    sparkle_gradient.add_key(1.0, Vec4::ZERO);

    // Slightly elongated sparks.
    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::new(0.18, 0.05, 0.05));
    size_gradient.add_key(1.0, Vec3::new(0.09, 0.03, 0.03));

    let slot_noise = writer.lit(0u32).expr();
    let slot_ramp = writer.lit(1u32).expr();
    let slot_mask = writer.lit(2u32).expr();

    let mut module = writer.finish();
    module.add_texture_slot("noise");
    module.add_texture_slot("ramp");
    module.add_texture_slot("mask");

    EffectAsset::new(4096, SpawnerSettings::once(140.0.into()), module)
        .with_name("dice_firework")
        .init(init_pos)
        .init(init_vel)
        .init(init_color)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        // Keep per-particle random color; avoid ramp RGB modulation so fireworks
        // don’t get tinted into a single hue.
        .render(ParticleTextureModifier {
            texture_slot: slot_ramp,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_mask,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_noise,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier {
            gradient: alpha_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::A,
        })
        .render(ColorOverLifetimeModifier {
            gradient: sparkle_gradient,
            blend: ColorBlendMode::Add,
            mask: ColorBlendMask::RGB,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::AlongVelocity))
}

fn create_explosion_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.40).expr(),
        dimension: ShapeDimension::Volume,
    };

    // "Atomic" style: huge initial flash + fast outward expansion, then buoyant rise.
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::new(0.0, -1.4, 0.0)).expr(),
        speed: writer.lit(9.5).expr(),
    };

    // Warm explosion color range.
    let t = writer.rand(ScalarType::Float);
    let rgb = writer
        .lit(Vec3::new(1.0, 0.85, 0.35))
        .mix(writer.lit(Vec3::new(1.0, 0.35, 0.05)), t);
    let col = rgb.vec4_xyz_w(writer.lit(1.0)).pack4x8unorm();
    let init_color = SetAttributeModifier::new(Attribute::COLOR, col.expr());

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(2.60).expr());

    // Drag keeps it from instantly disappearing off-screen, and upward accel helps form the rise.
    let update_drag = LinearDragModifier::new(writer.lit(2.10).expr());
    let update_accel = AccelModifier::new(writer.lit(Vec3::Y * 3.2).expr());

    let mut alpha_gradient = bevy_hanabi::Gradient::new();
    alpha_gradient.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 0.0));
    alpha_gradient.add_key(0.015, Vec4::new(0.0, 0.0, 0.0, 1.0));
    alpha_gradient.add_key(0.35, Vec4::new(0.0, 0.0, 0.0, 0.9));
    alpha_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut hot_to_smoke = bevy_hanabi::Gradient::new();
    // Very hot flash -> orange fireball -> dirty smoke.
    hot_to_smoke.add_key(0.0, Vec4::new(80.0, 70.0, 55.0, 0.0));
    hot_to_smoke.add_key(0.05, Vec4::new(35.0, 18.0, 6.0, 0.0));
    hot_to_smoke.add_key(0.20, Vec4::new(10.0, 4.0, 1.2, 0.0));
    hot_to_smoke.add_key(0.55, Vec4::new(1.2, 0.6, 0.35, 0.0));
    hot_to_smoke.add_key(0.85, Vec4::new(0.25, 0.25, 0.25, 0.0));
    hot_to_smoke.add_key(1.0, Vec4::ZERO);

    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.18));
    size_gradient.add_key(0.10, Vec3::splat(0.75));
    size_gradient.add_key(0.40, Vec3::splat(1.45));
    size_gradient.add_key(1.0, Vec3::splat(0.55));

    let slot_noise = writer.lit(0u32).expr();
    let slot_ramp = writer.lit(1u32).expr();
    let slot_mask = writer.lit(2u32).expr();

    let mut module = writer.finish();
    module.add_texture_slot("noise");
    module.add_texture_slot("ramp");
    module.add_texture_slot("mask");

    EffectAsset::new(8192, SpawnerSettings::once(520.0.into()), module)
        .with_name("dice_explosion")
        .init(init_pos)
        .init(init_vel)
        .init(init_color)
        .init(init_age)
        .init(init_lifetime)
        .update(update_accel)
        .update(update_drag)
        // Use explicit color progression; avoid ramp RGB modulation which tends
        // to make explosions look like the fire plume.
        .render(ParticleTextureModifier {
            texture_slot: slot_ramp,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_mask,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_noise,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier {
            gradient: alpha_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::A,
        })
        .render(ColorOverLifetimeModifier {
            gradient: hot_to_smoke,
            blend: ColorBlendMode::Add,
            mask: ColorBlendMask::RGB,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
}

#[allow(dead_code)]
fn create_custom_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let p_spawn_radius = writer.add_property("spawn_radius", 1.0.into());
    let p_speed_mul = writer.add_property("speed_mul", 1.0.into());
    let p_lifetime_mul = writer.add_property("lifetime_mul", 1.0.into());
    let p_drag_mul = writer.add_property("drag_mul", 1.0.into());
    let p_accel_mul = writer.add_property("accel_mul", 1.0.into());

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.prop(p_spawn_radius).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.lit(2.5) * writer.prop(p_speed_mul)).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, (writer.lit(0.9) * writer.prop(p_lifetime_mul)).expr());

    let update_drag = LinearDragModifier::new((writer.lit(2.0) * writer.prop(p_drag_mul)).expr());
    let _ = p_accel_mul;

    let mut alpha_gradient = bevy_hanabi::Gradient::new();
    alpha_gradient.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0));
    alpha_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut brighten_gradient = bevy_hanabi::Gradient::new();
    brighten_gradient.add_key(0.0, Vec4::ZERO);
    // Neutral brighten; custom ramp can provide hue, this just lifts away from dark.
    brighten_gradient.add_key(1.0, Vec4::new(0.25, 0.25, 0.25, 0.0));

    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.10));
    size_gradient.add_key(1.0, Vec3::splat(0.02));

    let slot_noise = writer.lit(0u32).expr();
    let slot_ramp = writer.lit(1u32).expr();
    let slot_mask = writer.lit(2u32).expr();
    let slot_source = writer.lit(3u32).expr();

    let mut module = writer.finish();
    module.add_texture_slot("noise");
    module.add_texture_slot("ramp");
    module.add_texture_slot("mask");
    module.add_texture_slot("source");

    EffectAsset::new(8192, SpawnerSettings::rate(260.0.into()), module)
        .with_name("dice_custom")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        // Apply user-provided textures via EffectMaterial.
        // Note: these are simple modulations; if you want true "ramp lookup"
        // behavior like the old WGSL, we can build a custom render modifier.
        .render(ParticleTextureModifier {
            texture_slot: slot_source,
            sample_mapping: ImageSampleMapping::ModulateRGB,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_ramp,
            sample_mapping: ImageSampleMapping::ModulateRGB,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_mask,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ParticleTextureModifier {
            texture_slot: slot_noise,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier {
            gradient: alpha_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::A,
        })
        .render(ColorOverLifetimeModifier {
            gradient: brighten_gradient,
            blend: ColorBlendMode::Add,
            mask: ColorBlendMask::RGB,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
}

fn setup_dice_fx_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let fire = effects.add(create_fire_effect());
    let lightning = effects.add(create_lightning_effect());
    let firework = effects.add(create_firework_effect());
    let explosion = effects.add(create_explosion_effect());

    commands.insert_resource(DiceFxEffectAssets {
        fire,
        lightning,
        firework,
        explosion,
    });
}

#[derive(Resource, Default, Clone, Copy)]
pub struct DiceFxRollingTracker {
    pub was_rolling: bool,
}

pub struct DiceFxPlugin;

impl Plugin for DiceFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_message::<crate::dice3d::types::DiceRollCompletedEvent>()
            .init_resource::<DiceFxRollingTracker>()
            .add_systems(Startup, (setup_dice_fx_effects, init_dice_fx_sounds))
            // Clear effects on roll start, apply on settle, then make visuals match.
            .add_systems(
                Update,
                clear_dice_fx_on_roll_start
                    .after(crate::dice3d::handle_input)
                    .after(crate::dice3d::handle_command_input)
                    .after(crate::dice3d::handle_quick_roll_clicks),
            )
            .add_systems(
                Update,
                apply_dice_fx_from_roll_complete.after(crate::dice3d::check_dice_settled),
            )
            .add_systems(
                Update,
                sync_dice_fx_visuals.after(apply_dice_fx_from_roll_complete),
            )
            .add_systems(
                Update,
                tick_delayed_fx_bursts.after(sync_dice_fx_visuals),
            )
            .add_systems(Update, tick_delayed_sfx.after(sync_dice_fx_visuals));
    }
}
