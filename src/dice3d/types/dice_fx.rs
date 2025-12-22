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
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DiceFxState {
    pub fire: bool,
    pub atomic: bool,
    pub electric: bool,

    /// User-defined custom effect (if configured).
    #[allow(dead_code)]
    pub custom: bool,
    #[allow(dead_code)]
    pub custom_started_at: f32,
    #[allow(dead_code)]
    pub custom_duration: f32,
}

/// Marker for the surface FX shell mesh attached to a die.
#[derive(Component)]
pub struct DiceFxSurfaceShell;

/// Marker for a fire plume attached to a die.
#[derive(Component)]
pub struct DiceFxFirePlume;

/// Marker for an "atomic" plume attached to a die.
#[derive(Component)]
pub struct DiceFxAtomicPlume;

/// Stores the material handle used by a dice FX entity, so we can animate it.
#[derive(Component, Clone, Debug)]
pub struct DiceFxMaterialHandle<T: Asset>(pub Handle<T>);
