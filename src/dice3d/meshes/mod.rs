pub mod d4;
pub mod d6;
pub mod d8;
pub mod d10;
pub mod d12;
pub mod d20;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::dice3d::types::DiceType;

pub use d4::create_d4;
pub use d6::create_d6;
pub use d8::create_d8;
pub use d10::create_d10;
pub use d12::create_d12;
pub use d20::create_d20;

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
