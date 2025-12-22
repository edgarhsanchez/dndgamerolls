//! Auto-center container glTF models in the dice view.
//!
//! Some authored glTF scenes have their meshes offset from the intended origin.
//! This system computes the visual model bounds and applies a translation so the
//! model is centered (in X/Z) under the dice container root.

use bevy::prelude::Mesh3d;
use bevy::prelude::*;
use bevy_mesh::{Mesh, VertexAttributeValues};

use crate::dice3d::types::{
    DiceBox, DiceContainerCentered, DiceContainerColliderGuide, DiceContainerVisualRoot,
};

fn safe_inv(v: f32) -> f32 {
    if v.abs() > 0.000_001 {
        1.0 / v
    } else {
        1.0
    }
}

fn collect_mesh_descendants(
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<&Mesh3d, Without<DiceContainerColliderGuide>>,
    name_query: &Query<&Name>,
    out: &mut Vec<Entity>,
) {
    if mesh_query.get(entity).is_ok() {
        // Never use authored COLLIDER_* guide meshes for centering.
        if name_query
            .get(entity)
            .map(|n| n.as_str().starts_with("COLLIDER_"))
            .unwrap_or(false)
        {
            // no-op
        } else {
            out.push(entity);
        }
    }

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        collect_mesh_descendants(child, children_query, mesh_query, name_query, out);
    }
}

fn world_min_max_for_local_min_max(local_min: Vec3, local_max: Vec3, world_from_local: Mat4) -> (Vec3, Vec3) {
    let corners = [
        Vec3::new(local_min.x, local_min.y, local_min.z),
        Vec3::new(local_min.x, local_min.y, local_max.z),
        Vec3::new(local_min.x, local_max.y, local_min.z),
        Vec3::new(local_min.x, local_max.y, local_max.z),
        Vec3::new(local_max.x, local_min.y, local_min.z),
        Vec3::new(local_max.x, local_min.y, local_max.z),
        Vec3::new(local_max.x, local_max.y, local_min.z),
        Vec3::new(local_max.x, local_max.y, local_max.z),
    ];

    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for c in corners {
        let p = world_from_local.transform_point3(c);
        min = min.min(p);
        max = max.max(p);
    }

    (min, max)
}

/// Center (X/Z) the active container visuals under the dice container root.
///
/// This runs continuously until it has enough information (loaded meshes + transforms)
/// to compute bounds; then it applies the offset once per visual root.
pub fn center_container_models_in_view(
    mut commands: Commands,
    container_root_query: Query<(Entity, &GlobalTransform), With<DiceBox>>,
    mut visual_roots: Query<
        (Entity, &mut Transform),
        (With<DiceContainerVisualRoot>, Without<DiceContainerCentered>),
    >,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    mesh_query: Query<&Mesh3d, Without<DiceContainerColliderGuide>>,
    global_query: Query<&GlobalTransform>,
    meshes: Res<Assets<Mesh>>,
) {
    // We want to center relative to the container root; expect exactly one.
    let mut root_iter = container_root_query.iter();
    let Some((_container_entity, container_gt)) = root_iter.next() else {
        return;
    };
    if root_iter.next().is_some() {
        return;
    }

    let container_trs = container_gt.compute_transform();
    let container_translation = container_trs.translation;
    let container_rotation = container_trs.rotation;
    let container_scale = container_trs.scale;
    let inv_container_scale = Vec3::new(
        safe_inv(container_scale.x),
        safe_inv(container_scale.y),
        safe_inv(container_scale.z),
    );

    for (root_entity, mut root_local_transform) in &mut visual_roots {
        // We can only center once scene children exist.
        if children_query.get(root_entity).is_err() {
            continue;
        }

        let mut mesh_entities: Vec<Entity> = Vec::new();
        collect_mesh_descendants(
            root_entity,
            &children_query,
            &mesh_query,
            &name_query,
            &mut mesh_entities,
        );

        let mut min: Option<Vec3> = None;
        let mut max: Option<Vec3> = None;

        for mesh_entity in mesh_entities {
            let Ok(mesh_handle) = mesh_query.get(mesh_entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(&mesh_handle.0) else {
                continue;
            };
            let Ok(mesh_global) = global_query.get(mesh_entity) else {
                continue;
            };

            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                continue;
            };

            let mut local_min = Vec3::splat(f32::INFINITY);
            let mut local_max = Vec3::splat(f32::NEG_INFINITY);
            for p in positions {
                let v = Vec3::new(p[0], p[1], p[2]);
                local_min = local_min.min(v);
                local_max = local_max.max(v);
            }

            let (world_scale, world_rotation, world_translation) =
                mesh_global.to_scale_rotation_translation();
            let world_from_local =
                Mat4::from_scale_rotation_translation(world_scale, world_rotation, world_translation);
            let (mesh_min, mesh_max) =
                world_min_max_for_local_min_max(local_min, local_max, world_from_local);

            min = Some(match min {
                Some(v) => v.min(mesh_min),
                None => mesh_min,
            });
            max = Some(match max {
                Some(v) => v.max(mesh_max),
                None => mesh_max,
            });
        }

        let (Some(min), Some(max)) = (min, max) else {
            // Meshes not loaded yet; try again next frame.
            continue;
        };

        let center_world = (min + max) * 0.5;

        // Only center in X/Z; keep Y placement as-authored (floor alignment).
        let delta_world = Vec3::new(
            center_world.x - container_translation.x,
            0.0,
            center_world.z - container_translation.z,
        );

        // Convert the desired world-space delta into container-root local space.
        let delta_container_local = container_rotation
            .inverse()
            .mul_vec3(delta_world * inv_container_scale);

        // Shift the visual root so its center lands on the container origin.
        root_local_transform.translation -= delta_container_local;

        if let Ok(mut ec) = commands.get_entity(root_entity) {
            ec.insert(DiceContainerCentered);
        }
    }
}
