use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;

pub fn create_d8() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.5;

    let vertices = vec![
        Vec3::new(0.0, size, 0.0),
        Vec3::new(0.0, -size, 0.0),
        Vec3::new(size, 0.0, 0.0),
        Vec3::new(-size, 0.0, 0.0),
        Vec3::new(0.0, 0.0, size),
        Vec3::new(0.0, 0.0, -size),
    ];

    let n = 0.577;
    let face_normals = vec![
        (Vec3::new(n, n, n), 1),
        (Vec3::new(-n, n, n), 2),
        (Vec3::new(n, n, -n), 3),
        (Vec3::new(-n, n, -n), 4),
        (Vec3::new(n, -n, n), 8),
        (Vec3::new(-n, -n, n), 7),
        (Vec3::new(n, -n, -n), 6),
        (Vec3::new(-n, -n, -n), 5),
    ];

    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size));
    let mesh = create_octahedron_mesh(size);

    (mesh, collider, face_normals)
}

fn create_octahedron_mesh(size: f32) -> Mesh {
    let positions = vec![
        [0.0, size, 0.0], [size, 0.0, 0.0], [0.0, 0.0, size],
        [0.0, size, 0.0], [0.0, 0.0, size], [-size, 0.0, 0.0],
        [0.0, size, 0.0], [-size, 0.0, 0.0], [0.0, 0.0, -size],
        [0.0, size, 0.0], [0.0, 0.0, -size], [size, 0.0, 0.0],
        [0.0, -size, 0.0], [0.0, 0.0, size], [size, 0.0, 0.0],
        [0.0, -size, 0.0], [-size, 0.0, 0.0], [0.0, 0.0, size],
        [0.0, -size, 0.0], [0.0, 0.0, -size], [-size, 0.0, 0.0],
        [0.0, -size, 0.0], [size, 0.0, 0.0], [0.0, 0.0, -size],
    ];

    let n = 0.577_f32;
    let normals = vec![
        [n, n, n], [n, n, n], [n, n, n],
        [-n, n, n], [-n, n, n], [-n, n, n],
        [-n, n, -n], [-n, n, -n], [-n, n, -n],
        [n, n, -n], [n, n, -n], [n, n, -n],
        [n, -n, n], [n, -n, n], [n, -n, n],
        [-n, -n, n], [-n, -n, n], [-n, -n, n],
        [-n, -n, -n], [-n, -n, -n], [-n, -n, -n],
        [n, -n, -n], [n, -n, -n], [n, -n, -n],
    ];

    let uvs: Vec<[f32; 2]> = (0..24).map(|_| [0.5, 0.5]).collect();
    let indices: Vec<u32> = (0..24).collect();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
