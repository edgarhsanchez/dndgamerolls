//! 3D Dice Preview Component
//!
//! A component that renders a single die in a dedicated UI area with
//! 3D rotation ring controls. Uses render-to-texture for isolation from main scene.

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureUsages,
};

use crate::dice3d::meshes::create_die_mesh_and_collider;
use crate::dice3d::types::{
    AppTab, DiceDesignerState, DicePreviewContainer, DicePreviewDie,
    DicePreviewRotationRing, DiceType, RotationAxis, UiState,
};

/// Size of the dice preview viewport in pixels
pub const DICE_PREVIEW_SIZE: f32 = 300.0;

/// Render layer for the dice designer preview (separate from main scene)
const DICE_DESIGNER_PREVIEW_LAYER: u8 = 30;

/// Radius of the rotation rings around the die
const RING_RADIUS: f32 = 1.5;

/// Ring thickness
const RING_THICKNESS: f32 = 0.08;

// ============================================================================
// Resources
// ============================================================================

/// Render target for the dice designer preview
#[derive(Resource)]
pub struct DiceDesignerPreviewRenderTarget {
    pub image: Handle<Image>,
}

/// Tracks the dice designer preview scene entities
#[derive(Resource, Default)]
pub struct DiceDesignerPreviewScene {
    pub root: Option<Entity>,
    pub camera: Option<Entity>,
    pub light: Option<Entity>,
    pub die: Option<Entity>,
    pub ring_x: Option<Entity>,
    pub ring_y: Option<Entity>,
    pub ring_z: Option<Entity>,
}

// ============================================================================
// Setup Systems
// ============================================================================

/// Initializes the render target for the dice designer preview
pub fn init_dice_designer_preview_render_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: DICE_PREVIEW_SIZE as u32,
        height: DICE_PREVIEW_SIZE as u32,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("dice_designer_preview_render_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let handle = images.add(image);
    commands.insert_resource(DiceDesignerPreviewRenderTarget { image: handle });
    commands.insert_resource(DiceDesignerPreviewScene::default());
}

/// Manages the dice designer preview scene - spawns/despawns based on active tab
pub fn setup_dice_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    designer_state: Option<Res<DiceDesignerState>>,
    render_target: Option<Res<DiceDesignerPreviewRenderTarget>>,
    mut preview_scene: ResMut<DiceDesignerPreviewScene>,
    ui_state: Res<UiState>,
) {
    let Some(render_target) = render_target else {
        return;
    };

    // Only set up when on the Designer tab
    if ui_state.active_tab != AppTab::DiceDesigner {
        // Despawn if leaving the tab
        despawn_preview_scene(&mut commands, &mut *preview_scene);
        return;
    }

    if preview_scene.root.is_some() {
        return; // Already set up
    }

    let preview_layer = RenderLayers::layer(DICE_DESIGNER_PREVIEW_LAYER as usize);

    let die_type = designer_state
        .map(|s| s.selected_dice)
        .unwrap_or(DiceType::D6);

    // Root entity for the preview scene
    let root = commands
        .spawn((
            Transform::default(),
            Visibility::Visible,
            preview_layer.clone(),
            Name::new("DiceDesignerPreviewRoot"),
        ))
        .id();

    let mut camera_id: Option<Entity> = None;
    let mut light_id: Option<Entity> = None;
    let mut die_id: Option<Entity> = None;
    let mut ring_x_id: Option<Entity> = None;
    let mut ring_y_id: Option<Entity> = None;
    let mut ring_z_id: Option<Entity> = None;

    commands.entity(root).with_children(|parent| {
        // Camera rendering to texture
        let camera = parent
            .spawn((
                Camera3d::default(),
                Camera {
                    target: RenderTarget::Image(render_target.image.clone().into()),
                    clear_color: ClearColorConfig::Custom(Color::srgba(0.1, 0.1, 0.15, 1.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                preview_layer.clone(),
                Name::new("DiceDesignerPreviewCamera"),
            ))
            .id();
        camera_id = Some(camera);

        // Point light
        let light = parent
            .spawn((
                PointLight {
                    intensity: 50000.0,
                    range: 50.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_xyz(4.0, 6.0, 4.0),
                preview_layer.clone(),
                Name::new("DiceDesignerPreviewLight"),
            ))
            .id();
        light_id = Some(light);

        // Preview die
        let (mesh, _collider, _face_normals) = create_die_mesh_and_collider(die_type);
        let material = materials.add(StandardMaterial {
            base_color: die_type.color(),
            perceptual_roughness: 0.3,
            metallic: 0.1,
            reflectance: 0.5,
            ..default()
        });
        let scale = die_type.uniform_size_scale_factor() * 1.5;

        let die = parent
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(scale)),
                preview_layer.clone(),
                DicePreviewDie,
                Name::new("DiceDesignerPreviewDie"),
            ))
            .id();
        die_id = Some(die);

        // Rotation rings
        ring_x_id = Some(spawn_ring(
            parent,
            &mut *meshes,
            &mut *materials,
            RotationAxis::X,
            Color::srgba(0.9, 0.2, 0.2, 0.8),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            preview_layer.clone(),
        ));

        ring_y_id = Some(spawn_ring(
            parent,
            &mut *meshes,
            &mut *materials,
            RotationAxis::Y,
            Color::srgba(0.2, 0.9, 0.2, 0.8),
            Quat::IDENTITY,
            preview_layer.clone(),
        ));

        ring_z_id = Some(spawn_ring(
            parent,
            &mut *meshes,
            &mut *materials,
            RotationAxis::Z,
            Color::srgba(0.2, 0.2, 0.9, 0.8),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            preview_layer.clone(),
        ));
    });

    preview_scene.root = Some(root);
    preview_scene.camera = camera_id;
    preview_scene.light = light_id;
    preview_scene.die = die_id;
    preview_scene.ring_x = ring_x_id;
    preview_scene.ring_y = ring_y_id;
    preview_scene.ring_z = ring_z_id;

    info!(
        "Dice preview scene spawned for {:?} with camera {:?}, die {:?}",
        die_type, camera_id, die_id
    );
}

fn despawn_preview_scene(
    commands: &mut Commands,
    preview_scene: &mut DiceDesignerPreviewScene,
) {
    // Despawning root will despawn all children
    if let Some(root) = preview_scene.root.take() {
        commands.entity(root).despawn();
    }
    preview_scene.camera = None;
    preview_scene.light = None;
    preview_scene.die = None;
    preview_scene.ring_x = None;
    preview_scene.ring_y = None;
    preview_scene.ring_z = None;
}

fn spawn_ring(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    axis: RotationAxis,
    color: Color,
    orientation: Quat,
    render_layer: RenderLayers,
) -> Entity {
    let torus = Torus::new(RING_RADIUS - RING_THICKNESS, RING_RADIUS);

    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.into(),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    parent
        .spawn((
            Mesh3d(meshes.add(torus)),
            MeshMaterial3d(material),
            Transform::from_rotation(orientation),
            Visibility::Hidden, // Hidden until user hovers
            render_layer,
            DicePreviewRotationRing { axis },
            Name::new(format!("RotationRing_{:?}", axis)),
        ))
        .id()
}

// ============================================================================
// Update Systems
// ============================================================================

/// No longer needed - visibility is managed by setup_dice_preview spawning/despawning
pub fn update_preview_visibility(
    _ui_state: Res<UiState>,
) {
    // Preview visibility is now managed by the setup_dice_preview system
    // which spawns/despawns the entire preview scene based on active tab
}

/// Rotates the preview die based on designer state
pub fn update_preview_rotation(
    designer_state: Option<Res<DiceDesignerState>>,
    time: Res<Time>,
    ui_state: Res<UiState>,
    mut die_query: Query<&mut Transform, With<DicePreviewDie>>,
) {
    // Only update when Designer tab is active
    if ui_state.active_tab != AppTab::DiceDesigner {
        return;
    }

    let Some(state) = designer_state else {
        return;
    };

    for mut transform in die_query.iter_mut() {
        if state.auto_rotate && !state.is_dragging_rotation {
            // Auto-rotate slowly
            let rotation = Quat::from_rotation_y(time.delta_secs() * 0.5);
            transform.rotation = rotation * transform.rotation;
        } else if !state.auto_rotate {
            // Apply manual rotation from state
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                state.preview_rotation.x,
                state.preview_rotation.y,
                state.preview_rotation.z,
            );
        }
    }
}

/// Updates the preview die when the selected dice type changes
pub fn update_preview_die_type(
    designer_state: Option<Res<DiceDesignerState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut die_query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>, &mut Transform), With<DicePreviewDie>>,
    ui_state: Res<UiState>,
) {
    // Only update when Designer tab is active
    if ui_state.active_tab != AppTab::DiceDesigner {
        return;
    }

    let Some(state) = designer_state else {
        return;
    };

    if !state.is_changed() {
        return;
    }

    // Update existing die mesh instead of respawning
    for (mut mesh3d, mut material3d, mut transform) in die_query.iter_mut() {
        let die_type = state.selected_dice;
        let (mesh, _collider, _face_normals) = create_die_mesh_and_collider(die_type);

        mesh3d.0 = meshes.add(mesh);
        material3d.0 = materials.add(StandardMaterial {
            base_color: die_type.color(),
            perceptual_roughness: 0.3,
            metallic: 0.1,
            reflectance: 0.5,
            ..default()
        });

        let scale = die_type.uniform_size_scale_factor() * 1.5;
        transform.scale = Vec3::splat(scale);
    }
}

/// Shows rotation rings when hovering or dragging, hides otherwise
pub fn update_rotation_ring_visibility(
    designer_state: Option<Res<DiceDesignerState>>,
    mut ring_query: Query<(&mut Visibility, &DicePreviewRotationRing)>,
    preview_container_query: Query<&Interaction, With<DicePreviewContainer>>,
    ui_state: Res<UiState>,
) {
    // Only update when Designer tab is active
    if ui_state.active_tab != AppTab::DiceDesigner {
        return;
    }

    let Some(state) = designer_state else {
        return;
    };

    // Check if hovering the preview container
    let is_hovering_preview = preview_container_query
        .iter()
        .any(|interaction| *interaction == Interaction::Hovered || *interaction == Interaction::Pressed);

    // Show rings when user is dragging OR hovering
    let should_show = state.is_dragging_rotation || is_hovering_preview;

    for (mut visibility, ring) in ring_query.iter_mut() {
        // During drag, optionally highlight only the active axis
        let show_this_ring = should_show
            && state
                .active_drag_axis
                .map_or(true, |axis| axis == ring.axis);

        *visibility = if show_this_ring {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Updates ring positions to follow the die rotation
pub fn update_rotation_ring_transforms(
    designer_state: Option<Res<DiceDesignerState>>,
    die_query: Query<&Transform, With<DicePreviewDie>>,
    mut ring_query: Query<(&mut Transform, &DicePreviewRotationRing), Without<DicePreviewDie>>,
    ui_state: Res<UiState>,
) {
    // Only update when Designer tab is active
    if ui_state.active_tab != AppTab::DiceDesigner {
        return;
    }

    let Some(_state) = designer_state else {
        return;
    };

    let Ok(die_transform) = die_query.single() else {
        return;
    };

    for (mut ring_transform, ring) in ring_query.iter_mut() {
        // Keep ring centered on die
        ring_transform.translation = die_transform.translation;

        // Apply die rotation to rings, then add axis-specific orientation
        let axis_orientation = match ring.axis {
            RotationAxis::X => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            RotationAxis::Y => Quat::IDENTITY,
            RotationAxis::Z => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        };

        ring_transform.rotation = die_transform.rotation * axis_orientation;
    }
}

/// Handles mouse drag on the preview container for rotation
pub fn handle_preview_drag_rotation(
    mut designer_state: Option<ResMut<DiceDesignerState>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    preview_container_query: Query<&Interaction, With<DicePreviewContainer>>,
    ui_state: Res<UiState>,
) {
    // Only handle when Designer tab is active
    if ui_state.active_tab != AppTab::DiceDesigner {
        return;
    }

    let Some(ref mut state) = designer_state else {
        return;
    };

    // Check if we're interacting with the preview container
    let is_hovering_preview = preview_container_query
        .iter()
        .any(|interaction| *interaction == Interaction::Hovered || *interaction == Interaction::Pressed);

    // Handle drag start/end
    if mouse_button.just_pressed(MouseButton::Left) && is_hovering_preview {
        state.is_dragging_rotation = true;
        state.auto_rotate = false;
    }

    if mouse_button.just_released(MouseButton::Left) {
        state.is_dragging_rotation = false;
        state.active_drag_axis = None;
    }

    // Apply rotation from mouse motion when dragging
    if state.is_dragging_rotation {
        for motion in mouse_motion.read() {
            let sensitivity = 0.01;

            // Horizontal motion rotates around Y axis
            // Vertical motion rotates around X axis
            state.preview_rotation.y += motion.delta.x * sensitivity;
            state.preview_rotation.x += motion.delta.y * sensitivity;
        }
    }
}

// ============================================================================
// UI Component for Preview
// ============================================================================

/// Spawns the UI container for the dice preview with the render target image
pub fn spawn_dice_preview_ui(
    parent: &mut ChildSpawnerCommands,
    _theme_surface: Color,
    render_target: Option<&DiceDesignerPreviewRenderTarget>,
) -> Entity {
    let image_handle = match render_target {
        Some(rt) => {
            info!("Dice preview UI: render target available, using image handle");
            rt.image.clone()
        }
        None => {
            warn!("Dice preview UI: NO render target available!");
            Handle::default()
        }
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(DICE_PREVIEW_SIZE),
                height: Val::Px(DICE_PREVIEW_SIZE),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // Display the render target as the background
            ImageNode {
                image: image_handle,
                ..default()
            },
            BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 1.0)),
            BorderRadius::all(Val::Px(8.0)),
            FocusPolicy::Block,
            DicePreviewContainer,
        ))
        .with_children(|container| {
            // Help text at bottom
            container.spawn((
                Text::new("Drag to rotate"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(8.0),
                    ..default()
                },
            ));
        })
        .id()
}

// Legacy function - kept for compatibility but no longer needed
pub fn handle_gizmo_interactions(
    _designer_state: Option<ResMut<DiceDesignerState>>,
    _gizmo_query: Query<(&Interaction, &crate::dice3d::types::DicePreviewGizmoAxis), Changed<Interaction>>,
) {
    // Deprecated - rotation is now handled by drag on the preview container
}
