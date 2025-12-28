//! Collect dice spawn points from named glTF nodes (typically Blender empties).

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use rand::Rng;

use crate::dice3d::types::{DiceBox, DiceContainerStyle, Die};

/// Resource holding spawn point world positions extracted from glTF scenes.
#[derive(Resource, Default, Debug, Clone)]
pub struct DiceSpawnPoints {
    pub box_points: Vec<Vec3>,
    pub cup_points: Vec<Vec3>,
}

/// Marker to ensure each spawn point node is only processed once.
#[derive(Component)]
pub struct DiceSpawnPointProcessed;

/// Tracks whether we have already applied spawn points for the current container style.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct DiceSpawnPointsApplied {
    pub box_applied: bool,
    pub cup_applied: bool,
}

fn collect_spawn_point_nodes(
    entity: Entity,
    children_query: &Query<&Children>,
    name_query: &Query<&Name>,
    out: &mut Vec<Entity>,
) {
    if let Ok(name) = name_query.get(entity) {
        let n = name.as_str();
        if n.starts_with("SPAWN_BOX")
            || n.starts_with("SPAWN_CUP")
            || n.starts_with("DICE_SPAWN_BOX")
            || n.starts_with("DICE_SPAWN_CUP")
        {
            out.push(entity);
        }
    }

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        collect_spawn_point_nodes(child, children_query, name_query, out);
    }
}

/// Collect `SPAWN_BOX*` / `SPAWN_CUP*` nodes under any container root and store their world positions.
///
/// Intended for Blender empties exported in the glTF scene.
pub fn collect_dice_spawn_points_from_gltf(
    mut commands: Commands,
    container_roots: Query<
        Entity,
        (
            With<crate::dice3d::types::DiceContainerVisualRoot>,
            With<crate::dice3d::types::DiceContainerCentered>,
        ),
    >,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    global_query: Query<&GlobalTransform>,
    processed: Query<(), With<DiceSpawnPointProcessed>>,
    mut spawn_points: ResMut<DiceSpawnPoints>,
) {
    for root in &container_roots {
        let mut nodes: Vec<Entity> = Vec::new();
        collect_spawn_point_nodes(root, &children_query, &name_query, &mut nodes);

        for entity in nodes {
            if processed.get(entity).is_ok() {
                continue;
            }

            let Ok(name) = name_query.get(entity) else {
                continue;
            };

            let Ok(global) = global_query.get(entity) else {
                continue;
            };

            let pos = global.translation();
            let n = name.as_str();
            if n.starts_with("SPAWN_BOX") || n.starts_with("DICE_SPAWN_BOX") {
                spawn_points.box_points.push(pos);
            } else if n.starts_with("SPAWN_CUP") || n.starts_with("DICE_SPAWN_CUP") {
                spawn_points.cup_points.push(pos);
            }

            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.insert(DiceSpawnPointProcessed);
            }
        }
    }

    // Keep the order stable so indexing is deterministic.
    spawn_points
        .box_points
        .sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    spawn_points
        .cup_points
        .sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
}

/// If spawn points exist for the current container style, re-place dice once per style.
///
/// This compensates for asynchronous glTF scene spawning: dice may spawn before the nodes exist.
pub fn apply_spawn_points_to_dice_when_ready(
    style: Res<DiceContainerStyle>,
    spawn_points: Res<DiceSpawnPoints>,
    mut applied: ResMut<DiceSpawnPointsApplied>,
    mut dice_query: Query<(&mut Transform, &mut Velocity), With<Die>>,
    _container_query: Query<(), With<DiceBox>>,
) {
    let (points, already_applied) = match *style {
        DiceContainerStyle::Box => (&spawn_points.box_points, applied.box_applied),
        DiceContainerStyle::Cup => (&spawn_points.cup_points, applied.cup_applied),
    };

    if already_applied || points.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    for (i, (mut transform, mut velocity)) in dice_query.iter_mut().enumerate() {
        let p = points[i % points.len()] + Vec3::Y * 0.15;
        transform.translation = p;
        velocity.linvel = Vec3::new(
            rng.random_range(-0.5..0.5),
            rng.random_range(-0.5..0.0),
            rng.random_range(-0.5..0.5),
        );
        velocity.angvel = Vec3::new(
            rng.random_range(-1.5..1.5),
            rng.random_range(-1.5..1.5),
            rng.random_range(-1.5..1.5),
        );
    }

    match *style {
        DiceContainerStyle::Box => applied.box_applied = true,
        DiceContainerStyle::Cup => applied.cup_applied = true,
    }
}
