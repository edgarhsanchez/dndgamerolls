use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::*;

/// Creates a D4 using the modern design where each face has 3 numbers.
/// When the die lands, you read the number at the TOP of each visible face.
/// All three visible faces show the same number at the top.
pub fn create_d4() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.5; // Smallest die

    // Regular tetrahedron geometry
    let a = size; // edge length
    let h = a * (2.0_f32 / 3.0_f32).sqrt(); // height

    // Place base at y = -h/4, apex at y = 3h/4 (center of mass at origin)
    let base_y = -h / 4.0;
    let apex_y = 3.0 * h / 4.0;
    let base_r = a / (3.0_f32).sqrt(); // radius of base circumcircle

    // Vertices - 4 vertices of tetrahedron
    let v0 = Vec3::new(0.0, apex_y, 0.0); // top apex
    let v1 = Vec3::new(0.0, base_y, base_r); // front base
    let v2 = Vec3::new(base_r * 0.866, base_y, -base_r * 0.5); // right back base
    let v3 = Vec3::new(-base_r * 0.866, base_y, -base_r * 0.5); // left back base

    let vertices = vec![v0, v1, v2, v3];

    // Modern D4 numbering - each face has 3 numbers, one near each vertex
    // The number at the TOP of each visible face is the result
    //
    // When die lands on a face, that face is down, 3 faces are visible
    // The top edges of those 3 faces all show the same number
    //
    // Face layout (vertices):
    // - Face A: v1, v2, v3 (base) - opposite to v0
    // - Face B: v0, v1, v2 - opposite to v3
    // - Face C: v0, v2, v3 - opposite to v1
    // - Face D: v0, v3, v1 - opposite to v2
    //
    // When landing on Face A (base down, v0 up): result = 1
    // When landing on Face B (v3 up): result = 2
    // When landing on Face C (v1 up): result = 3
    // When landing on Face D (v2 up): result = 4

    // For result detection, we check which vertex points UP
    // The face_normals here represent the direction to check for each result
    let face_normals = vec![
        (v0.normalize(), 1), // v0 pointing up = result 1
        (v3.normalize(), 2), // v3 pointing up = result 2
        (v1.normalize(), 3), // v1 pointing up = result 3
        (v2.normalize(), 4), // v2 pointing up = result 4
    ];

    // Create mesh
    let mesh_n_base = (v2 - v1).cross(v3 - v1).normalize();
    let mesh_n_front = (v1 - v0).cross(v2 - v0).normalize();
    let mesh_n_right = (v2 - v0).cross(v3 - v0).normalize();
    let mesh_n_left = (v3 - v0).cross(v1 - v0).normalize();

    let positions: Vec<[f32; 3]> = vec![
        // Base face (v1, v2, v3)
        v1.into(),
        v2.into(),
        v3.into(),
        // Front face (v0, v1, v2)
        v0.into(),
        v1.into(),
        v2.into(),
        // Right-back face (v0, v2, v3)
        v0.into(),
        v2.into(),
        v3.into(),
        // Left-back face (v0, v3, v1)
        v0.into(),
        v3.into(),
        v1.into(),
    ];

    let normals: Vec<[f32; 3]> = vec![
        mesh_n_base.into(),
        mesh_n_base.into(),
        mesh_n_base.into(),
        mesh_n_front.into(),
        mesh_n_front.into(),
        mesh_n_front.into(),
        mesh_n_right.into(),
        mesh_n_right.into(),
        mesh_n_right.into(),
        mesh_n_left.into(),
        mesh_n_left.into(),
        mesh_n_left.into(),
    ];

    let uvs: Vec<[f32; 2]> = vec![
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
    ];

    let indices: Vec<u32> = (0..12).collect();

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices));

    // Create collider from the exact same vertices
    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size / 2.0));

    (mesh, collider, face_normals)
}

/// Returns the positions, rotations, and values for all numbers on the D4.
/// Each face has 3 numbers - returns (position, rotation, value) for each.
/// Each number's TOP points toward its nearest vertex/corner.
pub fn get_d4_number_positions() -> Vec<(Vec3, Quat, u32)> {
    let size = 0.5;
    let a = size;
    let h = a * (2.0_f32 / 3.0_f32).sqrt();
    let base_y = -h / 4.0;
    let apex_y = 3.0 * h / 4.0;
    let base_r = a / (3.0_f32).sqrt();

    let v0 = Vec3::new(0.0, apex_y, 0.0); // apex
    let v1 = Vec3::new(0.0, base_y, base_r); // front base
    let v2 = Vec3::new(base_r * 0.866, base_y, -base_r * 0.5); // right back base
    let v3 = Vec3::new(-base_r * 0.866, base_y, -base_r * 0.5); // left back base

    // Face centers and normals
    let face_a_center = (v1 + v2 + v3) / 3.0; // base (bottom)
    let face_b_center = (v0 + v1 + v2) / 3.0; // front
    let face_c_center = (v0 + v2 + v3) / 3.0; // right-back
    let face_d_center = (v0 + v3 + v1) / 3.0; // left-back

    let n_a = (v2 - v1).cross(v3 - v1).normalize(); // base normal (points down)
    let n_b = (v1 - v0).cross(v2 - v0).normalize();
    let n_c = (v2 - v0).cross(v3 - v0).normalize();
    let n_d = (v3 - v0).cross(v1 - v0).normalize();

    let label_offset_from_center = 0.07; // How far from face center toward vertex
    let surface_offset = 0.008; // Distance above the face surface (increased from ~0.003)

    let mut positions = Vec::new();

    // Helper to calculate rotation for a number on a face
    // The TOP of the number should point toward target_vertex
    // We build a rotation matrix from orthonormal basis vectors to avoid flipping issues
    let calc_rotation =
        |face_normal: Vec3, _face_center: Vec3, number_pos: Vec3, target_vertex: Vec3| -> Quat {
            // Z axis = face normal (pointing outward from face)
            let z_axis = face_normal;

            // Y axis = direction toward target vertex, projected onto face plane
            let to_vertex = target_vertex - number_pos;
            let y_axis = (to_vertex - face_normal * to_vertex.dot(face_normal)).normalize();

            // X axis = Y cross Z (right-hand rule)
            let x_axis = y_axis.cross(z_axis).normalize();

            // Build rotation from basis vectors
            // The matrix columns are the basis vectors
            Quat::from_mat3(&bevy::math::Mat3::from_cols(x_axis, y_axis, z_axis))
        };

    // Face A (base: v1, v2, v3) - bottom face, looking down
    // Numbers positioned near each vertex, tops pointing outward to that vertex
    // This face needs numbers flipped horizontally (180 degree rotation around Y)
    let pos_a1 = face_a_center
        + (v1 - face_a_center).normalize() * label_offset_from_center
        + n_a * surface_offset;
    let pos_a2 = face_a_center
        + (v2 - face_a_center).normalize() * label_offset_from_center
        + n_a * surface_offset;
    let pos_a3 = face_a_center
        + (v3 - face_a_center).normalize() * label_offset_from_center
        + n_a * surface_offset;

    let flip_y = Quat::from_rotation_y(std::f32::consts::PI);
    let rot_a1 = calc_rotation(n_a, face_a_center, pos_a1, v1) * flip_y;
    let rot_a2 = calc_rotation(n_a, face_a_center, pos_a2, v2) * flip_y;
    let rot_a3 = calc_rotation(n_a, face_a_center, pos_a3, v3) * flip_y;

    positions.push((pos_a1, rot_a1, 3)); // v1 up = 3
    positions.push((pos_a2, rot_a2, 4)); // v2 up = 4
    positions.push((pos_a3, rot_a3, 2)); // v3 up = 2

    // Face B (v0, v1, v2) - front face
    let pos_b0 = face_b_center
        + (v0 - face_b_center).normalize() * label_offset_from_center
        + n_b * surface_offset;
    let pos_b1 = face_b_center
        + (v1 - face_b_center).normalize() * label_offset_from_center
        + n_b * surface_offset;
    let pos_b2 = face_b_center
        + (v2 - face_b_center).normalize() * label_offset_from_center
        + n_b * surface_offset;

    let rot_b0 = calc_rotation(n_b, face_b_center, pos_b0, v0);
    let rot_b1 = calc_rotation(n_b, face_b_center, pos_b1, v1);
    let rot_b2 = calc_rotation(n_b, face_b_center, pos_b2, v2);

    positions.push((pos_b0, rot_b0, 1)); // v0 up = 1
    positions.push((pos_b1, rot_b1, 3)); // v1 up = 3
    positions.push((pos_b2, rot_b2, 4)); // v2 up = 4

    // Face C (v0, v2, v3) - right-back face
    let pos_c0 = face_c_center
        + (v0 - face_c_center).normalize() * label_offset_from_center
        + n_c * surface_offset;
    let pos_c2 = face_c_center
        + (v2 - face_c_center).normalize() * label_offset_from_center
        + n_c * surface_offset;
    let pos_c3 = face_c_center
        + (v3 - face_c_center).normalize() * label_offset_from_center
        + n_c * surface_offset;

    let rot_c0 = calc_rotation(n_c, face_c_center, pos_c0, v0);
    let rot_c2 = calc_rotation(n_c, face_c_center, pos_c2, v2);
    let rot_c3 = calc_rotation(n_c, face_c_center, pos_c3, v3);

    positions.push((pos_c0, rot_c0, 1)); // v0 up = 1
    positions.push((pos_c2, rot_c2, 4)); // v2 up = 4
    positions.push((pos_c3, rot_c3, 2)); // v3 up = 2

    // Face D (v0, v3, v1) - left-back face
    let pos_d0 = face_d_center
        + (v0 - face_d_center).normalize() * label_offset_from_center
        + n_d * surface_offset;
    let pos_d3 = face_d_center
        + (v3 - face_d_center).normalize() * label_offset_from_center
        + n_d * surface_offset;
    let pos_d1 = face_d_center
        + (v1 - face_d_center).normalize() * label_offset_from_center
        + n_d * surface_offset;

    let rot_d0 = calc_rotation(n_d, face_d_center, pos_d0, v0);
    let rot_d3 = calc_rotation(n_d, face_d_center, pos_d3, v3);
    let rot_d1 = calc_rotation(n_d, face_d_center, pos_d1, v1);

    positions.push((pos_d0, rot_d0, 1)); // v0 up = 1
    positions.push((pos_d3, rot_d3, 2)); // v3 up = 2
    positions.push((pos_d1, rot_d1, 3)); // v1 up = 3

    positions
}
