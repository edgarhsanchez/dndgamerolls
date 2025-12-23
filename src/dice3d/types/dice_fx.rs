use bevy::prelude::*;

use serde::{Deserialize, Serialize};

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
    pub effect: DiceFxEffectKind,
}

/// Which effect to play for a given die outcome.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiceFxEffectKind {
    #[default]
    None,
    Fire,
    Lightning,
    Firework,
    Explosion,
}

/// Marker for a fire particle effect attached to a die.
#[derive(Component)]
pub struct DiceFxFireEffect;

/// Marker for a lightning particle effect attached to a die.
#[derive(Component)]
pub struct DiceFxLightningEffect;

/// Marker for a firework particle effect attached to a die.
#[derive(Component)]
pub struct DiceFxFireworkEffect;

/// Marker for an explosion particle effect attached to a die.
#[derive(Component)]
pub struct DiceFxExplosionEffect;
