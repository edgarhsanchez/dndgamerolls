use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn create_d4() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.8;
    let h = size * (2.0_f32 / 3.0).sqrt();

    let vertices = vec![
        Vec3::new(0.0, h, 0.0),
        Vec3::new(-size / 2.0, 0.0, size * 0.577),
        Vec3::new(size / 2.0, 0.0, size * 0.577),
        Vec3::new(0.0, 0.0, -size * 0.577),
    ];

    let face_normals = vec![
        (Vec3::new(0.0, -1.0, 0.0), 1),
        (Vec3::new(0.0, 0.333, 0.943), 2),
        (Vec3::new(0.816, 0.333, -0.471), 3),
        (Vec3::new(-0.816, 0.333, -0.471), 4),
    ];

    let mesh = Mesh::from(Tetrahedron::default()).scaled_by(Vec3::splat(size));
    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size / 2.0));

    (mesh, collider, face_normals)
}
