use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn create_d6() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.6;

    let face_normals = vec![
        (Vec3::Y, 6),
        (Vec3::NEG_Y, 1),
        (Vec3::X, 3),
        (Vec3::NEG_X, 4),
        (Vec3::Z, 2),
        (Vec3::NEG_Z, 5),
    ];

    let mesh = Mesh::from(Cuboid::new(size, size, size));
    let collider = Collider::cuboid(size / 2.0, size / 2.0, size / 2.0);

    (mesh, collider, face_normals)
}
