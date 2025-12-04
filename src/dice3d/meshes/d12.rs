use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;

pub fn create_d12() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.5;
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio ~1.618
    
    // Scale factor for the dodecahedron
    let s = size * 0.35;
    
    // 20 vertices of a regular dodecahedron
    // Using the standard construction with cube vertices and rectangle vertices
    let vertices: Vec<Vec3> = vec![
        // Cube vertices (±1, ±1, ±1) scaled
        Vec3::new(-1.0, -1.0, -1.0) * s,  // 0
        Vec3::new(-1.0, -1.0,  1.0) * s,  // 1
        Vec3::new(-1.0,  1.0, -1.0) * s,  // 2
        Vec3::new(-1.0,  1.0,  1.0) * s,  // 3
        Vec3::new( 1.0, -1.0, -1.0) * s,  // 4
        Vec3::new( 1.0, -1.0,  1.0) * s,  // 5
        Vec3::new( 1.0,  1.0, -1.0) * s,  // 6
        Vec3::new( 1.0,  1.0,  1.0) * s,  // 7
        // Rectangle vertices (0, ±1/φ, ±φ)
        Vec3::new(0.0, -1.0/phi, -phi) * s,  // 8
        Vec3::new(0.0, -1.0/phi,  phi) * s,  // 9
        Vec3::new(0.0,  1.0/phi, -phi) * s,  // 10
        Vec3::new(0.0,  1.0/phi,  phi) * s,  // 11
        // Rectangle vertices (±1/φ, ±φ, 0)
        Vec3::new(-1.0/phi, -phi, 0.0) * s,  // 12
        Vec3::new(-1.0/phi,  phi, 0.0) * s,  // 13
        Vec3::new( 1.0/phi, -phi, 0.0) * s,  // 14
        Vec3::new( 1.0/phi,  phi, 0.0) * s,  // 15
        // Rectangle vertices (±φ, 0, ±1/φ)
        Vec3::new(-phi, 0.0, -1.0/phi) * s,  // 16
        Vec3::new(-phi, 0.0,  1.0/phi) * s,  // 17
        Vec3::new( phi, 0.0, -1.0/phi) * s,  // 18
        Vec3::new( phi, 0.0,  1.0/phi) * s,  // 19
    ];

    // 12 pentagonal faces - each face lists 5 vertex indices in order (CCW winding)
    let faces: [[usize; 5]; 12] = [
        [0, 8, 10, 2, 16],   // Face 1
        [0, 16, 17, 1, 12],  // Face 2
        [0, 12, 14, 4, 8],   // Face 3
        [1, 17, 3, 11, 9],   // Face 4
        [1, 9, 5, 14, 12],   // Face 5
        [2, 10, 6, 15, 13],  // Face 6
        [2, 13, 3, 17, 16],  // Face 7
        [3, 13, 15, 7, 11],  // Face 8
        [4, 14, 5, 19, 18],  // Face 9
        [4, 18, 6, 10, 8],   // Face 10
        [5, 9, 11, 7, 19],   // Face 11
        [6, 18, 19, 7, 15],  // Face 12
    ];

    // Face values - standard D12 arrangement (opposite faces sum to 13)
    let face_values: [u32; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    // Calculate actual face centers for each pentagonal face
    let mut face_normals: Vec<(Vec3, u32)> = Vec::new();
    for (i, face) in faces.iter().enumerate() {
        // Calculate center of pentagon (average of 5 vertices)
        let center: Vec3 = face.iter().map(|&idx| vertices[idx]).sum::<Vec3>() / 5.0;
        // Use normalized center as direction for label placement
        face_normals.push((center.normalize(), face_values[i]));
    }

    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size));
    let mesh = create_d12_mesh(&vertices, &faces);

    (mesh, collider, face_normals)
}

fn create_d12_mesh(vertices: &[Vec3], faces: &[[usize; 5]; 12]) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();

    for face in faces {
        // Get center of pentagon for triangulation
        let center: Vec3 = face
            .iter()
            .map(|&i| vertices[i])
            .sum::<Vec3>()
            / 5.0;
        let center_arr = center.to_array();

        // Calculate face normal using first two edges
        let v0 = vertices[face[0]];
        let v1 = vertices[face[1]];
        let v2 = vertices[face[2]];
        let edge1 = v1 - v0;
        let edge2 = v2 - v1;
        let normal = edge1.cross(edge2).normalize();
        
        // Make sure normal points outward (away from center)
        let to_center = -center.normalize();
        let normal = if normal.dot(to_center) > 0.0 { -normal } else { normal };
        let n = normal.to_array();

        // Create 5 triangles from center to each edge
        for i in 0..5 {
            let next = (i + 1) % 5;
            positions.push(center_arr);
            positions.push(vertices[face[i]].to_array());
            positions.push(vertices[face[next]].to_array());
            for _ in 0..3 {
                normals.push(n);
            }
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
