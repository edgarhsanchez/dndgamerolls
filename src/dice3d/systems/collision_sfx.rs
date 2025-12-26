//! Plays positional collision sound effects when dice hit the active container.

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;

use crate::dice3d::embedded_assets::{DICE_GLASS_CUP_SFX_PATH, DICE_WOODEN_BOX_SFX_PATH};
use crate::dice3d::types::{
    DiceContainerProceduralCollider, DiceContainerStyle, DiceContainerVoxelCollider, Die,
};

#[derive(Resource, Clone)]
pub struct DiceCollisionSfx {
    pub cup: Handle<AudioSource>,
    pub box_: Handle<AudioSource>,
}

/// Debounce state to avoid spamming collision SFX when Rapier reports multiple contact events
/// for the same die in quick succession.
#[derive(Resource, Default)]
pub struct DiceCollisionSfxDebounce {
    /// Last playback time (seconds since startup) per die entity.
    pub last_played_s: HashMap<Entity, f32>,
    /// Minimum interval between plays per die.
    pub min_interval_s: f32,
}

pub fn init_collision_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cup = asset_server.load(DICE_GLASS_CUP_SFX_PATH);
    let box_ = asset_server.load(DICE_WOODEN_BOX_SFX_PATH);
    commands.insert_resource(DiceCollisionSfx { cup, box_ });
    commands.insert_resource(DiceCollisionSfxDebounce {
        // "A little bit" of debounce: enough to stop contact spam, short enough to still feel responsive.
        min_interval_s: 0.10,
        ..Default::default()
    });
}

pub fn play_dice_container_collision_sfx(
    mut commands: Commands,
    style: Res<DiceContainerStyle>,
    sfx: Res<DiceCollisionSfx>,
    mut debounce: ResMut<DiceCollisionSfxDebounce>,
    mut collision_events: MessageReader<CollisionEvent>,
    dice_query: Query<(), With<Die>>,
    container_query: Query<
        (),
        Or<(
            With<DiceContainerVoxelCollider>,
            With<DiceContainerProceduralCollider>,
        )>,
    >,
    die_velocity: Query<&Velocity, With<Die>>,
    global_transforms: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    let now_s = time.elapsed_secs();

    for ev in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _flags) = *ev else {
            continue;
        };

        let die1 = dice_query.get(e1).is_ok();
        let die2 = dice_query.get(e2).is_ok();
        let cont1 = container_query.get(e1).is_ok();
        let cont2 = container_query.get(e2).is_ok();

        // We play "roll" SFX for:
        // - Die vs container (box/cup)
        // - Die vs die (common during rolls)
        let (primary_die, other_die, _pos) = if die1 && cont2 {
            let Ok(die_gt) = global_transforms.get(e1) else {
                continue;
            };
            (e1, None, die_gt.translation())
        } else if die2 && cont1 {
            let Ok(die_gt) = global_transforms.get(e2) else {
                continue;
            };
            (e2, None, die_gt.translation())
        } else if die1 && die2 {
            // To avoid double-playing for the same die-die contact event, pick a stable "primary" die.
            let primary = if e1.index() <= e2.index() { e1 } else { e2 };
            let other = if primary == e1 { e2 } else { e1 };

            let Ok(gt1) = global_transforms.get(e1) else {
                continue;
            };
            let Ok(gt2) = global_transforms.get(e2) else {
                continue;
            };

            (
                primary,
                Some(other),
                (gt1.translation() + gt2.translation()) * 0.5,
            )
        } else {
            continue;
        };

        if let Some(last_s) = debounce.last_played_s.get(&primary_die) {
            if now_s - *last_s < debounce.min_interval_s {
                continue;
            }
        }

        let (sound, variant_gain, _variant_name) = match *style {
            // The wooden box sample tends to read quieter than the glass cup sample.
            DiceContainerStyle::Box => (sfx.box_.clone(), 2.2_f32, "box"),
            DiceContainerStyle::Cup => (sfx.cup.clone(), 1.6_f32, "cup"),
        };

        debounce.last_played_s.insert(primary_die, now_s);

        // Approximate collision "strength" from the die's current velocities.
        // This is cheaper than force/impulse events and works well for audio scaling.
        let strength_for = |e: Entity| {
            die_velocity
                .get(e)
                .map(|v| v.linvel.length() + 0.15 * v.angvel.length())
                .unwrap_or(0.0)
        };

        let mut strength = strength_for(primary_die);
        if let Some(other) = other_die {
            strength += strength_for(other);
        }

        // Map strength -> volume. Keep a small floor so quiet collisions are still audible.
        // Tunables:
        // - `strength_ref`: roughly the velocity magnitude that should sound "full volume".
        // - `min_volume`: audible floor.
        // Slightly lower reference so typical die-die contacts are audible.
        let strength_ref = 3.6_f32;
        let min_volume = 0.08_f32;
        let max_volume = 1.0_f32;
        let t = (strength / strength_ref).clamp(0.0, 1.0);
        let volume = min_volume + (max_volume - min_volume) * t.powf(0.7);

        // Global gain bump: collision SFX are easy to end up too quiet on some Windows setups.
        // Keep a clamp to avoid clipping when collisions are strong.
        let volume = (volume * variant_gain).clamp(0.0, 1.0);

        #[cfg(debug_assertions)]
        {
            // Helps diagnose cases where the style is unexpectedly Cup (or vice versa).
            debug!(
                "collision_sfx: variant={} die={:?} other_die={:?} strength={:.2} vol={:.2}",
                _variant_name, primary_die, other_die, strength, volume
            );
        }

        // Non-spatial: collision SFX were easy to miss when the camera/listener is far
        // from the container (attenuation can make them effectively silent).
        commands.spawn((
            AudioPlayer(sound),
            PlaybackSettings::DESPAWN
                .with_spatial(false)
                .with_volume(Volume::Linear(volume)),
        ));
    }
}
