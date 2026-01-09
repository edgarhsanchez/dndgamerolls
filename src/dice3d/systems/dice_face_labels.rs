use std::collections::HashMap;

use bevy::{image::ImageLoaderSettings, prelude::*};

use crate::dice3d::embedded_assets::{
    DICE_NUMBER_ATLAS_COLOR_PATH, DICE_NUMBER_ATLAS_DEPTH_PATH, DICE_NUMBER_ATLAS_NORMAL_PATH,
};
use crate::dice3d::types::DiceFaceLabelAssets;

const ATLAS_COLS: u32 = 5;
const ATLAS_ROWS: u32 = 5;
const CELL_SIZE: f32 = 128.0;
const ATLAS_SIZE: f32 = ATLAS_COLS as f32 * CELL_SIZE;

pub fn init_dice_face_label_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Color map is sRGB; normal & depth must be linear.
    let color = asset_server.load(DICE_NUMBER_ATLAS_COLOR_PATH);

    let normal = asset_server.load_with_settings(
        DICE_NUMBER_ATLAS_NORMAL_PATH,
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );

    let depth = asset_server.load_with_settings(
        DICE_NUMBER_ATLAS_DEPTH_PATH,
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(color),
        normal_map_texture: Some(normal),
        depth_map: Some(depth),
        // Texture supplies the visible color.
        base_color: Color::WHITE,
        // Use Blend so transparent areas of the atlas show the die color beneath.
        alpha_mode: AlphaMode::Blend,
        // Parallax settings for engraved number effect.
        // Smaller scale = subtler depth; too high causes floating/distortion.
        parallax_depth_scale: 0.015,
        parallax_mapping_method: ParallaxMappingMethod::Occlusion,
        max_parallax_layer_count: 8.0,
        perceptual_roughness: 0.35,
        reflectance: 0.25,
        // Make the label slightly emissive so numbers are visible in shadow.
        emissive: LinearRgba::new(0.15, 0.15, 0.15, 1.0),
        ..default()
    });

    let mut meshes_by_value = HashMap::new();
    for value in 0u32..=20u32 {
        let mesh = make_number_quad_mesh(value);
        let handle = meshes.add(mesh);
        meshes_by_value.insert(value, handle);
    }

    commands.insert_resource(DiceFaceLabelAssets {
        material,
        meshes_by_value,
    });
}

fn make_number_quad_mesh(value: u32) -> Mesh {
    let (u_min, v_min, u_max, v_max) = atlas_uv_rect(value);

    // A unit quad in the XY plane, facing +Z.
    let positions: Vec<[f32; 3]> = vec![
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];

    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; 4];

    let uvs: Vec<[f32; 2]> = vec![
        [u_min, v_min],
        [u_max, v_min],
        [u_max, v_max],
        [u_min, v_max],
    ];

    let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];

    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
    // Required for normal/depth maps.
    .with_generated_tangents()
    .expect("generate tangents")
}

fn atlas_uv_rect(value: u32) -> (f32, f32, f32, f32) {
    let idx = value.min(ATLAS_COLS * ATLAS_ROWS - 1);
    let col = idx % ATLAS_COLS;
    let row = idx / ATLAS_COLS;

    let px0 = col as f32 * CELL_SIZE;
    let py0 = row as f32 * CELL_SIZE;

    // Inset by 1 px to avoid bleeding.
    let inset = 1.0;

    let u0 = (px0 + inset) / ATLAS_SIZE;
    let u1 = (px0 + CELL_SIZE - inset) / ATLAS_SIZE;

    // Bevy UV space uses v=1 at the top.
    let v_top = 1.0 - (py0 + inset) / ATLAS_SIZE;
    let v_bottom = 1.0 - (py0 + CELL_SIZE - inset) / ATLAS_SIZE;

    (u0, v_bottom, u1, v_top)
}
