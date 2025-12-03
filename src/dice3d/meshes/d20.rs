use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;

pub fn create_d20() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.5;
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let s = size * 0.35;

    // Icosahedron has 12 vertices
    let vertices: Vec<Vec3> = vec![
        Vec3::new(0.0, 1.0, phi) * s,
        Vec3::new(0.0, -1.0, phi) * s,
        Vec3::new(0.0, 1.0, -phi) * s,
        Vec3::new(0.0, -1.0, -phi) * s,
        Vec3::new(1.0, phi, 0.0) * s,
        Vec3::new(-1.0, phi, 0.0) * s,
        Vec3::new(1.0, -phi, 0.0) * s,
        Vec3::new(-1.0, -phi, 0.0) * s,
        Vec3::new(phi, 0.0, 1.0) * s,
        Vec3::new(-phi, 0.0, 1.0) * s,
        Vec3::new(phi, 0.0, -1.0) * s,
        Vec3::new(-phi, 0.0, -1.0) * s,
    ];

    // 20 triangular faces (vertex indices)
    let faces: [[usize; 3]; 20] = [
        [0, 1, 8],
        [0, 8, 4],
        [0, 4, 5],
        [0, 5, 9],
        [0, 9, 1],
        [1, 6, 8],
        [8, 6, 10],
        [8, 10, 4],
        [4, 10, 2],
        [4, 2, 5],
        [5, 2, 11],
        [5, 11, 9],
        [9, 11, 7],
        [9, 7, 1],
        [1, 7, 6],
        [3, 6, 7],
        [3, 10, 6],
        [3, 2, 10],
        [3, 11, 2],
        [3, 7, 11],
    ];

    // Calculate face normals
    let face_normals: Vec<(Vec3, u32)> = faces
        .iter()
        .enumerate()
        .map(|(i, face)| {
            let v0 = vertices[face[0]];
            let v1 = vertices[face[1]];
            let v2 = vertices[face[2]];
            let center = (v0 + v1 + v2) / 3.0;
            let normal = center.normalize();
            (normal, (i + 1) as u32)
        })
        .collect();

    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size));
    let mesh = create_d20_mesh(&vertices, &faces);

    (mesh, collider, face_normals)
}

fn create_d20_mesh(vertices: &[Vec3], faces: &[[usize; 3]; 20]) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();

    for face in faces {
        let v0 = vertices[face[0]];
        let v1 = vertices[face[1]];
        let v2 = vertices[face[2]];

        // Calculate face normal
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();
        let n = normal.to_array();

        positions.push(v0.to_array());
        positions.push(v1.to_array());
        positions.push(v2.to_array());

        normals.push(n);
        normals.push(n);
        normals.push(n);
    }

    let num_vertices = positions.len();
    let indices: Vec<u32> = (0..num_vertices as u32).collect();
    let uvs: Vec<[f32; 2]> = positions.iter().map(|_| [0.5, 0.5]).collect();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}
