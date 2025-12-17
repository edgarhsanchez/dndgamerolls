//! Throw Control State
//!
//! Contains the resource for tracking mouse-controlled throw parameters.

use bevy::prelude::*;

/// The dice box boundaries in world space
/// The box is centered at origin with walls at these positions
// NOTE: These should match the geometry spawned in `dice3d::systems::setup`.
// Floor is a 4x4 cuboid centered at (0, -0.15, 0) with height 0.3, so the top surface is y=0.0.
pub const BOX_MIN_X: f32 = -2.0;
pub const BOX_MAX_X: f32 = 2.0;
pub const BOX_MIN_Z: f32 = -2.0;
pub const BOX_MAX_Z: f32 = 2.0;
pub const BOX_FLOOR_Y: f32 = 0.0;
pub const BOX_WALL_HEIGHT: f32 = 1.5;
pub const BOX_TOP_Y: f32 = BOX_FLOOR_Y + BOX_WALL_HEIGHT;
pub const BOX_CENTER: Vec3 = Vec3::new(0.0, 0.0, 0.0);

/// Resource for tracking mouse-controlled throw state
#[derive(Resource)]
pub struct ThrowControlState {
    /// The target point on the box floor (in world coordinates)
    pub target_point: Vec3,

    /// Current throw strength (0.0 to 1.0, based on mouse distance from center)
    pub throw_strength: f32,

    /// Maximum throw strength multiplier (controlled by slider)
    pub max_strength: f32,

    /// Minimum throw strength (always applies some force)
    pub min_strength: f32,

    /// Whether the mouse is currently over the dice box
    pub mouse_over_box: bool,
}

impl Default for ThrowControlState {
    fn default() -> Self {
        Self {
            target_point: Vec3::ZERO,
            throw_strength: 0.5,
            max_strength: 8.0,
            min_strength: 2.0,
            mouse_over_box: false,
        }
    }
}

impl ThrowControlState {
    /// Calculate the effective throw velocity based on current state
    /// Dice are thrown TOWARD the target point from the center
    pub fn calculate_throw_velocity(&self) -> Vec3 {
        let strength = self.min_strength + self.throw_strength * self.max_strength;

        // Direction from box center toward target point
        let direction = Vec3::new(
            self.target_point.x - BOX_CENTER.x,
            0.0,
            self.target_point.z - BOX_CENTER.z,
        );

        let dir = if direction.length() > 0.001 {
            direction.normalize()
        } else {
            // Default direction if mouse is at center
            Vec3::new(0.0, 0.0, -1.0)
        };

        Vec3::new(
            dir.x * strength,
            -0.3 * strength.min(3.0), // Slight downward component, capped
            dir.z * strength,
        )
    }

    /// Calculate angular velocity based on throw strength
    pub fn calculate_angular_velocity(&self, rng: &mut impl rand::Rng) -> Vec3 {
        let strength = self.min_strength + self.throw_strength * self.max_strength;
        let spin_factor = strength * 3.0;

        Vec3::new(
            rng.gen_range(-spin_factor..spin_factor),
            rng.gen_range(-spin_factor..spin_factor),
            rng.gen_range(-spin_factor..spin_factor),
        )
    }

    /// Check if a point is within the box boundaries (XZ plane)
    pub fn is_point_in_box(point: Vec3) -> bool {
        point.x >= BOX_MIN_X && point.x <= BOX_MAX_X && point.z >= BOX_MIN_Z && point.z <= BOX_MAX_Z
    }

    /// Clamp a world point to the nearest point on/inside the box floor
    pub fn clamp_to_box_floor(point: Vec3) -> Vec3 {
        Vec3::new(
            point.x.clamp(BOX_MIN_X, BOX_MAX_X),
            BOX_FLOOR_Y,
            point.z.clamp(BOX_MIN_Z, BOX_MAX_Z),
        )
    }

    /// Calculate strength based on distance from box center
    pub fn calculate_strength_from_distance(target: Vec3) -> f32 {
        let distance = Vec2::new(target.x, target.z).length();
        let max_distance = 2.0; // Half the box width
        (distance / max_distance).clamp(0.0, 1.0)
    }
}

/// Marker for the Material slider controlling throw strength
#[derive(Component)]
pub struct StrengthSlider;

/// Marker component for the 3D throw direction arrow
#[derive(Component)]
pub struct ThrowDirectionArrow;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_point_in_box() {
        assert!(ThrowControlState::is_point_in_box(Vec3::new(0.0, 0.0, 0.0)));
        assert!(ThrowControlState::is_point_in_box(Vec3::new(1.0, 0.0, 1.0)));
        assert!(!ThrowControlState::is_point_in_box(Vec3::new(
            2.1, 0.0, 0.0
        )));
    }

    #[test]
    fn test_clamp_to_box_floor() {
        let inside = Vec3::new(0.5, 0.0, 0.5);
        let clamped = ThrowControlState::clamp_to_box_floor(inside);
        assert_eq!(clamped.x, 0.5);
        assert_eq!(clamped.y, BOX_FLOOR_Y);
        assert_eq!(clamped.z, 0.5);

        let outside = Vec3::new(3.0, 0.0, -3.0);
        let clamped = ThrowControlState::clamp_to_box_floor(outside);
        assert_eq!(clamped.x, BOX_MAX_X);
        assert_eq!(clamped.z, BOX_MIN_Z);
    }
}
