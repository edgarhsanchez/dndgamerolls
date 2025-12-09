pub mod d10;
pub mod d12;
pub mod d20;
pub mod d4;
pub mod d6;
pub mod d8;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::dice3d::types::DiceType;

pub use d10::create_d10;
pub use d12::create_d12;
pub use d20::create_d20;
pub use d4::{create_d4, get_d4_number_positions};
pub use d6::create_d6;
pub use d8::create_d8;

pub fn create_die_mesh_and_collider(die_type: DiceType) -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    match die_type {
        DiceType::D4 => create_d4(),
        DiceType::D6 => create_d6(),
        DiceType::D8 => create_d8(),
        DiceType::D10 => create_d10(),
        DiceType::D12 => create_d12(),
        DiceType::D20 => create_d20(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d4_has_4_faces() {
        let (_, _, face_normals) = create_d4();
        assert_eq!(face_normals.len(), 4, "D4 should have 4 face normals");
        // Check all values are in range 1-4
        for (_, value) in &face_normals {
            assert!(*value >= 1 && *value <= 4, "D4 face values should be 1-4");
        }
    }

    #[test]
    fn test_d6_has_6_faces() {
        let (_, _, face_normals) = create_d6();
        assert_eq!(face_normals.len(), 6, "D6 should have 6 face normals");
        for (_, value) in &face_normals {
            assert!(*value >= 1 && *value <= 6, "D6 face values should be 1-6");
        }
    }

    #[test]
    fn test_d8_has_8_faces() {
        let (_, _, face_normals) = create_d8();
        assert_eq!(face_normals.len(), 8, "D8 should have 8 face normals");
        for (_, value) in &face_normals {
            assert!(*value >= 1 && *value <= 8, "D8 face values should be 1-8");
        }
    }

    #[test]
    fn test_d10_has_10_faces() {
        let (_, _, face_normals) = create_d10();
        assert_eq!(face_normals.len(), 10, "D10 should have 10 face normals");
        for (_, value) in &face_normals {
            assert!(
                *value >= 1 && *value <= 10,
                "D10 face values should be 1-10"
            );
        }
    }

    #[test]
    fn test_d12_has_12_faces() {
        let (_, _, face_normals) = create_d12();
        assert_eq!(face_normals.len(), 12, "D12 should have 12 face normals");
        for (_, value) in &face_normals {
            assert!(
                *value >= 1 && *value <= 12,
                "D12 face values should be 1-12"
            );
        }
    }

    #[test]
    fn test_d20_has_20_faces() {
        let (_, _, face_normals) = create_d20();
        assert_eq!(face_normals.len(), 20, "D20 should have 20 face normals");
        for (_, value) in &face_normals {
            assert!(
                *value >= 1 && *value <= 20,
                "D20 face values should be 1-20"
            );
        }
    }

    #[test]
    fn test_create_die_mesh_and_collider() {
        // Test that all dice types can be created
        let (_, _, d4_faces) = create_die_mesh_and_collider(DiceType::D4);
        let (_, _, d6_faces) = create_die_mesh_and_collider(DiceType::D6);
        let (_, _, d8_faces) = create_die_mesh_and_collider(DiceType::D8);
        let (_, _, d10_faces) = create_die_mesh_and_collider(DiceType::D10);
        let (_, _, d12_faces) = create_die_mesh_and_collider(DiceType::D12);
        let (_, _, d20_faces) = create_die_mesh_and_collider(DiceType::D20);

        assert_eq!(d4_faces.len(), 4);
        assert_eq!(d6_faces.len(), 6);
        assert_eq!(d8_faces.len(), 8);
        assert_eq!(d10_faces.len(), 10);
        assert_eq!(d12_faces.len(), 12);
        assert_eq!(d20_faces.len(), 20);
    }
}
