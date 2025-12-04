use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;

pub fn create_d10() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.5;
    let angle = std::f32::consts::PI / 5.0;

    // D10 is a pentagonal trapezohedron (10 kite-shaped faces)
    let mut vertices = Vec::new();

    // Top and bottom points
    let top = Vec3::new(0.0, size * 0.9, 0.0);
    let bottom = Vec3::new(0.0, -size * 0.9, 0.0);
    vertices.push(top);
    vertices.push(bottom);

    // Upper ring (5 vertices)
    let mut upper_ring = Vec::new();
    for i in 0..5 {
        let a = i as f32 * angle * 2.0;
        let v = Vec3::new(
            a.cos() * size * 0.7,
            size * 0.3,
            a.sin() * size * 0.7,
        );
        upper_ring.push(v);
        vertices.push(v);
    }

    // Lower ring (5 vertices, offset by half angle)
    let mut lower_ring = Vec::new();
    for i in 0..5 {
        let a = (i as f32 + 0.5) * angle * 2.0;
        let v = Vec3::new(
            a.cos() * size * 0.7,
            -size * 0.3,
            a.sin() * size * 0.7,
        );
        lower_ring.push(v);
        vertices.push(v);
    }

    // Calculate actual face normals for the 10 kite-shaped faces
    // The face normal is the direction from center pointing outward through the face center
    let mut face_normals: Vec<(Vec3, u32)> = Vec::new();
    
    for i in 0..5 {
        let next = (i + 1) % 5;
        
        // Upper face: top, upper[i], lower[i], upper[next]
        // Face center is the average of the 4 vertices
        let upper_face_center = (top + upper_ring[i] + lower_ring[i] + upper_ring[next]) / 4.0;
        // Use normalized face center as the direction for label placement
        face_normals.push((upper_face_center.normalize(), (i * 2 + 1) as u32));
        
        // Lower face: bottom, lower[next], upper[next], lower[i]
        let lower_face_center = (bottom + lower_ring[next] + upper_ring[next] + lower_ring[i]) / 4.0;
        face_normals.push((lower_face_center.normalize(), (i * 2 + 2) as u32));
    }

    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size));
    let mesh = create_d10_mesh(size);

    (mesh, collider, face_normals)
}

fn create_d10_mesh(size: f32) -> Mesh {
    let angle = std::f32::consts::PI / 5.0;

    let top = [0.0, size * 0.9, 0.0];
    let bottom = [0.0, -size * 0.9, 0.0];

    // Upper ring vertices
    let mut upper_ring = Vec::new();
    for i in 0..5 {
        let a = i as f32 * angle * 2.0;
        upper_ring.push([a.cos() * size * 0.7, size * 0.3, a.sin() * size * 0.7]);
    }

    // Lower ring vertices (offset by half angle)
    let mut lower_ring = Vec::new();
    for i in 0..5 {
        let a = (i as f32 + 0.5) * angle * 2.0;
        lower_ring.push([a.cos() * size * 0.7, -size * 0.3, a.sin() * size * 0.7]);
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();

    // Create 10 kite-shaped faces
    for i in 0..5 {
        let next = (i + 1) % 5;

        // Upper face: top -> upper[i] -> lower[i] -> upper[next]
        positions.push(top);
        positions.push(upper_ring[i]);
        positions.push(lower_ring[i]);

        positions.push(top);
        positions.push(lower_ring[i]);
        positions.push(upper_ring[next]);

        let v1 = Vec3::from_array(upper_ring[i]) - Vec3::from_array(top);
        let v2 = Vec3::from_array(lower_ring[i]) - Vec3::from_array(top);
        let n = v1.cross(v2).normalize();
        for _ in 0..6 {
            normals.push(n.to_array());
        }

        // Lower face: bottom -> lower[next] -> upper[next] -> lower[i]
        positions.push(bottom);
        positions.push(lower_ring[next]);
        positions.push(upper_ring[next]);

        positions.push(bottom);
        positions.push(upper_ring[next]);
        positions.push(lower_ring[i]);

        let v1 = Vec3::from_array(lower_ring[next]) - Vec3::from_array(bottom);
        let v2 = Vec3::from_array(upper_ring[next]) - Vec3::from_array(bottom);
        let n = v1.cross(v2).normalize();
        for _ in 0..6 {
            normals.push(n.to_array());
        }
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
