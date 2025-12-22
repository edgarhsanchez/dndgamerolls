//! Dice special-effects (shaders + systems)

use bevy::color::LinearRgba;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::dice3d::systems::dice_fx::{
    apply_dice_fx_from_roll_complete, clear_dice_fx_on_roll_start, sync_dice_fx_visuals,
    tick_dice_fx_material_time, expire_custom_dice_fx,
};

pub const DICE_SURFACE_FX_SHADER: &str = "shaders/dice_surface_fx.wgsl";
pub const DICE_PLUME_FX_SHADER: &str = "shaders/dice_plume_fx.wgsl";

// -----------------------------------------------------------------------------
// Surface FX (shell)
// -----------------------------------------------------------------------------

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct DiceSurfaceFxExtension {
    /// We start at binding slot 100 to avoid conflicts with `StandardMaterial`.
    #[uniform(100)]
    pub params: DiceSurfaceFxParams,

    // Textures are embedded and loaded via `embedded://...` paths.
    // Keep bindings >= 101 to avoid conflicts with `StandardMaterial`.
    #[texture(101)]
    #[sampler(102)]
    pub fire_noise: Handle<Image>,

    #[texture(103)]
    #[sampler(104)]
    pub fire_ramp: Handle<Image>,

    #[texture(105)]
    #[sampler(106)]
    pub fire_mask: Handle<Image>,

    #[texture(107)]
    #[sampler(108)]
    pub atomic_noise: Handle<Image>,

    #[texture(109)]
    #[sampler(110)]
    pub atomic_ramp: Handle<Image>,

    #[texture(111)]
    #[sampler(112)]
    pub atomic_mask: Handle<Image>,

    #[texture(113)]
    #[sampler(114)]
    pub electric_noise: Handle<Image>,

    #[texture(115)]
    #[sampler(116)]
    pub electric_ramp: Handle<Image>,

    #[texture(117)]
    #[sampler(118)]
    pub electric_mask: Handle<Image>,

    // Runtime-loaded custom textures (optional user-defined effect)
    #[texture(119)]
    #[sampler(120)]
    pub custom_noise: Handle<Image>,

    #[texture(121)]
    #[sampler(122)]
    pub custom_ramp: Handle<Image>,

    #[texture(123)]
    #[sampler(124)]
    pub custom_mask: Handle<Image>,
}

impl MaterialExtension for DiceSurfaceFxExtension {
    fn fragment_shader() -> ShaderRef {
        DICE_SURFACE_FX_SHADER.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        DICE_SURFACE_FX_SHADER.into()
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType, Reflect)]
pub struct DiceSurfaceFxParams {
    pub time: f32,
    pub fire: f32,
    pub atomic_fx: f32,
    pub electric: f32,

    pub origin_ws: Vec3,
    pub custom: f32,

    /// 0..1: increases/decreases custom noise contrast/visibility.
    pub custom_noise: f32,
    /// 0..1: shifts custom mask thresholds/contrast.
    pub custom_mask: f32,
    /// 0..1: hue shift amount applied to the sampled custom ramp color.
    pub custom_hue: f32,
}

// -----------------------------------------------------------------------------
// Plume FX (vertical)
// -----------------------------------------------------------------------------

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct DicePlumeFxExtension {
    #[uniform(100)]
    pub params: DicePlumeFxParams,

    #[texture(101)]
    #[sampler(102)]
    pub fire_noise: Handle<Image>,

    #[texture(103)]
    #[sampler(104)]
    pub fire_ramp: Handle<Image>,

    #[texture(105)]
    #[sampler(106)]
    pub fire_mask: Handle<Image>,

    #[texture(107)]
    #[sampler(108)]
    pub atomic_noise: Handle<Image>,

    #[texture(109)]
    #[sampler(110)]
    pub atomic_ramp: Handle<Image>,

    #[texture(111)]
    #[sampler(112)]
    pub atomic_mask: Handle<Image>,
}

impl MaterialExtension for DicePlumeFxExtension {
    fn fragment_shader() -> ShaderRef {
        DICE_PLUME_FX_SHADER.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        DICE_PLUME_FX_SHADER.into()
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType, Reflect)]
pub struct DicePlumeFxParams {
    pub time: f32,
    pub intensity: f32,
    pub kind: f32,
    pub color: Vec4,

    pub origin_ws: Vec3,
    pub _pad0: f32,
}

impl DicePlumeFxParams {
    pub fn set_color(&mut self, color: Color) {
        self.color = LinearRgba::from(color).to_vec4();
    }
}

pub type DiceSurfaceFxMaterial = ExtendedMaterial<StandardMaterial, DiceSurfaceFxExtension>;
pub type DicePlumeFxMaterial = ExtendedMaterial<StandardMaterial, DicePlumeFxExtension>;

#[derive(Resource, Default, Clone, Copy)]
pub struct DiceFxRollingTracker {
    pub was_rolling: bool,
}

#[derive(Resource, Clone)]
pub struct DiceFxMeshes {
    pub plume: Handle<Mesh>,
}

impl FromWorld for DiceFxMeshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        // Capsule gives a softer silhouette than a cylinder.
        let plume = meshes.add(bevy::math::primitives::Capsule3d::default());
        Self { plume }
    }
}

pub struct DiceFxPlugin;

/// Cached runtime-loaded textures for the user-defined custom effect.
#[derive(Resource, Default, Clone)]
pub struct CustomDiceFxTextures {
    pub noise_path: Option<String>,
    pub ramp_path: Option<String>,
    pub mask_path: Option<String>,

    pub noise: Option<Handle<Image>>,
    pub ramp: Option<Handle<Image>>,
    pub mask: Option<Handle<Image>>,
}

impl Plugin for DiceFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::pbr::MaterialPlugin::<DiceSurfaceFxMaterial>::default())
            .add_plugins(bevy::pbr::MaterialPlugin::<DicePlumeFxMaterial>::default())
            .add_message::<crate::dice3d::types::DiceRollCompletedEvent>()
            .init_resource::<DiceFxRollingTracker>()
            .init_resource::<DiceFxMeshes>()
            .init_resource::<CustomDiceFxTextures>()
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
                expire_custom_dice_fx.after(apply_dice_fx_from_roll_complete),
            )
            .add_systems(
                Update,
                sync_dice_fx_visuals.after(expire_custom_dice_fx),
            )
            .add_systems(
                Update,
                tick_dice_fx_material_time.after(sync_dice_fx_visuals),
            );
    }
}
