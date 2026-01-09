//! Rendering utilities for dice number labels
//!
//! This module contains functions for creating number meshes and positioning
//! labels on dice faces.

use bevy::prelude::*;

use crate::dice3d::types::DiceType;

/// Get the offset distance for number labels from the die center.
///
/// This should be the distance from the die's center to the face surface,
/// plus a tiny epsilon to avoid z-fighting. The label quad is then placed
/// at `normal * offset`.
pub fn get_label_offset(die_type: DiceType) -> f32 {
    // Small offset above the face surface to prevent z-fighting.
    const EPSILON: f32 = 0.005;

    match die_type {
        // D4 uses vertex normals for number positions; handled separately.
        DiceType::D4 => 0.15 + EPSILON,
        // D6 cube: half-extent = 0.3 (size 0.6)
        DiceType::D6 => 0.30 + EPSILON,
        // D8 octahedron: size=0.5, face distance = size/sqrt(3) ≈ 0.289
        DiceType::D8 => 0.29 + EPSILON,
        // D10: size=0.5, kite faces; approximate inscribed radius ~0.30
        DiceType::D10 => 0.30 + EPSILON,
        // D12: scale factor 0.35, phi-based; approximate face distance ~0.31
        DiceType::D12 => 0.31 + EPSILON,
        // D20: scale factor 0.35, phi-based; approximate face distance ~0.29
        DiceType::D20 => 0.29 + EPSILON,
    }
}

/// Get the scale factor for number labels based on die type
pub fn get_label_scale(die_type: DiceType) -> f32 {
    // Scale for number labels - clear and readable
    match die_type {
        DiceType::D4 => 0.10, // Small numbers for 3-per-face layout
        DiceType::D6 => 0.24,
        DiceType::D8 => 0.18,
        DiceType::D10 => 0.15,
        DiceType::D12 => 0.09,
        DiceType::D20 => 0.11,
    }
}

/// Calculate the rotation for a label to face outward from a die face.
///
/// The quad mesh is in the XY plane facing +Z. After rotation:
/// - The quad's +Z axis aligns with `normal` (facing outward).
/// - The quad's +Y axis (text "up") points toward world +Y when possible,
///   or toward +Z for top/bottom faces so text reads correctly from the camera.
pub fn get_label_rotation(normal: Vec3) -> Quat {
    // Choose an "up" hint that keeps text upright from common viewing angles.
    let up_hint = if normal.y.abs() > 0.99 {
        // Top or bottom face: use +Z as the reference up so text reads
        // correctly when looking down at the die.
        if normal.y > 0.0 {
            Vec3::Z
        } else {
            Vec3::NEG_Z
        }
    } else {
        // Side faces: use world +Y so text stands upright.
        Vec3::Y
    };

    // Build a coordinate frame: forward = normal, up ≈ up_hint.
    let forward = normal.normalize();
    let right = up_hint.cross(forward).normalize_or_zero();
    // If up_hint was parallel to forward, fall back to a perpendicular.
    let right = if right.length_squared() < 0.001 {
        forward.any_orthonormal_vector()
    } else {
        right
    };
    let up = forward.cross(right).normalize();

    // The quad's local axes are: +X right, +Y up, +Z forward.
    // Build a rotation matrix from those columns.
    Quat::from_mat3(&Mat3::from_cols(right, up, forward))
}

/// Create a mesh handle for a number label
pub fn create_number_mesh(value: u32, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
    // Create a mesh representing the number using curved digit style
    meshes.add(create_digit_mesh(value))
}

/// Create a 3D mesh for a number value
pub fn create_digit_mesh(value: u32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    // Create 3D box geometry for numbers
    let (positions, indices) = generate_number_geometry(value);

    // Generate proper normals for 3D boxes
    // Each box has 6 faces with 4 vertices each = 24 vertices per segment
    // Normals: front(+Z), back(-Z), top(+Y), bottom(-Y), left(-X), right(+X)
    let mut normals = Vec::new();
    let verts_per_box = 24;
    let num_boxes = positions.len() / verts_per_box;

    for _ in 0..num_boxes {
        // Front face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, 0.0, 1.0]);
        }
        // Back face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, 0.0, -1.0]);
        }
        // Top face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, 1.0, 0.0]);
        }
        // Bottom face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, -1.0, 0.0]);
        }
        // Left face (4 verts)
        for _ in 0..4 {
            normals.push([-1.0, 0.0, 0.0]);
        }
        // Right face (4 verts)
        for _ in 0..4 {
            normals.push([1.0, 0.0, 0.0]);
        }
    }

    // Handle any remaining vertices (shouldn't happen but just in case)
    while normals.len() < positions.len() {
        normals.push([0.0, 0.0, 1.0]);
    }

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

/// Generate the vertex positions and indices for a multi-digit number
pub fn generate_number_geometry(value: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let digits: Vec<u32> = if value == 0 {
        vec![0]
    } else {
        let mut v = value;
        let mut d = Vec::new();
        while v > 0 {
            d.push(v % 10);
            v /= 10;
        }
        d.reverse();
        d
    };

    let num_digits = digits.len();
    let digit_width = 0.6;
    let spacing = 0.1;
    let total_width = num_digits as f32 * digit_width + (num_digits - 1) as f32 * spacing;
    let start_x = -total_width / 2.0 + digit_width / 2.0;

    for (i, &digit) in digits.iter().enumerate() {
        let offset_x = start_x + i as f32 * (digit_width + spacing);
        let base_idx = positions.len() as u32;

        let (digit_pos, digit_idx) = get_digit_geometry(digit, offset_x);

        for pos in digit_pos {
            positions.push(pos);
        }
        for idx in digit_idx {
            indices.push(base_idx + idx);
        }
    }

    (positions, indices)
}

/// Generate the geometry for a single digit at a given x offset
pub fn get_digit_geometry(digit: u32, offset_x: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    // Smooth curved digit representation using rounded segments
    let stroke_width = 0.12; // Thinner stroke for cleaner look
    let h = 0.5; // Half height
    let w = 0.35; // Half width
    let d = 0.02; // Very thin depth - flat on surface
    let curve_segments = 6; // Segments for curves

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Helper to add a rounded rectangle segment (pill shape)
    let add_segment = |positions: &mut Vec<[f32; 3]>,
                       indices: &mut Vec<u32>,
                       x1: f32,
                       y1: f32,
                       x2: f32,
                       y2: f32| {
        let base_idx = positions.len() as u32;
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        // Perpendicular direction for width
        let px = -dy / len * stroke_width / 2.0;
        let py = dx / len * stroke_width / 2.0;

        // Front face - quad along the segment
        positions.push([offset_x + x1 - px, y1 - py, d / 2.0]);
        positions.push([offset_x + x1 + px, y1 + py, d / 2.0]);
        positions.push([offset_x + x2 + px, y2 + py, d / 2.0]);
        positions.push([offset_x + x2 - px, y2 - py, d / 2.0]);
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);

        // Back face
        let base_idx = positions.len() as u32;
        positions.push([offset_x + x1 + px, y1 + py, -d / 2.0]);
        positions.push([offset_x + x1 - px, y1 - py, -d / 2.0]);
        positions.push([offset_x + x2 - px, y2 - py, -d / 2.0]);
        positions.push([offset_x + x2 + px, y2 + py, -d / 2.0]);
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
    };

    // Helper to add curved segment
    let add_curve = |positions: &mut Vec<[f32; 3]>,
                     indices: &mut Vec<u32>,
                     cx: f32,
                     cy: f32,
                     radius: f32,
                     start_angle: f32,
                     end_angle: f32| {
        for i in 0..curve_segments {
            let t1 = i as f32 / curve_segments as f32;
            let t2 = (i + 1) as f32 / curve_segments as f32;
            let a1 = start_angle + (end_angle - start_angle) * t1;
            let a2 = start_angle + (end_angle - start_angle) * t2;
            let x1 = cx + radius * a1.cos();
            let y1 = cy + radius * a1.sin();
            let x2 = cx + radius * a2.cos();
            let y2 = cy + radius * a2.sin();
            add_segment(positions, indices, x1, y1, x2, y2);
        }
    };

    let pi = std::f32::consts::PI;
    let half_pi = std::f32::consts::FRAC_PI_2;

    // Define digit paths using line segments and curves
    match digit {
        0 => {
            // Oval shape
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.5,
                w * 0.6,
                half_pi,
                pi + half_pi,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.5,
                w * 0.6,
                -half_pi,
                half_pi,
            );
            add_segment(
                &mut positions,
                &mut indices,
                -w * 0.6,
                h * 0.5,
                -w * 0.6,
                -h * 0.5,
            );
            add_segment(
                &mut positions,
                &mut indices,
                w * 0.6,
                h * 0.5,
                w * 0.6,
                -h * 0.5,
            );
        }
        1 => {
            // Simple vertical line with small top serif
            add_segment(&mut positions, &mut indices, 0.0, h, 0.0, -h);
            add_segment(&mut positions, &mut indices, -w * 0.3, h * 0.6, 0.0, h);
        }
        2 => {
            // Top curve, diagonal, bottom
            add_curve(&mut positions, &mut indices, 0.0, h * 0.5, w * 0.5, 0.0, pi);
            add_segment(&mut positions, &mut indices, w * 0.5, h * 0.5, -w * 0.6, -h);
            add_segment(&mut positions, &mut indices, -w * 0.6, -h, w * 0.6, -h);
        }
        3 => {
            // Two curves stacked
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.5,
                w * 0.5,
                -half_pi,
                pi,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.5,
                w * 0.5,
                -pi,
                half_pi,
            );
        }
        4 => {
            // Angled line with vertical
            add_segment(&mut positions, &mut indices, -w * 0.6, h, -w * 0.6, 0.0);
            add_segment(&mut positions, &mut indices, -w * 0.6, 0.0, w * 0.6, 0.0);
            add_segment(&mut positions, &mut indices, w * 0.4, h, w * 0.4, -h);
        }
        5 => {
            // Top, down, curve bottom
            add_segment(&mut positions, &mut indices, w * 0.5, h, -w * 0.5, h);
            add_segment(&mut positions, &mut indices, -w * 0.5, h, -w * 0.5, 0.0);
            add_segment(&mut positions, &mut indices, -w * 0.5, 0.0, w * 0.3, 0.0);
            add_curve(
                &mut positions,
                &mut indices,
                w * 0.1,
                -h * 0.5,
                w * 0.5,
                half_pi,
                -pi,
            );
        }
        6 => {
            // Top curve into full bottom circle
            add_curve(&mut positions, &mut indices, 0.0, h * 0.3, w * 0.5, 0.0, pi);
            add_segment(
                &mut positions,
                &mut indices,
                -w * 0.5,
                h * 0.3,
                -w * 0.5,
                -h * 0.3,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.4,
                w * 0.5,
                0.0,
                2.0 * pi,
            );
        }
        7 => {
            // Top line with diagonal
            add_segment(&mut positions, &mut indices, -w * 0.5, h, w * 0.5, h);
            add_segment(&mut positions, &mut indices, w * 0.5, h, -w * 0.2, -h);
        }
        8 => {
            // Two stacked circles
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.5,
                w * 0.4,
                0.0,
                2.0 * pi,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.45,
                w * 0.5,
                0.0,
                2.0 * pi,
            );
        }
        9 => {
            // Top circle with tail
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.4,
                w * 0.5,
                0.0,
                2.0 * pi,
            );
            add_segment(
                &mut positions,
                &mut indices,
                w * 0.5,
                h * 0.2,
                w * 0.5,
                -h * 0.3,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.3,
                w * 0.5,
                0.0,
                -pi,
            );
        }
        _ => {
            // Fallback: simple box
            add_segment(&mut positions, &mut indices, -w * 0.5, h, w * 0.5, h);
            add_segment(&mut positions, &mut indices, w * 0.5, h, w * 0.5, -h);
            add_segment(&mut positions, &mut indices, w * 0.5, -h, -w * 0.5, -h);
            add_segment(&mut positions, &mut indices, -w * 0.5, -h, -w * 0.5, h);
        }
    }

    (positions, indices)
}
