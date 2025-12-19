//! Plays positional collision sound effects when dice hit the active container.

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::dice3d::embedded_assets::{
    DICE_GLASS_CUP_SFX_PATH, DICE_WOODEN_BOX_SFX_PATH,
};
use crate::dice3d::types::{DiceContainerProceduralCollider, DiceContainerStyle, DiceContainerVoxelCollider, Die};

#[derive(Resource, Clone)]
pub struct DiceCollisionSfx {
    pub cup: Handle<AudioSource>,
    pub box_: Handle<AudioSource>,
}

pub fn init_collision_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cup = asset_server.load(DICE_GLASS_CUP_SFX_PATH);
    let box_ = asset_server.load(DICE_WOODEN_BOX_SFX_PATH);
    commands.insert_resource(DiceCollisionSfx { cup, box_ });
}

pub fn play_dice_container_collision_sfx(
    mut commands: Commands,
    style: Res<DiceContainerStyle>,
    sfx: Res<DiceCollisionSfx>,
    mut collision_events: MessageReader<CollisionEvent>,
    dice_query: Query<(), With<Die>>,
    container_query: Query<(), Or<(With<DiceContainerVoxelCollider>, With<DiceContainerProceduralCollider>)>>,
    global_transforms: Query<&GlobalTransform>,
) {
    for ev in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _flags) = *ev else {
            continue;
        };

        let (die_entity, _container_entity) = if dice_query.get(e1).is_ok() && container_query.get(e2).is_ok() {
            (e1, e2)
        } else if dice_query.get(e2).is_ok() && container_query.get(e1).is_ok() {
            (e2, e1)
        } else {
            continue;
        };

        // Emit from the die's world position (good approximation of the impact location).
        let Ok(die_gt) = global_transforms.get(die_entity) else {
            continue;
        };
        let pos = die_gt.translation();

        let sound = match *style {
            DiceContainerStyle::Box => sfx.box_.clone(),
            DiceContainerStyle::Cup => sfx.cup.clone(),
        };

        commands.spawn((
            AudioPlayer(sound),
            PlaybackSettings::DESPAWN.with_spatial(true),
            Transform::from_translation(pos),
            GlobalTransform::default(),
        ));
    }
}
