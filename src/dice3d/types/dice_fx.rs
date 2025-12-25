use bevy::prelude::*;

use super::DiceType;

/// Fired when a dice roll has fully settled and face values have been determined.
#[derive(Message, Clone, Debug, Default)]
pub struct DiceRollCompletedEvent {
    pub results: Vec<DieRollOutcome>,
}

#[derive(Clone, Copy, Debug)]
pub struct DieRollOutcome {
    pub entity: Entity,
    pub die_type: DiceType,
    pub value: u32,
}

/// Stores the last settled roll value for a die.
#[derive(Component, Clone, Copy, Debug)]
pub struct DieLastRoll {
    pub value: u32,
}

/// Which special effects should be shown for a die.
///
/// These flags map directly to the shader params in `dice3d::dice_fx`.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DiceFxState {
    pub fire: bool,
    pub electric: bool,

    pub fireworks: bool,
    pub explosion: bool,

    /// Time (seconds since startup) when the current FX instance began.
    pub started_at: f32,
    /// Duration seconds before FX auto-expires. Use <= 0 to disable expiry.
    pub duration: f32,
}

/// Marker for the surface FX shell mesh attached to a die.
#[derive(Component)]
pub struct DiceFxSurfaceShell;

/// Marker for the fire plume mesh attached to a die.
#[derive(Component)]
pub struct DiceFxFirePlume;

/// Marker for the atomic plume mesh attached to a die.
#[derive(Component)]

pub struct DiceFxAtomicPlume;

/// Component to store a material handle for FX child entities.
#[derive(Component, Clone, Debug)]
pub struct DiceFxMaterialHandle<M: Material>(pub Handle<M>);
