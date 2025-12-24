use bevy::prelude::*;

use crate::dice3d::dice_fx::{
    CustomDiceFxTextures, DiceFxMeshes, DiceFxRollingTracker, DicePlumeFxMaterial,
    DiceSurfaceFxMaterial,
};
use crate::dice3d::types::*;

use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::path::Path;

#[derive(Component, Clone, Copy, Debug)]
pub struct DiceFxOwner(pub Entity);

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
        cache.noise = None;
        cache.ramp = None;
        cache.mask = None;
        cache.noise_path = None;
        cache.ramp_path = None;
        cache.mask_path = None;
        return;
    };

    if !cfg.enabled {
        return;
    }

    let (Some(noise_path), Some(ramp_path), Some(mask_path)) = (
        cfg.noise_image_path.as_ref(),
        cfg.ramp_image_path.as_ref(),
        cfg.mask_image_path.as_ref(),
    ) else {
        return;
    };

    let changed = cache.noise_path.as_deref() != Some(noise_path)
        || cache.ramp_path.as_deref() != Some(ramp_path)
        || cache.mask_path.as_deref() != Some(mask_path);

    if changed {
        cache.noise = None;
        cache.ramp = None;
        cache.mask = None;
        cache.noise_path = Some(noise_path.clone());
        cache.ramp_path = Some(ramp_path.clone());
        cache.mask_path = Some(mask_path.clone());
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
    shells: &Query<(), With<DiceFxSurfaceShell>>,
    fire: &Query<(), With<DiceFxFirePlume>>,
    atomic: &Query<(), With<DiceFxAtomicPlume>>,
) {
    let Some(children) = children else {
        return;
    };

    for child in children.iter() {
        if shells.get(child).is_ok() || fire.get(child).is_ok() || atomic.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

pub fn clear_dice_fx_on_roll_start(
    mut commands: Commands,
    roll_state: Res<RollState>,
    mut tracker: ResMut<DiceFxRollingTracker>,
    dice_query: Query<(Entity, Option<&Children>), With<Die>>,
    shells: Query<(), With<DiceFxSurfaceShell>>,
    fire: Query<(), With<DiceFxFirePlume>>,
    atomic: Query<(), With<DiceFxAtomicPlume>>,
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
            &shells,
            &fire,
            &atomic,
        );
    }
}

pub fn apply_dice_fx_from_roll_complete(
    mut commands: Commands,
    mut ev: MessageReader<DiceRollCompletedEvent>,
    time: Res<Time>,
    settings_state: Res<SettingsState>,
) {
    for event in ev.read() {
        let nat20_count = event
            .results
            .iter()
            .filter(|r| r.die_type == DiceType::D20 && r.value == 20)
            .count();
        let atomic_active = nat20_count >= 2;

        let custom_cfg = settings_state.settings.custom_dice_fx.as_ref();
        let custom_active = custom_cfg
            .map(|cfg| trigger_custom_fx(cfg, &event.results))
            .unwrap_or(false);
        let custom_duration = custom_cfg
            .map(|cfg| cfg.duration_seconds.max(0.0))
            .unwrap_or(0.0);
        let custom_started_at = time.elapsed_secs();

        for r in &event.results {
            let max = r.die_type.max_value();
            let electric = r.value == max;
            let fire = r.die_type == DiceType::D20 && r.value == 20;
            let atomic = atomic_active && fire;

            commands.entity(r.entity).insert(DieLastRoll { value: r.value });
            commands.entity(r.entity).insert(DiceFxState {
                fire,
                atomic,
                electric,
                custom: custom_active,
                custom_started_at,
                custom_duration,
            });
        }
    }
}

pub fn sync_dice_fx_visuals(
    mut commands: Commands,
    fx_meshes: Res<DiceFxMeshes>,
    asset_server: Res<AssetServer>,
    settings_state: Res<SettingsState>,
    mut custom_textures: ResMut<CustomDiceFxTextures>,
    mut images: ResMut<Assets<Image>>,
    mut surface_materials: ResMut<Assets<DiceSurfaceFxMaterial>>,
    mut plume_materials: ResMut<Assets<DicePlumeFxMaterial>>,
    dice_query: Query<(Entity, &DiceFxState, &Mesh3d, &Transform, Option<&Children>), With<Die>>,
    shells: Query<(), With<DiceFxSurfaceShell>>,
    fire: Query<(), With<DiceFxFirePlume>>,
    atomic: Query<(), With<DiceFxAtomicPlume>>,
) {
    let surface_opacity = effective_dice_fx_surface_opacity(&settings_state);
    let plume_h_mul = effective_dice_fx_plume_height_multiplier(&settings_state);
    let plume_r_mul = effective_dice_fx_plume_radius_multiplier(&settings_state);

    ensure_custom_fx_textures_loaded(
        settings_state.settings.custom_dice_fx.as_ref(),
        &mut custom_textures,
        &mut images,
    );

    for (die_entity, state, die_mesh, die_transform, children) in dice_query.iter() {
        let any = state.fire || state.atomic || state.electric || state.custom;

        if !any {
            despawn_all_fx_children(
                &mut commands,
                children,
                &shells,
                &fire,
                &atomic,
            );
            continue;
        }

        let die_scale = die_transform.scale.x.max(die_transform.scale.y).max(die_transform.scale.z);

        let has_shell = children
            .map(|ch| ch.iter().any(|c| shells.get(c).is_ok()))
            .unwrap_or(false);
        if !has_shell {
            let origin_ws = die_transform.translation;

            let (custom_noise, custom_ramp, custom_mask) = (
                custom_textures
                    .noise
                    .clone()
                    .unwrap_or_else(|| asset_server.load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_NOISE_PATH)),
                custom_textures
                    .ramp
                    .clone()
                    .unwrap_or_else(|| asset_server.load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_RAMP_PATH)),
                custom_textures
                    .mask
                    .clone()
                    .unwrap_or_else(|| asset_server.load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_MASK_PATH)),
            );

            let mut mat = DiceSurfaceFxMaterial {
                base: StandardMaterial {
                    base_color: Color::srgba(0.0, 0.0, 0.0, surface_opacity),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                },
                extension: crate::dice3d::dice_fx::DiceSurfaceFxExtension {
                    params: crate::dice3d::dice_fx::DiceSurfaceFxParams {
                        time: 0.0,
                        fire: if state.fire { 1.0 } else { 0.0 },
                        atomic_fx: if state.atomic { 1.0 } else { 0.0 },
                        electric: if state.electric { 1.0 } else { 0.0 },
                        origin_ws,
                        custom: if state.custom { 1.0 } else { 0.0 },
                        custom_noise: 0.0,
                        custom_mask: 0.0,
                        custom_hue: 0.0,
                    },
                    fire_noise: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_NOISE_PATH),
                    fire_ramp: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_RAMP_PATH),
                    fire_mask: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_MASK_PATH),

                    atomic_noise: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_NOISE_PATH),
                    atomic_ramp: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_RAMP_PATH),
                    atomic_mask: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_MASK_PATH),

                    electric_noise: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_NOISE_PATH),
                    electric_ramp: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_RAMP_PATH),
                    electric_mask: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ELECTRIC_MASK_PATH),

                    custom_noise,
                    custom_ramp,
                    custom_mask,
                },
            };

            // Ensure alpha looks nice.
            mat.base.reflectance = 0.0;
            mat.base.perceptual_roughness = 1.0;

            let mat_handle = surface_materials.add(mat);

            commands.entity(die_entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(die_mesh.0.clone()),
                    MeshMaterial3d(mat_handle.clone()),
                    Transform::from_scale(Vec3::splat(1.03)),
                    Visibility::Visible,
                    DiceFxSurfaceShell,
                    DiceFxOwner(die_entity),
                    DiceFxMaterialHandle::<DiceSurfaceFxMaterial>(mat_handle),
                ));
            });
        }

        let has_fire = state.fire
            && children
                .map(|ch| ch.iter().any(|c| fire.get(c).is_ok()))
                .unwrap_or(false);
        if state.fire && !has_fire {
            let origin_ws = die_transform.translation;
            let mut params = crate::dice3d::dice_fx::DicePlumeFxParams {
                time: 0.0,
                intensity: 1.0,
                kind: 0.0,
                color: Vec4::ZERO,
                origin_ws,
                _pad0: 0.0,
            };
            params.set_color(Color::srgba(0.95, 0.25, 0.10, 1.0));

            let mat_handle = plume_materials.add(DicePlumeFxMaterial {
                base: StandardMaterial {
                    base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                    alpha_mode: AlphaMode::Add,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                },
                extension: crate::dice3d::dice_fx::DicePlumeFxExtension {
                    params,
                    fire_noise: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_NOISE_PATH),
                    fire_ramp: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_RAMP_PATH),
                    fire_mask: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_MASK_PATH),
                    atomic_noise: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_NOISE_PATH),
                    atomic_ramp: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_RAMP_PATH),
                    atomic_mask: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_MASK_PATH),
                },
            });

            let height = 1.2 * die_scale;
            let radius = 0.55 * die_scale;

            let height = height * plume_h_mul;
            let radius = radius * plume_r_mul;

            commands.entity(die_entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(fx_meshes.plume.clone()),
                    MeshMaterial3d(mat_handle.clone()),
                    Transform::from_translation(Vec3::ZERO)
                        .with_scale(Vec3::new(radius, height, radius)),
                    Visibility::Visible,
                    DiceFxFirePlume,
                    DiceFxOwner(die_entity),
                    DiceFxMaterialHandle::<DicePlumeFxMaterial>(mat_handle),
                ));
            });
        }

        let has_atomic = state.atomic
            && children
                .map(|ch| ch.iter().any(|c| atomic.get(c).is_ok()))
                .unwrap_or(false);
        if state.atomic && !has_atomic {
            let origin_ws = die_transform.translation;
            let mut params = crate::dice3d::dice_fx::DicePlumeFxParams {
                time: 0.0,
                intensity: 2.2,
                kind: 1.0,
                color: Vec4::ZERO,
                origin_ws,
                _pad0: 0.0,
            };
            params.set_color(Color::srgba(1.0, 0.95, 0.70, 1.0));

            let mat_handle = plume_materials.add(DicePlumeFxMaterial {
                base: StandardMaterial {
                    base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                    alpha_mode: AlphaMode::Add,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                },
                extension: crate::dice3d::dice_fx::DicePlumeFxExtension {
                    params,
                    fire_noise: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_NOISE_PATH),
                    fire_ramp: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_RAMP_PATH),
                    fire_mask: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_FIRE_MASK_PATH),
                    atomic_noise: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_NOISE_PATH),
                    atomic_ramp: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_RAMP_PATH),
                    atomic_mask: asset_server
                        .load(crate::dice3d::embedded_assets::DICE_FX_ATOMIC_MASK_PATH),
                },
            });

            let height = 1.8 * die_scale;
            let radius = 0.85 * die_scale;

            let height = height * plume_h_mul;
            let radius = radius * plume_r_mul;

            commands.entity(die_entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(fx_meshes.plume.clone()),
                    MeshMaterial3d(mat_handle.clone()),
                    Transform::from_translation(Vec3::ZERO)
                        .with_scale(Vec3::new(radius, height, radius)),
                    Visibility::Visible,
                    DiceFxAtomicPlume,
                    DiceFxOwner(die_entity),
                    DiceFxMaterialHandle::<DicePlumeFxMaterial>(mat_handle),
                ));
            });
        }
    }
}

pub fn tick_dice_fx_material_time(
    time: Res<Time>,
    settings_state: Res<SettingsState>,
    mut surface_materials: ResMut<Assets<DiceSurfaceFxMaterial>>,
    mut plume_materials: ResMut<Assets<DicePlumeFxMaterial>>,
    shell_query: Query<(
        &DiceFxOwner,
        &DiceFxMaterialHandle<DiceSurfaceFxMaterial>,
    )>,
    plume_query: Query<(
        &DiceFxOwner,
        &DiceFxMaterialHandle<DicePlumeFxMaterial>,
    )>,
    state_query: Query<&DiceFxState, With<Die>>,
    owner_transform_query: Query<&GlobalTransform, With<Die>>,
) {
    let t = time.elapsed_secs();

    let surface_opacity = effective_dice_fx_surface_opacity(&settings_state);

    let custom_cfg = settings_state.settings.custom_dice_fx.as_ref();

    for (owner, mat_handle) in shell_query.iter() {
        if let Some(mat) = surface_materials.get_mut(&mat_handle.0) {
            mat.extension.params.time = t;

            if let Ok(gt) = owner_transform_query.get(owner.0) {
                mat.extension.params.origin_ws = gt.translation();
            }

            if let Ok(state) = state_query.get(owner.0) {
                mat.extension.params.fire = if state.fire { 1.0 } else { 0.0 };
                mat.extension.params.atomic_fx = if state.atomic { 1.0 } else { 0.0 };
                mat.extension.params.electric = if state.electric { 1.0 } else { 0.0 };

                // Keep shell opacity synced (supports live preview while modal is open).
                // If a custom effect is active, apply the opacity curve as an envelope (0..1)
                // multiplied by the base slider opacity.
                let mut alpha = surface_opacity;
                if state.custom {
                    let dur = state.custom_duration.max(0.0);
                    if dur > 0.0001 {
                        let age = (t - state.custom_started_at).max(0.0);
                        if age < dur {
                            let u = (age / dur).clamp(0.0, 1.0);
                            if let Some(cfg) = custom_cfg {
                                alpha *= sample_custom_curve01(&cfg.curve_points_opacity, u)
                                    .clamp(0.0, 1.0);
                            }
                        }
                    }
                }
                mat.base.base_color = Color::srgba(0.0, 0.0, 0.0, alpha.clamp(0.0, 1.0));

                // Custom intensity: normalized time sampled through the mask curve (0..1).
                let (custom_intensity, custom_noise, custom_mask, custom_hue) = if state.custom {
                    let dur = state.custom_duration.max(0.0);
                    if dur <= 0.0001 {
                        (0.0, 0.0, 0.0, 0.0)
                    } else {
                        let age = (t - state.custom_started_at).max(0.0);
                        if age >= dur {
                            (0.0, 0.0, 0.0, 0.0)
                        } else {
                            let u = (age / dur).clamp(0.0, 1.0);
                            if let Some(cfg) = custom_cfg {
                                (
                                    sample_custom_curve01(&cfg.curve_points_mask, u),
                                    sample_custom_curve01(&cfg.curve_points_noise, u),
                                    sample_custom_curve01(&cfg.curve_points_mask, u),
                                    sample_custom_curve01(&cfg.curve_points_ramp, u),
                                )
                            } else {
                                (1.0, 1.0, 1.0, 0.0)
                            }
                        }
                    }
                } else {
                    (0.0, 0.0, 0.0, 0.0)
                };

                mat.extension.params.custom = custom_intensity;
                mat.extension.params.custom_noise = custom_noise;
                mat.extension.params.custom_mask = custom_mask;
                mat.extension.params.custom_hue = custom_hue;
            } else {
                // If we can't read state, still keep opacity synced.
                mat.base.base_color = Color::srgba(0.0, 0.0, 0.0, surface_opacity);
            }
        }
    }

    for (owner, mat_handle) in plume_query.iter() {
        if let Some(mat) = plume_materials.get_mut(&mat_handle.0) {
            mat.extension.params.time = t;

            if let Ok(gt) = owner_transform_query.get(owner.0) {
                mat.extension.params.origin_ws = gt.translation();
            }
        }
    }
}

pub fn sync_dice_fx_plume_transforms(
    time: Res<Time>,
    settings_state: Res<SettingsState>,
    owner_query: Query<&GlobalTransform, With<Die>>,
    state_query: Query<&DiceFxState, With<Die>>,
    mut fire_plumes: Query<
        (&DiceFxOwner, &mut Transform),
        (With<DiceFxFirePlume>, Without<DiceFxAtomicPlume>),
    >,
    mut atomic_plumes: Query<
        (&DiceFxOwner, &mut Transform),
        (With<DiceFxAtomicPlume>, Without<DiceFxFirePlume>),
    >,
) {
    let t = time.elapsed_secs();

    let plume_h_mul = effective_dice_fx_plume_height_multiplier(&settings_state);
    let plume_r_mul = effective_dice_fx_plume_radius_multiplier(&settings_state);

    let custom_cfg = settings_state.settings.custom_dice_fx.as_ref();

    for (owner, mut transform) in fire_plumes.iter_mut() {
        let Ok(gt) = owner_query.get(owner.0) else {
            continue;
        };
        let s = gt
            .to_scale_rotation_translation()
            .0;
        let die_scale = s.x.max(s.y).max(s.z);

        let (h_mul, r_mul) = match state_query.get(owner.0) {
            Ok(state) if state.custom && state.custom_duration > 0.0001 => {
                let age = (t - state.custom_started_at).max(0.0);
                if age < state.custom_duration {
                    let u = (age / state.custom_duration).clamp(0.0, 1.0);
                    if let Some(cfg) = custom_cfg {
                        (
                            plume_h_mul
                                * sample_custom_curve01(&cfg.curve_points_plume_height, u)
                                    .clamp(0.0, 1.0),
                            plume_r_mul
                                * sample_custom_curve01(&cfg.curve_points_plume_radius, u)
                                    .clamp(0.0, 1.0),
                        )
                    } else {
                        (plume_h_mul, plume_r_mul)
                    }
                } else {
                    (plume_h_mul, plume_r_mul)
                }
            }
            _ => (plume_h_mul, plume_r_mul),
        };

        let height = 1.2 * die_scale * h_mul;
        let radius = 0.55 * die_scale * r_mul;

        transform.translation = Vec3::ZERO;
        transform.scale = Vec3::new(radius, height, radius);
    }

    for (owner, mut transform) in atomic_plumes.iter_mut() {
        let Ok(gt) = owner_query.get(owner.0) else {
            continue;
        };
        let s = gt
            .to_scale_rotation_translation()
            .0;
        let die_scale = s.x.max(s.y).max(s.z);

        let (h_mul, r_mul) = match state_query.get(owner.0) {
            Ok(state) if state.custom && state.custom_duration > 0.0001 => {
                let age = (t - state.custom_started_at).max(0.0);
                if age < state.custom_duration {
                    let u = (age / state.custom_duration).clamp(0.0, 1.0);
                    if let Some(cfg) = custom_cfg {
                        (
                            plume_h_mul
                                * sample_custom_curve01(&cfg.curve_points_plume_height, u)
                                    .clamp(0.0, 1.0),
                            plume_r_mul
                                * sample_custom_curve01(&cfg.curve_points_plume_radius, u)
                                    .clamp(0.0, 1.0),
                        )
                    } else {
                        (plume_h_mul, plume_r_mul)
                    }
                } else {
                    (plume_h_mul, plume_r_mul)
                }
            }
            _ => (plume_h_mul, plume_r_mul),
        };

        let height = 1.8 * die_scale * h_mul;
        let radius = 0.85 * die_scale * r_mul;

        transform.translation = Vec3::ZERO;
        transform.scale = Vec3::new(radius, height, radius);
    }
}

pub fn billboard_dice_fx_plumes_to_camera(
    camera_query: Query<&GlobalTransform, (With<Camera3d>, With<MainCamera>)>,
    owner_query: Query<&GlobalTransform, With<Die>>,
    mut plumes: Query<
        (&DiceFxOwner, &mut Transform),
        Or<(With<DiceFxFirePlume>, With<DiceFxAtomicPlume>)>,
    >,
) {
    let Ok(camera_gt) = camera_query.single() else {
        return;
    };
    let cam_pos = camera_gt.translation();

    for (owner, mut transform) in plumes.iter_mut() {
        let Ok(owner_gt) = owner_query.get(owner.0) else {
            continue;
        };

        // Keep the plume centered on the die, but rotate it so its *global*
        // orientation faces the camera (i.e., don't inherit the die's rotation).
        transform.translation = Vec3::ZERO;

        let owner_pos = owner_gt.translation();
        let mut desired_world = Transform::from_translation(owner_pos);
        desired_world.look_at(cam_pos, Vec3::Y);
        let desired_world_rot = desired_world.rotation;

        let (_s, owner_world_rot, _t) = owner_gt.to_scale_rotation_translation();
        transform.rotation = owner_world_rot.inverse() * desired_world_rot;
    }
}

fn sample_custom_curve01(points: &[FxCurvePointSetting], t: f32) -> f32 {
    if points.is_empty() {
        return 1.0;
    }
    if points.len() == 1 {
        return points[0].value.clamp(0.0, 1.0);
    }

    let t = t.clamp(0.0, 1.0);
    let mut points_sorted: std::borrow::Cow<'_, [FxCurvePointSetting]> =
        std::borrow::Cow::Borrowed(points);
    if !points.windows(2).all(|w| w[0].t <= w[1].t) {
        let mut tmp = points.to_vec();
        tmp.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
        points_sorted = std::borrow::Cow::Owned(tmp);
    }
    let points = points_sorted.as_ref();

    if t <= points[0].t {
        return points[0].value.clamp(0.0, 1.0);
    }
    if t >= points[points.len() - 1].t {
        return points[points.len() - 1].value.clamp(0.0, 1.0);
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

    for w in points.windows(2) {
        let a = &w[0];
        let b = &w[1];
        if t >= a.t && t <= b.t {
            let dt = (b.t - a.t).max(0.0001);
            let initial_u = ((t - a.t) / dt).clamp(0.0, 1.0);

            let mut p1 = a.out_handle.map(|h| Vec2::new(h[0], h[1])).unwrap_or(Vec2::new(
                lerp(a.t, b.t, 1.0 / 3.0),
                a.value,
            ));
            let mut p2 = b.in_handle.map(|h| Vec2::new(h[0], h[1])).unwrap_or(Vec2::new(
                lerp(a.t, b.t, 2.0 / 3.0),
                b.value,
            ));

            p1.x = p1.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p2.x = p2.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p1.y = p1.y.clamp(0.0, 1.0);
            p2.y = p2.y.clamp(0.0, 1.0);

            let mut u = initial_u;
            for _ in 0..8 {
                let x = cubic_bezier(a.t, p1.x, p2.x, b.t, u);
                let dx = cubic_bezier_derivative(a.t, p1.x, p2.x, b.t, u);
                if dx.abs() < 1e-5 {
                    break;
                }
                u = (u - (x - t) / dx).clamp(0.0, 1.0);
            }

            return cubic_bezier(a.value, p1.y, p2.y, b.value, u).clamp(0.0, 1.0);
        }
    }

    points[0].value.clamp(0.0, 1.0)
}

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
