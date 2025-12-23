use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::prelude::*;
use std::path::{Path, PathBuf};

pub const CUP_MODEL_ASSET_PATH: &str = concat!(env!("CARGO_PKG_NAME"), "/models/cup.glb");
pub const CUP_MODEL_SCENE_PATH: &str = concat!(
    "embedded://",
    env!("CARGO_PKG_NAME"),
    "/models/cup.glb#Scene0"
);

pub const BOX_MODEL_ASSET_PATH: &str = concat!(env!("CARGO_PKG_NAME"), "/models/box.glb");
pub const BOX_MODEL_GLTF_PATH: &str = concat!(
    "embedded://",
    env!("CARGO_PKG_NAME"),
    "/models/box.glb"
);
pub const BOX_MODEL_SCENE_PATH: &str = concat!(
    "embedded://",
    env!("CARGO_PKG_NAME"),
    "/models/box.glb#Scene0"
);

pub const DICE_GLASS_CUP_SFX_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/sounds/dice_glass_cup.mp3");
pub const DICE_GLASS_CUP_SFX_PATH: &str = concat!(
    "embedded://",
    env!("CARGO_PKG_NAME"),
    "/sounds/dice_glass_cup.mp3"
);

pub const DICE_WOODEN_BOX_SFX_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/sounds/dice_wooden_box.mp3");
pub const DICE_WOODEN_BOX_SFX_PATH: &str = concat!(
    "embedded://",
    env!("CARGO_PKG_NAME"),
    "/sounds/dice_wooden_box.mp3"
);

// -----------------------------------------------------------------------------
// Dice FX SFX (embedded)
// -----------------------------------------------------------------------------

pub const DICE_FX_ELECTRICITY_SFX_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/sounds/fx_electricity.mp3");
pub const DICE_FX_ELECTRICITY_SFX_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/sounds/fx_electricity.mp3");

pub const DICE_FX_EXPLOSION_SFX_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/sounds/fx_explosion.mp3");
pub const DICE_FX_EXPLOSION_SFX_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/sounds/fx_explosion.mp3");

pub const DICE_FX_FIREWORKS_SFX_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/sounds/fx_fireworks.mp3");
pub const DICE_FX_FIREWORKS_SFX_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/sounds/fx_fireworks.mp3");

// -----------------------------------------------------------------------------
// Dice FX textures (embedded)
// -----------------------------------------------------------------------------

pub const DICE_FX_FIRE_NOISE_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/fire/noise.png");
pub const DICE_FX_FIRE_NOISE_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/fire/noise.png");

pub const DICE_FX_FIRE_RAMP_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/fire/ramp.png");
pub const DICE_FX_FIRE_RAMP_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/fire/ramp.png");

pub const DICE_FX_FIRE_MASK_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/fire/mask.png");
pub const DICE_FX_FIRE_MASK_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/fire/mask.png");

pub const DICE_FX_ATOMIC_NOISE_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/atomic/noise.png");
pub const DICE_FX_ATOMIC_NOISE_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/atomic/noise.png");

pub const DICE_FX_ATOMIC_RAMP_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/atomic/ramp.png");
pub const DICE_FX_ATOMIC_RAMP_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/atomic/ramp.png");

pub const DICE_FX_ATOMIC_MASK_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/atomic/mask.png");
pub const DICE_FX_ATOMIC_MASK_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/atomic/mask.png");

pub const DICE_FX_ELECTRIC_NOISE_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/electric/noise.png");
pub const DICE_FX_ELECTRIC_NOISE_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/electric/noise.png");

pub const DICE_FX_ELECTRIC_RAMP_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/electric/ramp.png");
pub const DICE_FX_ELECTRIC_RAMP_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/electric/ramp.png");

pub const DICE_FX_ELECTRIC_MASK_ASSET_PATH: &str =
    concat!(env!("CARGO_PKG_NAME"), "/fx/electric/mask.png");
pub const DICE_FX_ELECTRIC_MASK_PATH: &str =
    concat!("embedded://", env!("CARGO_PKG_NAME"), "/fx/electric/mask.png");

pub struct Dice3dEmbeddedAssetsPlugin;

impl Plugin for Dice3dEmbeddedAssetsPlugin {
    fn build(&self, app: &mut App) {
        // Ensure the embedded registry exists (it should already be present with DefaultPlugins,
        // but init'ing it is harmless).
        app.init_resource::<EmbeddedAssetRegistry>();

        // Register the cup model as an embedded asset so releases don't need to ship external files.
        let registry = app.world_mut().resource_mut::<EmbeddedAssetRegistry>();

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("models/cup.glb");
        registry.insert_asset(
            PathBuf::from("3d/cup.glb"),
            &asset_path,
            include_bytes!("../../3d/cup.glb"),
        );

        // Register the box model as an embedded asset.
        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("models/box.glb");
        registry.insert_asset(
            PathBuf::from("3d/box.glb"),
            &asset_path,
            include_bytes!("../../3d/box.glb"),
        );

        // Register collision SFX as embedded assets.
        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("sounds/dice_glass_cup.mp3");
        registry.insert_asset(
            PathBuf::from("assets/sounds/dice_glass_cup.mp3"),
            &asset_path,
            include_bytes!("../../assets/sounds/dice_glass_cup.mp3"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("sounds/dice_wooden_box.mp3");
        registry.insert_asset(
            PathBuf::from("assets/sounds/dice_wooden_box.mp3"),
            &asset_path,
            include_bytes!("../../assets/sounds/dice_wooden_box.mp3"),
        );

        // Register Dice FX SFX as embedded assets.
        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("sounds/fx_electricity.mp3");
        registry.insert_asset(
            PathBuf::from("assets/sounds/fx_electricity.mp3"),
            &asset_path,
            include_bytes!("../../assets/sounds/fx_electricity.mp3"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("sounds/fx_explosion.mp3");
        registry.insert_asset(
            PathBuf::from("assets/sounds/fx_explosion.mp3"),
            &asset_path,
            include_bytes!("../../assets/sounds/fx_explosion.mp3"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("sounds/fx_fireworks.mp3");
        registry.insert_asset(
            PathBuf::from("assets/sounds/fx_fireworks.mp3"),
            &asset_path,
            include_bytes!("../../assets/sounds/fx_fireworks.mp3"),
        );

        // -----------------------------------------------------------------
        // Dice FX textures (placeholders can be replaced by user assets).
        // -----------------------------------------------------------------
        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/fire/noise.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/fire/noise.png"),
            &asset_path,
            include_bytes!("../../assets/fx/fire/noise.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/fire/ramp.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/fire/ramp.png"),
            &asset_path,
            include_bytes!("../../assets/fx/fire/ramp.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/fire/mask.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/fire/mask.png"),
            &asset_path,
            include_bytes!("../../assets/fx/fire/mask.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/atomic/noise.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/atomic/noise.png"),
            &asset_path,
            include_bytes!("../../assets/fx/atomic/noise.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/atomic/ramp.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/atomic/ramp.png"),
            &asset_path,
            include_bytes!("../../assets/fx/atomic/ramp.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/atomic/mask.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/atomic/mask.png"),
            &asset_path,
            include_bytes!("../../assets/fx/atomic/mask.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/electric/noise.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/electric/noise.png"),
            &asset_path,
            include_bytes!("../../assets/fx/electric/noise.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/electric/ramp.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/electric/ramp.png"),
            &asset_path,
            include_bytes!("../../assets/fx/electric/ramp.png"),
        );

        let asset_path = Path::new(env!("CARGO_PKG_NAME")).join("fx/electric/mask.png");
        registry.insert_asset(
            PathBuf::from("assets/fx/electric/mask.png"),
            &asset_path,
            include_bytes!("../../assets/fx/electric/mask.png"),
        );
    }
}
