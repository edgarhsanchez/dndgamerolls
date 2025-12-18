//! Scene setup system
//!
//! This module contains the main setup function that initializes the 3D scene,
//! including camera, lights, dice box, dice, and UI elements.

use bevy::color::LinearRgba;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_material_ui::icons::MaterialIconFont;
use bevy_material_ui::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::dice3d::box_highlight::{
    DiceBoxFloor, DiceBoxHighlightExtension, DiceBoxHighlightMaterial, DiceBoxHighlightParams,
};
use crate::dice3d::meshes::create_die_mesh_and_collider;
use crate::dice3d::throw_control::{spawn_throw_arrow, StrengthSlider, ThrowControlState};
use crate::dice3d::types::*;

use super::rendering::{create_number_mesh, get_label_offset, get_label_rotation, get_label_scale};

/// Main setup system - initializes the entire 3D scene
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut box_materials: ResMut<Assets<DiceBoxHighlightMaterial>>,
    dice_config: Res<DiceConfig>,
    character_data: Res<CharacterData>,
    zoom_state: Res<ZoomState>,
    shake_state: Res<ShakeState>,
    shake_config: Res<ContainerShakeConfig>,
    throw_state: Res<ThrowControlState>,
    settings_state: Res<SettingsState>,
    icon_font: Res<MaterialIconFont>,
    theme: Res<MaterialTheme>,
    container_style: Res<DiceContainerStyle>,
) {
    // Camera - position based on zoom state (closer by default)
    let camera_distance = zoom_state.get_distance();
    let camera_height = camera_distance * 0.7; // Maintain angle ratio
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, camera_height, camera_distance * 0.7)
            .looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
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

    // Expose container materials for runtime wall rebuilds (box/cup toggle).
    commands.insert_resource(DiceContainerMaterials {
        crystal: crystal_mat.clone(),
    });

    let floor_material = box_materials.add(DiceBoxHighlightMaterial {
        base: StandardMaterial {
            // Windows Sandbox often runs with a virtual GPU / software rendering path where
            // blended transparency can sort incorrectly or appear to vanish. Keep the floor
            // opaque so the "bottom of the box" is always visible.
            base_color: Color::srgba(0.7, 0.85, 0.95, 1.0),
            alpha_mode: AlphaMode::Opaque,
            reflectance: 0.8,
            perceptual_roughness: 0.1,
            metallic: 0.0,
            ..default()
        },
        extension: DiceBoxHighlightExtension {
            params: DiceBoxHighlightParams {
                highlight_color: LinearRgba::from(
                    settings_state.settings.dice_box_highlight_color.to_color(),
                )
                .to_vec4(),
                hovered: 0.0,
                strength: 1.0,
                _pad: Vec2::ZERO,
            },
        },
    });

    // Floor - smaller box (4x4 units)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 0.3, 4.0))),
        MeshMaterial3d(floor_material.clone()),
        Transform::from_xyz(0.0, -0.15, 0.0),
        Collider::cuboid(2.0, 0.15, 2.0),
        RigidBody::KinematicPositionBased,
        Restitution::coefficient(0.2),
        Friction::coefficient(0.8),
        DiceBox,
        DiceBoxFloor,
    ));

    // Walls - taller walls for better containment
    let wall_height = 1.5;
    let wall_thickness = 0.15;
    let box_size = 2.0;

    let spawn_box_walls = |commands: &mut Commands,
                           meshes: &mut ResMut<Assets<Mesh>>,
                           crystal_mat: Handle<StandardMaterial>| {
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
                // Extend along Z so corners overlap with the front/back walls.
                Vec3::new(wall_thickness, wall_height, 4.0 + wall_thickness * 2.0),
            ),
            (
                Vec3::new(box_size, wall_height / 2.0, 0.0),
                // Extend along Z so corners overlap with the front/back walls.
                Vec3::new(wall_thickness, wall_height, 4.0 + wall_thickness * 2.0),
            ),
        ] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                MeshMaterial3d(crystal_mat.clone()),
                Transform::from_translation(pos),
                Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
                RigidBody::KinematicPositionBased,
                Restitution::coefficient(0.2),
                Friction::coefficient(0.8),
                DiceBox,
                DiceBoxWall,
            ));
        }
    };

    let spawn_cup_walls = |commands: &mut Commands,
                           meshes: &mut ResMut<Assets<Mesh>>,
                           crystal_mat: Handle<StandardMaterial>| {
        // Visual cup: a glass cylinder + a simple handle.
        // Collisions: keep the "invisible boundary" principle by using collider-only wall segments.
        let radius: f32 = 2.0;

        // Cylinder visual
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(radius, wall_height))),
            MeshMaterial3d(crystal_mat.clone()),
            Transform::from_xyz(0.0, wall_height / 2.0, 0.0),
            DiceBox,
            DiceBoxWall,
        ));

        // Handle visual (simple rectangular handle)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.6, 0.8, 0.12))),
            MeshMaterial3d(crystal_mat.clone()),
            Transform::from_xyz(radius + 0.35, wall_height * 0.60, 0.0),
            DiceBox,
            DiceBoxWall,
        ));

        // Invisible collider ring (more segments = smoother)
        let segments: usize = 24;
        let segment_length: f32 = (std::f32::consts::TAU * radius) / segments as f32;
        let wall_depth: f32 = wall_thickness;
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let angle = t * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            let pos = Vec3::new(x, wall_height / 2.0, z);
            let rot = Quat::from_rotation_y(-angle);
            let size = Vec3::new(segment_length + wall_thickness, wall_height, wall_depth);

            commands.spawn((
                Transform::from_translation(pos).with_rotation(rot),
                Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
                RigidBody::KinematicPositionBased,
                Restitution::coefficient(0.2),
                Friction::coefficient(0.8),
                DiceBox,
                DiceBoxWall,
            ));
        }
    };

    match *container_style {
        DiceContainerStyle::Box => spawn_box_walls(&mut commands, &mut meshes, crystal_mat.clone()),
        DiceContainerStyle::Cup => spawn_cup_walls(&mut commands, &mut meshes, crystal_mat.clone()),
    }

    // Invisible ceiling collider to prevent dice from bouncing out.
    // Note: collider-only (no mesh/material), so it's completely see-through.
    let ceiling_size = 4.0 + wall_thickness * 2.0;
    let ceiling_thickness = 0.10;
    commands.spawn((
        Transform::from_xyz(0.0, wall_height + ceiling_thickness / 2.0, 0.0),
        Collider::cuboid(
            ceiling_size / 2.0,
            ceiling_thickness / 2.0,
            ceiling_size / 2.0,
        ),
        RigidBody::KinematicPositionBased,
        Restitution::coefficient(0.05),
        Friction::coefficient(0.3),
        DiceBox,
        DiceBoxCeiling,
    ));

    // Spawn dice based on configuration
    let dice_to_spawn = &dice_config.dice_to_roll;
    let num_dice = dice_to_spawn.len();

    let mut rng = rand::thread_rng();
    for (i, die_type) in dice_to_spawn.iter().enumerate() {
        let position = match *container_style {
            DiceContainerStyle::Box => calculate_dice_position(i, num_dice),
            DiceContainerStyle::Cup => {
                // Spawn inside the cup and let gravity "drop" the dice.
                let radius = 1.2;
                let x = rng.gen_range(-radius..radius);
                let z = rng.gen_range(-radius..radius);
                Vec3::new(x, 1.25, z)
            }
        };
        let _die_entity = spawn_die(
            &mut commands,
            &mut meshes,
            &mut materials,
            *die_type,
            position,
        );
    }

    // Dice box controls panel (draggable)
    {
        let pos = settings_state.settings.dice_box_controls_panel_position;

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(pos.x),
                    top: Val::Px(pos.y),
                    width: Val::Px(180.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(theme.surface_container),
                BorderRadius::all(Val::Px(12.0)),
                ZIndex(14),
                DiceRollerRoot,
                DiceBoxControlsPanelRoot,
                DiceBoxControlsPanelDragState::default(),
            ))
            .with_children(|panel| {
                panel
                    .spawn((
                        Button,
                        DiceBoxControlsPanelHandle,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme.surface_container_high),
                        BorderRadius::all(Val::Px(8.0)),
                        Interaction::None,
                        FocusPolicy::Block,
                    ))
                    .with_children(|h| {
                        h.spawn((
                            Text::new("Drag"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));
                    });

                panel.spawn((
                    Text::new("Box Controls"),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                ));

                // Small icon buttons row
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        // Rotate (single direction)
                        row.spawn((
                            IconButtonBuilder::new("rotate").standard().build(&theme),
                            TooltipTrigger::new("Rotate camera").top(),
                            DiceBoxControlsPanelRotateButton,
                        ))
                        .with_children(|b| {
                            // U+E028 (good rotation icon)
                            let icon = MaterialIcon::new('\u{E028}');
                            b.spawn((
                                Text::new(icon.as_str()),
                                TextFont {
                                    font: icon_font.0.clone(),
                                    font_size: ICON_SIZE,
                                    ..default()
                                },
                                TextColor(theme.on_surface_variant),
                            ));
                        });

                        // Shake
                        row.spawn((
                            IconButtonBuilder::new("vibration").standard().build(&theme),
                            TooltipTrigger::new("Shake container").top(),
                            DiceBoxShakeBoxButton,
                        ))
                        .with_children(|b| {
                            // U+EAF2 is a good shake icon.
                            let icon = MaterialIcon::new('\u{EAF2}');
                            b.spawn((
                                Text::new(icon.as_str()),
                                TextFont {
                                    font: icon_font.0.clone(),
                                    font_size: ICON_SIZE,
                                    ..default()
                                },
                                TextColor(theme.on_surface_variant),
                            ));
                        });

                        // Toggle container
                        row.spawn((
                            IconButtonBuilder::new("swap_horiz")
                                .standard()
                                .build(&theme),
                            TooltipTrigger::new("Toggle container (box/cup)").top(),
                            DiceBoxToggleContainerButton,
                        ))
                        .with_children(|b| {
                            // Show a cup icon when the current mode is Box (i.e. click to switch to cup).
                            // U+EA1B is a good cup icon.
                            let icon = match *container_style {
                                DiceContainerStyle::Box => MaterialIcon::new('\u{EA1B}'),
                                DiceContainerStyle::Cup => MaterialIcon::from_name("swap_horiz")
                                    .or_else(|| MaterialIcon::from_name("swap_horizontal_circle"))
                                    .unwrap_or_else(MaterialIcon::search),
                            };
                            b.spawn((
                                Text::new(icon.as_str()),
                                TextFont {
                                    font: icon_font.0.clone(),
                                    font_size: ICON_SIZE,
                                    ..default()
                                },
                                TextColor(theme.on_surface_variant),
                                DiceBoxToggleContainerIconText,
                            ));
                        });
                    });

                panel.spawn((
                    Text::new("Mode: Box"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.primary),
                    DiceBoxContainerModeText,
                ));
            });
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

    // UI - Results panel (draggable)
    let ui_text = format!(
        "{}\n{}\nLeft-click inside the box to roll dice | Press R to reset | Type below to enter a command",
        char_info, modifier_info
    );

    // Global snackbar host (required for ShowSnackbar messages)
    commands.spawn(SnackbarHostBuilder::build());

    {
        let pos = settings_state.settings.results_panel_position;

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(pos.x),
                    top: Val::Px(pos.y),
                    max_width: Val::Px(360.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    min_height: Val::Px(170.0),
                    ..default()
                },
                ZIndex(10),
                DiceRollerRoot,
                ResultsPanelRoot,
                ResultsPanelDragState::default(),
                BackgroundColor(theme.surface_container_highest),
                BorderRadius::all(Val::Px(12.0)),
            ))
            .with_children(|panel| {
                // Drag handle
                panel
                    .spawn((
                        Button,
                        ResultsPanelHandle,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme.surface_container_highest),
                        BorderRadius::all(Val::Px(8.0)),
                        Interaction::None,
                        FocusPolicy::Block,
                    ))
                    .with_children(|h| {
                        h.spawn((
                            Text::new("Drag"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));
                    });

                panel.spawn((
                    Text::new(ui_text),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                    ResultsText,
                    Node {
                        max_width: Val::Px(360.0),
                        ..default()
                    },
                ));
            });
    }

    // Command input UI at bottom
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
            ZIndex(10),
            DiceRollerRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(CardBuilder::new().filled().padding(12.0).build(&theme))
                .with_children(|card| {
                    // Command text input (MD3 TextField)
                    card.spawn(Node {
                        width: Val::Percent(100.0),
                        ..default()
                    })
                    .with_children(|slot| {
                        let builder = TextFieldBuilder::new()
                            .outlined()
                            .label("Command")
                            .placeholder("--dice 2d6 --checkon stealth")
                            .supporting_text("Press Enter to run")
                            .auto_focus(false)
                            .width(Val::Percent(100.0));
                        spawn_text_field_control_with(slot, &theme, builder, CommandInputField);
                    });
                });
        });

    // Draggable Command History panel
    {
        let pos = settings_state.settings.command_history_panel_position;

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(pos.x),
                    top: Val::Px(pos.y),
                    width: Val::Px(200.0),
                    height: Val::Px(170.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(theme.surface_container),
                BorderRadius::all(Val::Px(12.0)),
                ZIndex(12),
                DiceRollerRoot,
                CommandHistoryPanelRoot,
                CommandHistoryPanelDragState::default(),
            ))
            .with_children(|panel| {
                // Drag handle
                panel
                    .spawn((
                        Button,
                        CommandHistoryPanelHandle,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme.surface_container_high),
                        BorderRadius::all(Val::Px(8.0)),
                        Interaction::None,
                        FocusPolicy::Block,
                    ))
                    .with_children(|h| {
                        h.spawn((
                            Text::new("Drag"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));
                    });

                panel
                    .spawn((
                        ScrollContainer::vertical(),
                        ScrollPosition::default(),
                        Node {
                            width: Val::Percent(100.0),
                            // Important in a flex column: allow the scroll area to be smaller than
                            // its content so overflow/scrolling can actually happen.
                            min_height: Val::Px(0.0),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                    ))
                    .with_children(|scroll| {
                        scroll
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(4.0),
                                    ..default()
                                },
                                CommandHistoryList,
                            ))
                            .with_children(|list| {
                                list.spawn((
                                    Text::new("Command History"),
                                    TextFont {
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface),
                                ));
                                list.spawn((
                                    Text::new("(Press 1-9 to reroll)"),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface_variant),
                                ));
                            });
                    });
            });
    }

    // Draggable slider group (zoom + strength + shake) in the dice view
    {
        let pos = settings_state.settings.slider_group_position;

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(pos.x),
                    top: Val::Px(pos.y),
                    width: Val::Px(280.0),
                    height: Val::Px(270.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(theme.surface_container),
                BorderRadius::all(Val::Px(12.0)),
                ZIndex(15),
                DiceRollerRoot,
                SliderGroupRoot,
                SliderGroupDragState::default(),
            ))
            .with_children(|panel| {
                // Drag handle
                panel
                    .spawn((
                        Button,
                        SliderGroupHandle,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(theme.surface_container_high),
                        BorderRadius::all(Val::Px(8.0)),
                        Interaction::None,
                        FocusPolicy::Block,
                    ))
                    .with_children(|h| {
                        h.spawn((
                            Text::new("Drag"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));
                    });

                // Sliders row
                panel
                    .spawn(Node {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        // Zoom column
                        row.spawn((Node {
                            width: Val::Px(30.0),
                            height: Val::Px(220.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|col| {
                                let icon = MaterialIcon::from_name("zoom_in")
                                    .or_else(|| MaterialIcon::from_name("zoom_in_map"))
                                    .unwrap_or_else(MaterialIcon::search);
                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Camera zoom").top(),
                                ))
                                .with_children(|tip| {
                                    tip.spawn((
                                        Text::new(icon.as_str()),
                                        TextFont {
                                            font: icon_font.0.clone(),
                                            font_size: ICON_SIZE,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));
                                });

                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(160.0),
                                        margin: UiRect::vertical(Val::Px(5.0)),
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Camera zoom").right(),
                                ))
                                .with_children(|slot| {
                                    let slider = MaterialSlider::new(0.0, 1.0)
                                        .with_value(zoom_state.level)
                                        .vertical()
                                        .direction(SliderDirection::StartToEnd)
                                        .track_height(6.0)
                                        .thumb_radius(10.0);
                                    spawn_slider_control_with(slot, &theme, slider, ZoomSlider);
                                });

                                let icon = MaterialIcon::from_name("zoom_out")
                                    .or_else(|| MaterialIcon::from_name("zoom_out_map"))
                                    .unwrap_or_else(MaterialIcon::search);
                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Camera zoom").bottom(),
                                ))
                                .with_children(|tip| {
                                    tip.spawn((
                                        Text::new(icon.as_str()),
                                        TextFont {
                                            font: icon_font.0.clone(),
                                            font_size: ICON_SIZE,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));
                                });
                            });

                        // Strength column
                        row.spawn((Node {
                            width: Val::Px(30.0),
                            height: Val::Px(220.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|col| {
                                let icon = MaterialIcon::from_name("north_east")
                                    .or_else(|| MaterialIcon::from_name("trending_up"))
                                    .unwrap_or_else(MaterialIcon::arrow_upward);
                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Throw strength").top(),
                                ))
                                .with_children(|tip| {
                                    tip.spawn((
                                        Text::new(icon.as_str()),
                                        TextFont {
                                            font: icon_font.0.clone(),
                                            font_size: ICON_SIZE,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));
                                });

                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(160.0),
                                        margin: UiRect::vertical(Val::Px(5.0)),
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Throw strength").right(),
                                ))
                                .with_children(|slot| {
                                    let slider = MaterialSlider::new(1.0, 15.0)
                                        .with_value(throw_state.max_strength)
                                        .vertical()
                                        .direction(SliderDirection::EndToStart)
                                        .track_height(6.0)
                                        .thumb_radius(10.0);
                                    spawn_slider_control_with(slot, &theme, slider, StrengthSlider);
                                });

                                let icon = MaterialIcon::from_name("south_west")
                                    .or_else(|| MaterialIcon::from_name("trending_down"))
                                    .unwrap_or_else(MaterialIcon::arrow_downward);
                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Throw strength").bottom(),
                                ))
                                .with_children(|tip| {
                                    tip.spawn((
                                        Text::new(icon.as_str()),
                                        TextFont {
                                            font: icon_font.0.clone(),
                                            font_size: ICON_SIZE,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));
                                });
                            });

                        // Shake column
                        row.spawn((Node {
                            width: Val::Px(30.0),
                            height: Val::Px(220.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|col| {
                                let icon = MaterialIcon::new('\u{EAF2}');
                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Shake strength").top(),
                                ))
                                .with_children(|tip| {
                                    tip.spawn((
                                        Text::new(icon.as_str()),
                                        TextFont {
                                            font: icon_font.0.clone(),
                                            font_size: ICON_SIZE,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));
                                });

                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(160.0),
                                        margin: UiRect::vertical(Val::Px(5.0)),
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Shake strength").right(),
                                ))
                                .with_children(|slot| {
                                    let slider = MaterialSlider::new(0.0, 1.0)
                                        .with_value(shake_state.strength)
                                        .vertical()
                                        .direction(SliderDirection::EndToStart)
                                        .track_height(6.0)
                                        .thumb_radius(10.0);
                                    spawn_slider_control_with(slot, &theme, slider, ShakeSlider);
                                });

                                // Keep the bottom label minimal; the icon above describes the feature.
                                col.spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: 1.0,
                                        ..default()
                                    },
                                    TextColor(Color::NONE),
                                ));
                            });

                        // Shake distance column
                        row.spawn((Node {
                            width: Val::Px(30.0),
                            height: Val::Px(220.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|col| {
                                let icon = MaterialIcon::from_name("swap_horiz")
                                    .or_else(|| MaterialIcon::from_name("straighten"))
                                    .unwrap_or_else(MaterialIcon::search);
                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Shake distance").top(),
                                ))
                                .with_children(|tip| {
                                    tip.spawn((
                                        Text::new(icon.as_str()),
                                        TextFont {
                                            font: icon_font.0.clone(),
                                            font_size: ICON_SIZE,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));
                                });

                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(160.0),
                                        margin: UiRect::vertical(Val::Px(5.0)),
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Shake distance").right(),
                                ))
                                .with_children(|slot| {
                                    let slider = MaterialSlider::new(0.2, 1.6)
                                        .with_value(shake_config.distance)
                                        .vertical()
                                        .direction(SliderDirection::EndToStart)
                                        .track_height(6.0)
                                        .thumb_radius(10.0);
                                    spawn_slider_control_with(
                                        slot,
                                        &theme,
                                        slider,
                                        ShakeDistanceSlider,
                                    );
                                });

                                col.spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: 1.0,
                                        ..default()
                                    },
                                    TextColor(Color::NONE),
                                ));
                            });

                        // Shake speed column
                        row.spawn((Node {
                            width: Val::Px(30.0),
                            height: Val::Px(220.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|col| {
                                let icon = MaterialIcon::from_name("speed")
                                    .or_else(|| MaterialIcon::from_name("schedule"))
                                    .unwrap_or_else(MaterialIcon::clock);
                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(24.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Shake speed").top(),
                                ))
                                .with_children(|tip| {
                                    tip.spawn((
                                        Text::new(icon.as_str()),
                                        TextFont {
                                            font: icon_font.0.clone(),
                                            font_size: ICON_SIZE,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));
                                });

                                col.spawn((
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(160.0),
                                        margin: UiRect::vertical(Val::Px(5.0)),
                                        ..default()
                                    },
                                    Interaction::None,
                                    FocusPolicy::Pass,
                                    TooltipTrigger::new("Shake speed").right(),
                                ))
                                .with_children(|slot| {
                                    let slider = MaterialSlider::new(0.0, 1.0)
                                        .with_value(shake_config.speed)
                                        .vertical()
                                        .direction(SliderDirection::EndToStart)
                                        .track_height(6.0)
                                        .thumb_radius(10.0);
                                    spawn_slider_control_with(
                                        slot,
                                        &theme,
                                        slider,
                                        ShakeSpeedSlider,
                                    );
                                });

                                col.spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: 1.0,
                                        ..default()
                                    },
                                    TextColor(Color::NONE),
                                ));
                            });
                    });
            });
    }

    // Spawn the 3D throw direction arrow
    spawn_throw_arrow(&mut commands, &mut meshes, &mut materials);

    // Spawn the quick roll panel
    spawn_quick_roll_panel(
        &mut commands,
        &character_data,
        &theme,
        icon_font.0.clone(),
        settings_state.settings.quick_roll_panel_position,
    );

    // Spawn the settings button
    super::settings::spawn_settings_button(&mut commands, &theme, icon_font.0.clone());
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
) -> Entity {
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

    // Get die-specific density for realistic physics
    // Larger dice are heavier and roll differently
    let die_density = die_type.density();
    let die_scale = die_type.scale();

    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(die_material),
        Transform::from_translation(position)
            .with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
            ))
            .with_scale(Vec3::splat(die_scale)),
        RigidBody::Dynamic,
        // Prevent fast dice from tunneling through the walls/ceiling.
        Ccd::enabled(),
        collider,
        Velocity {
            linvel: throw_vel,
            angvel: angular_vel,
        },
        Restitution::coefficient(0.15),
        Friction::coefficient(0.7),
        ColliderMassProperties::Density(die_density),
        Die {
            die_type,
            face_normals,
        },
    ));

    let die_entity = entity_commands.id();

    entity_commands.with_children(|parent| {
        // D4 has special numbering: 3 numbers per face
        if die_type == DiceType::D4 {
            let scale = get_label_scale(die_type);
            for (pos, rotation, value) in get_d4_number_positions() {
                // Calculate the face normal from position (pointing outward)
                let normal = pos.normalize();

                // Spawn black outline
                let outline_mesh = create_number_mesh(value, meshes);
                let outline_pos = pos - normal * 0.002;
                parent.spawn((
                    Mesh3d(outline_mesh),
                    MeshMaterial3d(outline_material.clone()),
                    Transform::from_translation(outline_pos)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(scale * 1.2)),
                ));

                // Spawn white number
                let label_mesh = create_number_mesh(value, meshes);
                parent.spawn((
                    Mesh3d(label_mesh),
                    MeshMaterial3d(label_material.clone()),
                    Transform::from_translation(pos)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(scale)),
                ));
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
                parent.spawn((
                    Mesh3d(outline_mesh),
                    MeshMaterial3d(outline_material.clone()),
                    Transform::from_translation(outline_pos)
                        .with_rotation(label_rotation)
                        .with_scale(Vec3::splat(scale * 1.25)),
                ));

                // Spawn white number on top
                let label_mesh = create_number_mesh(*value, meshes);
                parent.spawn((
                    Mesh3d(label_mesh),
                    MeshMaterial3d(label_material.clone()),
                    Transform::from_translation(label_pos)
                        .with_rotation(label_rotation)
                        .with_scale(Vec3::splat(scale)),
                ));
            }
        }
    });

    die_entity
}

/// Spawn the quick roll panel on the right side of the dice roller view
pub fn spawn_quick_roll_panel(
    commands: &mut Commands,
    character_data: &CharacterData,
    theme: &MaterialTheme,
    icon_font: Handle<Font>,
    position: UiPositionSetting,
) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                width: Val::Px(190.0),
                height: Val::Percent(70.0),
                max_height: Val::Px(420.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(theme.surface_container),
            BorderRadius::all(Val::Px(12.0)),
            Visibility::Visible,
            ZIndex(12),
            QuickRollPanel,
            QuickRollPanelDragState::default(),
            DiceRollerRoot,
        ))
        .with_children(|parent| {
            // Drag handle
            parent
                .spawn((
                    Button,
                    QuickRollPanelHandle,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(theme.surface_container_high),
                    BorderRadius::all(Val::Px(8.0)),
                    Interaction::None,
                    FocusPolicy::Block,
                ))
                .with_children(|h| {
                    h.spawn((
                        Text::new("Drag"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));
                });

            // Scrollable content area
            parent
                .spawn((
                    ScrollContainer::vertical(),
                    ScrollPosition::default(),
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        // Important in a flex column: allow the scroll area to be smaller than
                        // its content so overflow/scrolling can actually happen.
                        min_height: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .with_children(|scroll| {
                    scroll
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|card| {
                            // Title
                            card.spawn((
                                Text::new("Quick Rolls"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(theme.primary),
                            ));

                            if let Some(sheet) = &character_data.sheet {
                                // Ability Checks section
                                card.spawn((
                                    Text::new("Ability Checks"),
                                    TextFont {
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface_variant),
                                    Node {
                                        margin: UiRect::top(Val::Px(6.0)),
                                        ..default()
                                    },
                                ));

                                let abilities = [
                                    ("STR", "strength", sheet.modifiers.strength),
                                    ("DEX", "dexterity", sheet.modifiers.dexterity),
                                    ("CON", "constitution", sheet.modifiers.constitution),
                                    ("INT", "intelligence", sheet.modifiers.intelligence),
                                    ("WIS", "wisdom", sheet.modifiers.wisdom),
                                    ("CHA", "charisma", sheet.modifiers.charisma),
                                ];

                                for (abbrev, name, modifier) in abilities {
                                    let sign = if modifier >= 0 { "+" } else { "" };
                                    spawn_quick_roll_button(
                                        card,
                                        &format!("{} ({}{}) ", abbrev, sign, modifier),
                                        QuickRollType::AbilityCheck(name.to_string()),
                                        icon_font.clone(),
                                        theme,
                                    );
                                }

                                // Saving Throws section
                                card.spawn((
                                    Text::new("Saving Throws"),
                                    TextFont {
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface_variant),
                                    Node {
                                        margin: UiRect::top(Val::Px(6.0)),
                                        ..default()
                                    },
                                ));

                                let save_order = [
                                    "strength",
                                    "dexterity",
                                    "constitution",
                                    "intelligence",
                                    "wisdom",
                                    "charisma",
                                ];
                                for save_name in save_order {
                                    if let Some(save) = sheet.saving_throws.get(save_name) {
                                        let abbrev = match save_name {
                                            "strength" => "STR",
                                            "dexterity" => "DEX",
                                            "constitution" => "CON",
                                            "intelligence" => "INT",
                                            "wisdom" => "WIS",
                                            "charisma" => "CHA",
                                            _ => save_name,
                                        };
                                        let sign = if save.modifier >= 0 { "+" } else { "" };
                                        spawn_quick_roll_button(
                                            card,
                                            &format!("{} ({}{}) ", abbrev, sign, save.modifier),
                                            QuickRollType::SavingThrow(save_name.to_string()),
                                            icon_font.clone(),
                                            theme,
                                        );
                                    }
                                }

                                // Skills section
                                card.spawn((
                                    Text::new("Skills"),
                                    TextFont {
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface_variant),
                                    Node {
                                        margin: UiRect::top(Val::Px(6.0)),
                                        ..default()
                                    },
                                ));

                                // Sort skills alphabetically
                                let mut skills: Vec<_> = sheet.skills.iter().collect();
                                skills.sort_by(|a, b| a.0.cmp(b.0));

                                for (skill_name, skill) in skills {
                                    let sign = if skill.modifier >= 0 { "+" } else { "" };
                                    // Format skill name nicely (camelCase to Title Case)
                                    let display_name = format_skill_name(skill_name);
                                    spawn_quick_roll_button(
                                        card,
                                        &format!("{} ({}{}) ", display_name, sign, skill.modifier),
                                        QuickRollType::Skill(skill_name.clone()),
                                        icon_font.clone(),
                                        theme,
                                    );
                                }
                            } else {
                                card.spawn((
                                    Text::new("No character loaded"),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface_variant),
                                ));
                            }
                        });
                });
        })
        .id()
}

/// Spawn a quick roll button
fn spawn_quick_roll_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    roll_type: QuickRollType,
    _icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    parent
        .spawn((
            MaterialButtonBuilder::new(label).text().build(theme),
            QuickRollButton { roll_type },
        ))
        // Override the button's Node style (the button bundle already includes Node)
        .insert(Node {
            width: Val::Percent(100.0),
            height: Val::Px(28.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            padding: UiRect::horizontal(Val::Px(8.0)),
            ..default()
        })
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(theme.primary),
                ButtonLabel,
            ));
        });
}

/// Format skill name from camelCase to Title Case
fn format_skill_name(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if i == 0 {
            result.push(c.to_ascii_uppercase());
        } else if c.is_uppercase() {
            result.push(' ');
            result.push(c);
        } else {
            result.push(c);
        }
    }
    result
}

/// Rebuild the quick roll panel when character data changes
pub fn rebuild_quick_roll_panel(
    mut commands: Commands,
    character_data: Res<CharacterData>,
    theme: Res<MaterialTheme>,
    ui_state: Res<UiState>,
    settings_state: Res<SettingsState>,
    icon_font: Res<MaterialIconFont>,
    panel_query: Query<Entity, With<QuickRollPanel>>,
) {
    if !character_data.is_changed() {
        return;
    }

    // Despawn existing panel (and its descendants)
    for entity in panel_query.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new panel with updated character data
    let panel = spawn_quick_roll_panel(
        &mut commands,
        &character_data,
        &theme,
        icon_font.0.clone(),
        settings_state.settings.quick_roll_panel_position,
    );
    commands
        .entity(panel)
        .insert(if ui_state.active_tab == AppTab::DiceRoller {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
}

pub fn rebuild_command_history_panel(
    mut commands: Commands,
    history: Res<CommandHistory>,
    theme: Res<MaterialTheme>,
    list_query: Query<Entity, With<CommandHistoryList>>,
    children_query: Query<&Children>,
) {
    if !history.is_changed() {
        return;
    }

    rebuild_command_history_list(
        &mut commands,
        &history,
        &theme,
        &list_query,
        &children_query,
    );
}

pub fn rebuild_command_history_list(
    commands: &mut Commands,
    history: &CommandHistory,
    theme: &MaterialTheme,
    list_query: &Query<Entity, With<CommandHistoryList>>,
    children_query: &Query<&Children>,
) {
    for list_entity in list_query.iter() {
        if let Ok(children) = children_query.get(list_entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        commands.entity(list_entity).with_children(|list| {
            list.spawn((
                Text::new("Command History"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(theme.on_surface),
            ));

            if history.commands.is_empty() {
                list.spawn((
                    Text::new("(no commands yet)"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            } else {
                for (i, cmd) in history.commands.iter().enumerate().rev().take(30) {
                    let label = format!("{}: {}", i + 1, cmd);

                    list.spawn((
                        MaterialButtonBuilder::new(&label).text().build(theme),
                        CommandHistoryItem { index: i },
                    ))
                    .insert(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(26.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.primary),
                            ButtonLabel,
                            Node {
                                max_width: Val::Px(200.0),
                                ..default()
                            },
                        ));
                    });
                }
            }
        });
    }
}
