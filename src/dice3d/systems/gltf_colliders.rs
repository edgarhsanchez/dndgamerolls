//! Spawn Rapier colliders for the dice container.
//!
//! The container visuals are loaded from glTF scenes. For collisions we generate
//! voxelized-mesh colliders (Rapier `SharedShape::voxelized_mesh`) from the render
//! meshes once they load.
//!
//! Any authored `COLLIDER_*` helper nodes are always hidden and ignored for physics.
use bevy::log::{info, warn};
use bevy::prelude::*;
use bevy::prelude::Mesh3d;
use bevy_mesh::{Indices, VertexAttributeValues};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::FillMode;
use bevy_rapier3d::rapier::prelude::SharedShape;

use std::collections::HashSet;

use crate::dice3d::types::{
    DiceBox, DiceBoxWall, DiceContainerColliderGuide, DiceContainerCrystalMaterialApplied,
    DiceContainerGeneratedCollider, DiceContainerMaterials, DiceContainerProceduralCollider,
    DiceContainerVisualRoot, DiceContainerVoxelCollider,
};

fn abs_vec3(v: Vec3) -> Vec3 {
    Vec3::new(v.x.abs(), v.y.abs(), v.z.abs())
}

fn collect_mesh_descendants(
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<&Mesh3d>,
    name_query: &Query<&Name>,
    out: &mut Vec<Entity>,
) {
    if mesh_query.get(entity).is_ok() {
        // Skip authored COLLIDER_* guide meshes; those are never part of collisions in voxel mode.
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

fn collect_collider_guides(
    entity: Entity,
    children_query: &Query<&Children>,
    name_query: &Query<&Name>,
    out: &mut Vec<Entity>,
) {
    if let Ok(name) = name_query.get(entity) {
        if name.as_str().starts_with("COLLIDER_") {
            out.push(entity);
        }
    }

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        collect_collider_guides(child, children_query, name_query, out);
    }
}

#[derive(Default)]
pub struct ColliderGuideDebugState {
    logged_start: bool,
    logged_no_root: bool,
    logged_multiple_roots: bool,
    logged_no_guides_for_root: HashSet<Entity>,
    logged_mesh_wait_for_voxel: HashSet<Entity>,
    procedural_removed: bool,
    last_status: Option<(usize, usize, usize)>,
    last_roots: HashSet<Entity>,
}

/// Generate voxelized-mesh colliders for the container glTF scenes.
pub fn spawn_colliders_from_gltf_guides(
    mut commands: Commands,
    container_root: Query<(Entity, &GlobalTransform), With<DiceBox>>,
    roots: Query<Entity, With<DiceContainerVisualRoot>>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    global_query: Query<&GlobalTransform>,
    mesh_query: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    voxel_colliders: Query<Entity, With<DiceContainerVoxelCollider>>,
    procedural_colliders: Query<Entity, With<DiceContainerProceduralCollider>>,
    mut debug: Local<ColliderGuideDebugState>,
) {
    const VOXEL_SIZE: f32 = 0.2;
    let mut root_iter = container_root.iter();
    let Some((container_root_entity, container_root_gt)) = root_iter.next() else {
        if !debug.logged_no_root {
            info!("glTF colliders: waiting for DiceBox root entity");
            debug.logged_no_root = true;
        }
        return;
    };
    if root_iter.next().is_some() {
        if !debug.logged_multiple_roots {
            warn!("glTF colliders: expected exactly 1 DiceBox root; found multiple; skipping");
            debug.logged_multiple_roots = true;
        }
        return;
    }

    if !debug.logged_start {
        info!(
            "glTF colliders: active (voxel_size={:.2}, container_root={:?}, visual_roots={})",
            VOXEL_SIZE,
            container_root_entity,
            roots.iter().count()
        );
        debug.logged_start = true;
    }

    // If the set of active visual roots changes (container style toggle), drop old voxel colliders.
    // Procedural colliders are respawned by the toggle system and will remain until voxel colliders are ready.
    let current_roots: HashSet<Entity> = roots.iter().collect();
    if debug.last_roots != current_roots {
        for entity in &voxel_colliders {
            commands.entity(entity).despawn();
        }
        debug.procedural_removed = false;
        debug.logged_mesh_wait_for_voxel.clear();
        debug.last_roots = current_roots;
    }

    let container_root_transform = container_root_gt.compute_transform();
    let root_rotation = container_root_transform.rotation;
    let root_translation = container_root_transform.translation;
    let root_scale = abs_vec3(container_root_transform.scale);
    let inv_root_scale = Vec3::new(
        if root_scale.x.abs() > 0.000_001 {
            1.0 / root_scale.x
        } else {
            1.0
        },
        if root_scale.y.abs() > 0.000_001 {
            1.0 / root_scale.y
        } else {
            1.0
        },
        if root_scale.z.abs() > 0.000_001 {
            1.0 / root_scale.z
        } else {
            1.0
        },
    );

    // Always hide/tag any COLLIDER_* helper nodes so they don't render.
    for root in &roots {
        let mut guides: Vec<Entity> = Vec::new();
        collect_collider_guides(root, &children_query, &name_query, &mut guides);
        if guides.is_empty()
            && children_query.get(root).is_ok()
            && debug.logged_no_guides_for_root.insert(root)
        {
            info!(
                "glTF colliders: no COLLIDER_* helper nodes found under visual root {:?}",
                root
            );
        }
        for guide_entity in guides {
            if let Ok(mut ec) = commands.get_entity(guide_entity) {
                ec.insert((DiceContainerColliderGuide, Visibility::Hidden));
            }
        }
    }

    // If voxel colliders already exist, remove procedural fallback once (if not removed yet).
    if voxel_colliders.iter().next().is_some() {
        if !debug.procedural_removed {
            for entity in &procedural_colliders {
                commands.entity(entity).despawn();
            }
            debug.procedural_removed = true;
        }
        return;
    }

    let mut spawned_voxel: usize = 0;
    let mut mesh_candidates: usize = 0;

    for root in &roots {
        let mut mesh_entities: Vec<Entity> = Vec::new();
        collect_mesh_descendants(
            root,
            &children_query,
            &mesh_query,
            &name_query,
            &mut mesh_entities,
        );

        mesh_candidates += mesh_entities.len();

        for mesh_entity in mesh_entities {
            let Ok(mesh_handle) = mesh_query.get(mesh_entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(&mesh_handle.0) else {
                if debug.logged_mesh_wait_for_voxel.insert(mesh_entity) {
                    let label = name_query
                        .get(mesh_entity)
                        .map(|n| n.as_str().to_string())
                        .unwrap_or_else(|_| "<unnamed>".to_string());
                    info!(
                        "glTF voxel collider source '{}' ({:?}) mesh not loaded yet; waiting",
                        label,
                        mesh_entity
                    );
                }
                continue;
            };

            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                continue;
            };

            let indices_u32: Vec<u32> = match mesh.indices() {
                Some(Indices::U16(v)) => v.iter().map(|&i| i as u32).collect(),
                Some(Indices::U32(v)) => v.clone(),
                None => (0..positions.len() as u32).collect(),
            };

            // Convert mesh_entity global TRS into container-root local space.
            let Ok(mesh_global) = global_query.get(mesh_entity) else {
                continue;
            };
            let (world_scale, world_rotation, world_translation) =
                mesh_global.to_scale_rotation_translation();
            let world_scale = abs_vec3(world_scale);
            let rel_scale = world_scale * inv_root_scale;
            let local_translation = root_rotation
                .inverse()
                .mul_vec3((world_translation - root_translation) * inv_root_scale);
            let local_rotation = root_rotation.inverse() * world_rotation;

            let mut verts: Vec<bevy_rapier3d::rapier::math::Point<f32>> =
                Vec::with_capacity(positions.len());
            for p in positions.iter() {
                let v = Vec3::new(p[0], p[1], p[2]) * rel_scale;
                verts.push(bevy_rapier3d::rapier::math::Point::new(v.x, v.y, v.z));
            }

            let mut tris: Vec<[u32; 3]> = Vec::with_capacity(indices_u32.len() / 3);
            for chunk in indices_u32.chunks_exact(3) {
                tris.push([chunk[0], chunk[1], chunk[2]]);
            }

            if tris.is_empty() || verts.is_empty() {
                continue;
            }

            let shape =
                SharedShape::voxelized_mesh(&verts, &tris, VOXEL_SIZE, FillMode::default());
            let collider = Collider::from(shape);

            commands.entity(container_root_entity).with_children(|parent| {
                parent.spawn((
                    Transform::from_translation(local_translation).with_rotation(local_rotation),
                    collider,
                    Restitution::coefficient(0.2),
                    Friction::coefficient(0.8),
                    DiceBoxWall,
                    DiceContainerGeneratedCollider,
                    DiceContainerVoxelCollider,
                ));
            });

            spawned_voxel += 1;
        }
    }

    let status = (roots.iter().count(), mesh_candidates, spawned_voxel);
    if debug.last_status != Some(status) {
        info!(
            "glTF voxel colliders status: visual_roots={} mesh_candidates={} spawned_this_frame={}",
            status.0, status.1, status.2
        );
        debug.last_status = Some(status);
    }
}

fn collect_descendants_with_material(
    entity: Entity,
    children_query: &Query<&Children>,
    material_query: &Query<
        &MeshMaterial3d<StandardMaterial>,
        (
            Without<DiceContainerColliderGuide>,
            Without<DiceContainerCrystalMaterialApplied>,
        ),
    >,
    out: &mut Vec<Entity>,
) {
    if material_query.get(entity).is_ok() {
        out.push(entity);
    }

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        collect_descendants_with_material(child, children_query, material_query, out);
    }
}

/// Force the box/cup glTF model meshes to use the game's crystal material.
///
/// This overrides whatever materials are authored in the glTF. We do it once per mesh entity
/// (tracked via `DiceContainerCrystalMaterialApplied`) so hover highlighting can safely clone
/// and mutate per-mesh materials afterward.
pub fn apply_crystal_material_to_container_models(
    mut commands: Commands,
    roots: Query<Entity, With<DiceContainerVisualRoot>>,
    children_query: Query<&Children>,
    material_query: Query<
        &MeshMaterial3d<StandardMaterial>,
        (
            Without<DiceContainerColliderGuide>,
            Without<DiceContainerCrystalMaterialApplied>,
        ),
    >,
    container_materials: Res<DiceContainerMaterials>,
) {
    for root in &roots {
        let mut candidates: Vec<Entity> = Vec::new();
        collect_descendants_with_material(root, &children_query, &material_query, &mut candidates);

        for entity in candidates {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.insert((
                    MeshMaterial3d(container_materials.crystal.clone()),
                    DiceContainerCrystalMaterialApplied,
                ));
            }
        }
    }
}
