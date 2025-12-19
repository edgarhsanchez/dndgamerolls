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
    }
}
