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
    }
}
