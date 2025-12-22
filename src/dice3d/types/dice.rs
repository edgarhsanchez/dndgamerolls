//! Dice-related types and components
//!
//! This module contains all types related to dice: DiceType, Die component,
//! DiceBox, DiceResults, DiceConfig, and RollState.

use bevy::prelude::*;

/// Component attached to each die entity
#[derive(Component)]
pub struct Die {
    pub die_type: DiceType,
    pub face_normals: Vec<(Vec3, u32)>,
}

/// Marker component for the dice box/container
#[derive(Component)]
pub struct DiceBox;

/// Marker component for the dice container floor collider.
#[derive(Component)]
pub struct DiceBoxFloorCollider;

/// Marker component for dice container wall segments.
#[derive(Component)]
pub struct DiceBoxWall;

/// Marker component for the dice container ceiling collider.
#[derive(Component)]
pub struct DiceBoxCeiling;

/// Marker component for the container's visual root entity (spawned via `SceneRoot`).
#[derive(Component)]
pub struct DiceContainerVisualRoot;

/// Marker component indicating a container visual root has been auto-centered.
///
/// This is used to ensure other systems (spawn points, colliders) only run once
/// the visual model has been moved into its final position.
#[derive(Component)]
pub struct DiceContainerCentered;

/// Marker component for any mesh/material entity that is part of the container visual.
#[derive(Component)]
pub struct DiceContainerVisualPart;

/// Marker component for glTF nodes whose name starts with `COLLIDER_`.
///
/// These meshes are authored in Blender as guides and should not be rendered.
#[derive(Component)]
pub struct DiceContainerColliderGuide;

/// Marker component for Rapier colliders generated from glTF collider guides.
#[derive(Component)]
pub struct DiceContainerGeneratedCollider;

/// Marker component for generated colliders that came from voxelizing the glTF render meshes.
#[derive(Component)]
pub struct DiceContainerVoxelCollider;

/// Marker for the legacy/procedural container colliders (floor/walls/ceiling).
///
/// When glTF collider guides are present and processed, these are despawned to avoid double-collisions.
#[derive(Component)]
pub struct DiceContainerProceduralCollider;

/// Marker component for container visual mesh entities that have had their material overridden
/// to the game's crystal material.
#[derive(Component)]
pub struct DiceContainerCrystalMaterialApplied;

/// Stores the original emissive color for a container visual part, so hover highlighting can be reverted.
#[derive(Component, Clone, Copy)]
pub struct ContainerOriginalEmissive(pub bevy::color::LinearRgba);

/// Visual/physics style for the dice container.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiceContainerStyle {
    #[default]
    Box,
    Cup,
}

/// Materials used for spawning the dice container visuals.
#[derive(Resource, Clone)]
pub struct DiceContainerMaterials {
    pub crystal: Handle<StandardMaterial>,
}

/// All supported dice types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
}

impl DiceType {
    pub fn max_value(&self) -> u32 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DiceType::D4 => "D4",
            DiceType::D6 => "D6",
            DiceType::D8 => "D8",
            DiceType::D10 => "D10",
            DiceType::D12 => "D12",
            DiceType::D20 => "D20",
        }
    }

    pub fn color(&self) -> Color {
        // Slightly translucent crystal-like colors
        match self {
            DiceType::D4 => Color::srgba(0.3, 0.4, 0.9, 0.92), // Blue crystal
            DiceType::D6 => Color::srgba(0.1, 0.1, 0.1, 0.95), // Black/smoke crystal
            DiceType::D8 => Color::srgba(0.6, 0.2, 0.8, 0.92), // Purple crystal
            DiceType::D10 => Color::srgba(0.95, 0.95, 0.95, 0.92), // White/clear crystal
            DiceType::D12 => Color::srgba(0.95, 0.5, 0.1, 0.92), // Orange crystal
            DiceType::D20 => Color::srgba(0.95, 0.85, 0.2, 0.92), // Yellow crystal
        }
    }

    pub fn parse(s: &str) -> Option<DiceType> {
        match s.to_lowercase().as_str() {
            "d4" => Some(DiceType::D4),
            "d6" => Some(DiceType::D6),
            "d8" => Some(DiceType::D8),
            "d10" => Some(DiceType::D10),
            "d12" => Some(DiceType::D12),
            "d20" => Some(DiceType::D20),
            _ => None,
        }
    }

    /// Get the physical density of the die for physics simulation.
    /// Larger dice are heavier, affecting how they roll and bounce.
    /// Density is based on realistic proportions where D20 is heaviest.
    pub fn density(&self) -> f32 {
        match self {
            DiceType::D4 => 1.0,  // Lightest - small tetrahedron
            DiceType::D6 => 1.5,  // Standard cube
            DiceType::D8 => 1.8,  // Octahedron
            DiceType::D10 => 2.0, // Medium
            DiceType::D12 => 2.5, // Larger dodecahedron
            DiceType::D20 => 3.0, // Heaviest - large icosahedron
        }
    }

    /// Get the scale factor for the die mesh.
    /// This affects both visual size and collision volume.
    pub fn scale(&self) -> f32 {
        match self {
            DiceType::D4 => 0.9,   // Smaller
            DiceType::D6 => 1.0,   // Standard
            DiceType::D8 => 1.0,   // Standard
            DiceType::D10 => 1.05, // Slightly larger
            DiceType::D12 => 1.1,  // Larger
            DiceType::D20 => 1.2,  // Largest
        }
    }

    /// Approximate base mesh radius (distance from origin to the furthest vertex)
    /// for each die mesh at `Transform::scale = 1.0`.
    ///
    /// This is used to normalize sizes so that equal user scale values produce
    /// equal visual sizes across different dice shapes.
    pub fn mesh_base_radius(&self) -> f32 {
        match self {
            // d4.rs uses edge length a=0.5, giving radius ~= 0.306186.
            DiceType::D4 => 0.306_186,
            // d6.rs uses Cuboid::new(0.6,0.6,0.6) => vertex radius = sqrt(3)*(0.6/2).
            DiceType::D6 => 0.519_615,
            // d8.rs uses vertices at ±0.5 on axes.
            DiceType::D8 => 0.5,
            // d10.rs top is at y=0.45.
            DiceType::D10 => 0.45,
            // d12.rs uses s=0.175; cube vertices are length sqrt(3)*s.
            DiceType::D12 => 0.303_108,
            // d20.rs uses s=0.175; vertices are length sqrt(1+phi^2)*s.
            DiceType::D20 => 0.332_870,
        }
    }

    /// Scale correction to normalize mesh sizes to a common reference.
    ///
    /// With this applied, two dice with the same user scale should appear
    /// roughly the same overall size.
    pub fn uniform_size_scale_factor(&self) -> f32 {
        // Most meshes use a `size = 0.5` convention; use that as the reference.
        const REFERENCE_RADIUS: f32 = 0.5;
        REFERENCE_RADIUS / self.mesh_base_radius()
    }
}

/// Resource storing the results of dice rolls
#[derive(Resource, Default)]
pub struct DiceResults {
    pub results: Vec<(DiceType, u32)>,
}

/// Resource tracking the current roll state
#[derive(Resource, Default)]
pub struct RollState {
    pub rolling: bool,
    pub settle_timer: f32,
    /// Timer tracking how long dice have been rolling (for timeout detection)
    pub roll_timer: f32,
}

/// Configuration for which dice to spawn
#[derive(Resource, Clone, Debug)]
pub struct DiceConfig {
    pub dice_to_roll: Vec<DiceType>,
    pub modifier: i32,
    pub modifier_name: String,
}

impl Default for DiceConfig {
    fn default() -> Self {
        Self {
            dice_to_roll: vec![DiceType::D20],
            modifier: 0,
            modifier_name: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dice_type_max_value() {
        assert_eq!(DiceType::D4.max_value(), 4);
        assert_eq!(DiceType::D6.max_value(), 6);
        assert_eq!(DiceType::D8.max_value(), 8);
        assert_eq!(DiceType::D10.max_value(), 10);
        assert_eq!(DiceType::D12.max_value(), 12);
        assert_eq!(DiceType::D20.max_value(), 20);
    }

    #[test]
    fn test_dice_type_name() {
        assert_eq!(DiceType::D4.name(), "D4");
        assert_eq!(DiceType::D6.name(), "D6");
        assert_eq!(DiceType::D20.name(), "D20");
    }

    #[test]
    fn test_dice_type_parse() {
        assert_eq!(DiceType::parse("d4"), Some(DiceType::D4));
        assert_eq!(DiceType::parse("D4"), Some(DiceType::D4));
        assert_eq!(DiceType::parse("d20"), Some(DiceType::D20));
        assert_eq!(DiceType::parse("D20"), Some(DiceType::D20));
        assert_eq!(DiceType::parse("invalid"), None);
        assert_eq!(DiceType::parse("d100"), None);
    }

    #[test]
    fn test_dice_config_default() {
        let config = DiceConfig::default();
        assert_eq!(config.dice_to_roll, vec![DiceType::D20]);
        assert_eq!(config.modifier, 0);
        assert!(config.modifier_name.is_empty());
    }

    #[test]
    fn test_dice_results_default() {
        let results = DiceResults::default();
        assert!(results.results.is_empty());
    }

    #[test]
    fn test_roll_state_default() {
        let state = RollState::default();
        assert!(!state.rolling);
        assert_eq!(state.settle_timer, 0.0);
    }

    #[test]
    fn test_dice_type_density() {
        // D4 should be lightest, D20 heaviest
        assert!(DiceType::D4.density() < DiceType::D6.density());
        assert!(DiceType::D6.density() < DiceType::D8.density());
        assert!(DiceType::D8.density() < DiceType::D10.density());
        assert!(DiceType::D10.density() < DiceType::D12.density());
        assert!(DiceType::D12.density() < DiceType::D20.density());

        // Check specific values
        assert_eq!(DiceType::D4.density(), 1.0);
        assert_eq!(DiceType::D20.density(), 3.0);
    }

    #[test]
    fn test_dice_type_scale() {
        // D4 should be smallest, D20 largest
        assert!(DiceType::D4.scale() <= DiceType::D6.scale());
        assert!(DiceType::D6.scale() <= DiceType::D10.scale());
        assert!(DiceType::D10.scale() <= DiceType::D12.scale());
        assert!(DiceType::D12.scale() <= DiceType::D20.scale());

        // D6 is the baseline
        assert_eq!(DiceType::D6.scale(), 1.0);
    }
}
