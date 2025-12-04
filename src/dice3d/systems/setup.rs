//! Scene setup system
//!
//! This module contains the main setup function that initializes the 3D scene,
//! including camera, lights, dice box, dice, and UI elements.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::dice3d::meshes::create_die_mesh_and_collider;
use crate::dice3d::types::*;

use super::rendering::{create_number_mesh, get_label_offset, get_label_rotation, get_label_scale};

/// Main setup system - initializes the entire 3D scene
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dice_config: Res<DiceConfig>,
    character_data: Res<CharacterData>,
    zoom_state: Res<ZoomState>,
) {
    // Camera - position based on zoom state (closer by default)
    let camera_distance = zoom_state.get_distance();
    let camera_height = camera_distance * 0.7; // Maintain angle ratio
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, camera_height, camera_distance * 0.7)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Crystal/glass material for the box
    let crystal_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.7, 0.85, 0.95, 0.3), // Light blue crystal, semi-transparent
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.8,
        perceptual_roughness: 0.1,
        metallic: 0.0,
        ..default()
    });

    // Floor - smaller box (4x4 units)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(4.0, 0.3, 4.0)),
            material: crystal_mat.clone(),
            transform: Transform::from_xyz(0.0, -0.15, 0.0),
            ..default()
        },
        Collider::cuboid(2.0, 0.15, 2.0),
        RigidBody::Fixed,
        Restitution::coefficient(0.2),
        Friction::coefficient(0.8),
        DiceBox,
    ));

    // Walls - taller walls for better containment
    let wall_height = 1.5;
    let wall_thickness = 0.15;
    let box_size = 2.0;
    for (pos, size) in [
        (
            Vec3::new(0.0, wall_height / 2.0, -box_size),
            Vec3::new(4.0 + wall_thickness * 2.0, wall_height, wall_thickness),
        ),
        (
            Vec3::new(0.0, wall_height / 2.0, box_size),
            Vec3::new(4.0 + wall_thickness * 2.0, wall_height, wall_thickness),
        ),
        (
            Vec3::new(-box_size, wall_height / 2.0, 0.0),
            Vec3::new(wall_thickness, wall_height, 4.0),
        ),
        (
            Vec3::new(box_size, wall_height / 2.0, 0.0),
            Vec3::new(wall_thickness, wall_height, 4.0),
        ),
    ] {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
                material: crystal_mat.clone(),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            RigidBody::Fixed,
            Restitution::coefficient(0.2),
            Friction::coefficient(0.8),
            DiceBox,
        ));
    }

    // Invisible ceiling to prevent dice from bouncing out
    commands.spawn((
        Collider::cuboid(2.5, 0.2, 2.5),
        Transform::from_xyz(0.0, wall_height - 0.1, 0.0),
        RigidBody::Fixed,
        Restitution::coefficient(0.05),
        Friction::coefficient(0.3),
        DiceBox,
    ));

    // Spawn dice based on configuration
    let dice_to_spawn = &dice_config.dice_to_roll;
    let num_dice = dice_to_spawn.len();

    for (i, die_type) in dice_to_spawn.iter().enumerate() {
        let position = calculate_dice_position(i, num_dice);
        spawn_die(
            &mut commands,
            &mut meshes,
            &mut materials,
            *die_type,
            position,
        );
    }

    // Build character info string
    let char_info = if let Some(sheet) = &character_data.sheet {
        format!(
            "{} - {} {} (Level {})",
            sheet.character.name,
            sheet.character.race,
            sheet.character.class,
            sheet.character.level
        )
    } else {
        String::from("No character loaded")
    };

    // Build modifier info
    let modifier_info = if !dice_config.modifier_name.is_empty() {
        let sign = if dice_config.modifier >= 0 { "+" } else { "" };
        format!(
            "\nModifier: {} ({}{})",
            dice_config.modifier_name, sign, dice_config.modifier
        )
    } else if dice_config.modifier != 0 {
        let sign = if dice_config.modifier >= 0 { "+" } else { "" };
        format!("\nModifier: {}{}", sign, dice_config.modifier)
    } else {
        String::new()
    };

    // UI - Results text at top
    let ui_text = format!(
        "{}\n{}\nPress SPACE to roll dice | Press R to reset | Press / to enter command",
        char_info, modifier_info
    );

    commands.spawn((
        TextBundle::from_section(
            ui_text,
            TextStyle {
                font_size: 22.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ResultsText,
    ));

    // Command input UI at bottom
    commands.spawn((
        TextBundle::from_section(
            "> Type command: --dice 2d6 --checkon stealth  |  Press 1-9 to reroll from history",
            TextStyle {
                font_size: 20.0,
                color: Color::srgba(0.7, 0.7, 0.7, 0.8),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        })
        .with_background_color(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        CommandInputText,
    ));

    // Command history panel on the right
    commands.spawn((
        TextBundle::from_section(
            "Command History:\n(Press 1-9 to reroll)",
            TextStyle {
                font_size: 18.0,
                color: Color::srgba(0.8, 0.8, 0.6, 0.9),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        })
        .with_background_color(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        CommandHistoryList,
    ));

    // Zoom slider on the lower left - vertical orientation
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.0),
                    bottom: Val::Px(60.0),
                    width: Val::Px(30.0),
                    height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            ZoomSliderContainer,
        ))
        .with_children(|parent| {
            // "+" label at top (zoom in)
            parent.spawn(TextBundle::from_section(
                "+",
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgba(0.9, 0.9, 0.9, 0.8),
                    ..default()
                },
            ));

            // Slider track
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(8.0),
                            height: Val::Px(160.0),
                            margin: UiRect::vertical(Val::Px(5.0)),
                            ..default()
                        },
                        background_color: Color::srgba(0.3, 0.3, 0.3, 0.7).into(),
                        ..default()
                    },
                    ZoomSliderTrack,
                ))
                .with_children(|track| {
                    // Slider handle - position based on zoom level
                    track.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                left: Val::Px(-6.0),
                                top: Val::Percent(zoom_state.level * 100.0),
                                ..default()
                            },
                            background_color: Color::srgba(0.8, 0.8, 0.2, 0.9).into(),
                            ..default()
                        },
                        ZoomSliderHandle,
                    ));
                });

            // "-" label at bottom (zoom out)
            parent.spawn(TextBundle::from_section(
                "-",
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgba(0.9, 0.9, 0.9, 0.8),
                    ..default()
                },
            ));
        });
}

/// Calculate the spawn position for a die based on its index
pub fn calculate_dice_position(index: usize, total: usize) -> Vec3 {
    let cols = ((total as f32).sqrt().ceil() as usize).max(1);
    let row = index / cols;
    let col = index % cols;

    let spacing = 0.6;
    let start_x = -((cols - 1) as f32 * spacing) / 2.0;
    let start_z = -((total / cols) as f32 * spacing) / 2.0;

    Vec3::new(
        start_x + col as f32 * spacing,
        1.0, // Start inside the box (below ceiling at 1.4)
        start_z + row as f32 * spacing,
    )
}

/// Spawn a single die entity with physics and number labels
pub fn spawn_die(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    die_type: DiceType,
    position: Vec3,
) {
    use crate::dice3d::meshes::get_d4_number_positions;

    let die_material = materials.add(StandardMaterial {
        base_color: die_type.color(),
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.7,
        perceptual_roughness: 0.15,
        metallic: 0.1,
        ..default()
    });

    let mut rng = rand::thread_rng();

    let angular_vel = Vec3::new(
        rng.gen_range(-8.0..8.0),
        rng.gen_range(-8.0..8.0),
        rng.gen_range(-8.0..8.0),
    );

    let (mesh, collider, face_normals) = create_die_mesh_and_collider(die_type);

    let throw_vel = Vec3::new(
        rng.gen_range(-1.5..1.5),
        rng.gen_range(-0.5..0.0),
        rng.gen_range(-1.5..1.5),
    );

    let outline_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    let label_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    let face_normals_clone = face_normals.clone();

    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(mesh),
                material: die_material,
                transform: Transform::from_translation(position).with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                )),
                ..default()
            },
            RigidBody::Dynamic,
            collider,
            Velocity {
                linvel: throw_vel,
                angvel: angular_vel,
            },
            Restitution::coefficient(0.15),
            Friction::coefficient(0.7),
            ColliderMassProperties::Density(2.0),
            Die {
                die_type,
                face_normals,
            },
        ))
        .with_children(|parent| {
            // D4 has special numbering: 3 numbers per face
            if die_type == DiceType::D4 {
                let scale = get_label_scale(die_type);
                for (pos, rotation, value) in get_d4_number_positions() {
                    // Calculate the face normal from position (pointing outward)
                    let normal = pos.normalize();

                    // Spawn black outline
                    let outline_mesh = create_number_mesh(value, meshes);
                    let outline_pos = pos - normal * 0.002;
                    parent.spawn(PbrBundle {
                        mesh: outline_mesh,
                        material: outline_material.clone(),
                        transform: Transform::from_translation(outline_pos)
                            .with_rotation(rotation)
                            .with_scale(Vec3::splat(scale * 1.2)),
                        ..default()
                    });

                    // Spawn white number
                    let label_mesh = create_number_mesh(value, meshes);
                    parent.spawn(PbrBundle {
                        mesh: label_mesh,
                        material: label_material.clone(),
                        transform: Transform::from_translation(pos)
                            .with_rotation(rotation)
                            .with_scale(Vec3::splat(scale)),
                        ..default()
                    });
                }
            } else {
                // Standard dice: one number per face
                for (normal, value) in &face_normals_clone {
                    let offset = get_label_offset(die_type);
                    let label_rotation = get_label_rotation(*normal);
                    let scale = get_label_scale(die_type);
                    let label_pos = *normal * offset;

                    // Spawn black outline first
                    let outline_mesh = create_number_mesh(*value, meshes);
                    let outline_pos = *normal * (offset - 0.005);
                    parent.spawn(PbrBundle {
                        mesh: outline_mesh,
                        material: outline_material.clone(),
                        transform: Transform::from_translation(outline_pos)
                            .with_rotation(label_rotation)
                            .with_scale(Vec3::splat(scale * 1.25)),
                        ..default()
                    });

                    // Spawn white number on top
                    let label_mesh = create_number_mesh(*value, meshes);
                    parent.spawn(PbrBundle {
                        mesh: label_mesh,
                        material: label_material.clone(),
                        transform: Transform::from_translation(label_pos)
                            .with_rotation(label_rotation)
                            .with_scale(Vec3::splat(scale)),
                        ..default()
                    });
                }
            }
        });
}
