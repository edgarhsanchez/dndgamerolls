use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;

pub fn create_d12() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.5;
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let inv_phi = 1.0 / phi;

    // Dodecahedron vertices (scaled)
    let s = size * 0.4;
    let mut vertices = Vec::new();
    
    // Cube vertices
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                vertices.push(Vec3::new(sx * s, sy * s, sz * s));
            }
        }
    }
    
    // Rectangle vertices on each axis
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            vertices.push(Vec3::new(0.0, sx * phi * s, sy * inv_phi * s));
            vertices.push(Vec3::new(sx * inv_phi * s, 0.0, sy * phi * s));
            vertices.push(Vec3::new(sx * phi * s, sy * inv_phi * s, 0.0));
        }
    }

    // Face normals for the 12 pentagonal faces
    let face_normals: Vec<(Vec3, u32)> = vec![
        (Vec3::new(0.0, 1.0, 0.0), 1),          // top
        (Vec3::new(0.0, -1.0, 0.0), 12),        // bottom
        (Vec3::new(phi, inv_phi, 0.0).normalize(), 2),
        (Vec3::new(-phi, inv_phi, 0.0).normalize(), 3),
        (Vec3::new(phi, -inv_phi, 0.0).normalize(), 11),
        (Vec3::new(-phi, -inv_phi, 0.0).normalize(), 10),
        (Vec3::new(0.0, inv_phi, phi).normalize(), 4),
        (Vec3::new(0.0, inv_phi, -phi).normalize(), 5),
        (Vec3::new(0.0, -inv_phi, phi).normalize(), 9),
        (Vec3::new(0.0, -inv_phi, -phi).normalize(), 8),
        (Vec3::new(inv_phi, 0.0, phi).normalize(), 6),
        (Vec3::new(-inv_phi, 0.0, -phi).normalize(), 7),
    ];

    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size));
    let mesh = create_d12_mesh(size);

    (mesh, collider, face_normals)
}

fn create_d12_mesh(size: f32) -> Mesh {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let inv_phi = 1.0 / phi;
    let s = size * 0.4;

    // 20 vertices of dodecahedron
    let verts: Vec<[f32; 3]> = vec![
        // Cube vertices (8)
        [-s, -s, -s], [-s, -s, s], [-s, s, -s], [-s, s, s],
        [s, -s, -s], [s, -s, s], [s, s, -s], [s, s, s],
        // Axis-aligned rectangles (12)
        [0.0, -phi*s, -inv_phi*s], [0.0, -phi*s, inv_phi*s],
        [0.0, phi*s, -inv_phi*s], [0.0, phi*s, inv_phi*s],
        [-inv_phi*s, 0.0, -phi*s], [-inv_phi*s, 0.0, phi*s],
        [inv_phi*s, 0.0, -phi*s], [inv_phi*s, 0.0, phi*s],
        [-phi*s, -inv_phi*s, 0.0], [-phi*s, inv_phi*s, 0.0],
        [phi*s, -inv_phi*s, 0.0], [phi*s, inv_phi*s, 0.0],
    ];

    // Define the 12 pentagonal faces by vertex indices
    let faces: [[usize; 5]; 12] = [
        [11, 3, 17, 2, 10],   // top
        [9, 1, 16, 0, 8],     // bottom  
        [7, 19, 6, 14, 15],   // front-right
        [3, 13, 1, 9, 15],    // front-left
        [2, 17, 0, 12, 14],   // back
        [11, 10, 19, 7, 15],  // top-front
        [3, 11, 15, 9, 13],   // left-top
        [2, 14, 6, 10, 17],   // right-top
        [1, 13, 17, 16, 0],   // left-bottom
        [5, 9, 8, 18, 4],     // right-bottom
        [18, 8, 0, 4, 6],     // bottom-back
        [19, 5, 18, 6, 4],    // bottom-front
    ];

    let mut positions = Vec::new();
    let mut normals = Vec::new();

    for face in &faces {
        // Get center of pentagon for triangulation
        let center: Vec3 = face.iter()
            .map(|&i| Vec3::from_array(verts[i]))
            .sum::<Vec3>() / 5.0;
        let center_arr = center.to_array();
        
        // Calculate face normal
        let v0 = Vec3::from_array(verts[face[0]]);
        let v1 = Vec3::from_array(verts[face[1]]);
        let v2 = Vec3::from_array(verts[face[2]]);
        let normal = (v1 - v0).cross(v2 - v0).normalize();
        let n = normal.to_array();
        
        // Create 5 triangles from center to each edge
        for i in 0..5 {
            let next = (i + 1) % 5;
            positions.push(center_arr);
            positions.push(verts[face[i]]);
            positions.push(verts[face[next]]);
            for _ in 0..3 { normals.push(n); }
        }
    }

    let num_vertices = positions.len();
    let indices: Vec<u32> = (0..num_vertices as u32).collect();
    let uvs: Vec<[f32; 2]> = positions.iter().map(|_| [0.5, 0.5]).collect();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
