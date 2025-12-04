use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use super::meshes::create_die_mesh_and_collider;
use super::types::*;

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
        Restitution::coefficient(0.2), // Less bouncy
        Friction::coefficient(0.8),
        DiceBox,
    ));

    // Walls - taller walls for better containment
    let wall_height = 1.5;
    let wall_thickness = 0.15;
    let box_size = 2.0; // half-size
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
            Restitution::coefficient(0.2), // Less bouncy
            Friction::coefficient(0.8),
            DiceBox,
        ));
    }

    // Invisible ceiling to prevent dice from bouncing out - covers entire box
    commands.spawn((
        Collider::cuboid(2.5, 0.2, 2.5),
        Transform::from_xyz(0.0, wall_height - 0.1, 0.0),
        RigidBody::Fixed,
        Restitution::coefficient(0.05), // Very low bounce on ceiling
        Friction::coefficient(0.3),
        DiceBox,
    ));

    // Spawn dice based on configuration - start inside the box
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
                                left: Val::Px(-6.0), // Center handle on track
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

fn calculate_dice_position(index: usize, total: usize) -> Vec3 {
    // Spawn dice INSIDE the box (below the ceiling)
    let cols = ((total as f32).sqrt().ceil() as usize).max(1);
    let row = index / cols;
    let col = index % cols;

    let spacing = 0.6; // Tighter spacing for small box
    let start_x = -((cols - 1) as f32 * spacing) / 2.0;
    let start_z = -((total / cols) as f32 * spacing) / 2.0;

    Vec3::new(
        start_x + col as f32 * spacing,
        1.0, // Start inside the box (below ceiling at 1.4)
        start_z + row as f32 * spacing,
    )
}

fn spawn_die(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    die_type: DiceType,
    position: Vec3,
) {
    // Crystal-like translucent material for dice
    let die_material = materials.add(StandardMaterial {
        base_color: die_type.color(),
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.7,
        perceptual_roughness: 0.15,
        metallic: 0.1,
        ..default()
    });

    let mut rng = rand::thread_rng();

    // Moderate initial spin for tumbling
    let angular_vel = Vec3::new(
        rng.gen_range(-8.0..8.0),
        rng.gen_range(-8.0..8.0),
        rng.gen_range(-8.0..8.0),
    );

    let (mesh, collider, face_normals) = create_die_mesh_and_collider(die_type);

    // Throw dice into the box with some horizontal velocity
    let throw_vel = Vec3::new(
        rng.gen_range(-1.5..1.5),
        rng.gen_range(-0.5..0.0), // Slight downward
        rng.gen_range(-1.5..1.5),
    );

    // Create number label materials
    // Black outline/border material - solid black
    let outline_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    // White number material - solid white, bright and visible
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
            Restitution::coefficient(0.15), // Much less bouncy - solid feel
            Friction::coefficient(0.7),     // More friction to settle faster
            ColliderMassProperties::Density(2.0), // Heavier dice
            Die {
                die_type,
                face_normals,
            },
        ))
        .with_children(|parent| {
            // Add number labels for each face
            for (normal, value) in &face_normals_clone {
                let offset = get_label_offset(die_type);
                let label_rotation = get_label_rotation(*normal);
                let scale = get_label_scale(die_type);

                // Position the label just outside the face surface
                let label_pos = *normal * offset;

                // Spawn black outline first (slightly behind and larger)
                let outline_mesh = create_number_mesh(*value, meshes);
                let outline_pos = *normal * (offset - 0.005); // Slightly behind
                parent.spawn(PbrBundle {
                    mesh: outline_mesh,
                    material: outline_material.clone(),
                    transform: Transform::from_translation(outline_pos)
                        .with_rotation(label_rotation)
                        .with_scale(Vec3::splat(scale * 1.25)), // Larger for thicker border effect
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
        });
}

fn get_label_offset(die_type: DiceType) -> f32 {
    // Offset from center of die - place label flush on the face surface
    // Reduced offsets to prevent floating appearance
    match die_type {
        DiceType::D4 => 0.28,  // Tetrahedron - closer to face
        DiceType::D6 => 0.301, // Cube is 0.6 units, face at 0.3 + tiny buffer
        DiceType::D8 => 0.30,  // Octahedron
        DiceType::D10 => 0.33, // Pentagonal trapezohedron
        DiceType::D12 => 0.36, // Dodecahedron
        DiceType::D20 => 0.33, // Icosahedron
    }
}

fn get_label_scale(die_type: DiceType) -> f32 {
    // Scale for number labels - clear and readable
    match die_type {
        DiceType::D4 => 0.22,
        DiceType::D6 => 0.24,
        DiceType::D8 => 0.18,
        DiceType::D10 => 0.15,
        DiceType::D12 => 0.13,
        DiceType::D20 => 0.11,
    }
}

fn get_label_rotation(normal: Vec3) -> Quat {
    // Calculate rotation so the label faces outward from the die face
    // The label mesh has Z pointing forward (out of the mesh), so we need to rotate
    // it to align with the face normal

    // Handle the Y-axis cases specially to avoid gimbal lock
    if normal.y.abs() > 0.99 {
        if normal.y > 0.0 {
            // Top face - rotate label to face up
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
        } else {
            // Bottom face - rotate label to face down
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
        }
    } else {
        // For side faces, use look rotation
        Quat::from_rotation_arc(Vec3::Z, normal)
            * Quat::from_rotation_z(if normal.x < -0.5 {
                std::f32::consts::PI
            } else {
                0.0
            })
    }
}

fn create_number_mesh(value: u32, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
    // Create a mesh representing the number using 7-segment style digits
    meshes.add(create_digit_mesh(value))
}

fn create_digit_mesh(value: u32) -> Mesh {
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    use bevy::render::render_asset::RenderAssetUsages;

    // Create 3D box geometry for numbers
    let (positions, indices) = generate_number_geometry(value);

    // Generate proper normals for 3D boxes
    // Each box has 6 faces with 4 vertices each = 24 vertices per segment
    // Normals: front(+Z), back(-Z), top(+Y), bottom(-Y), left(-X), right(+X)
    let mut normals = Vec::new();
    let verts_per_box = 24;
    let num_boxes = positions.len() / verts_per_box;

    for _ in 0..num_boxes {
        // Front face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, 0.0, 1.0]);
        }
        // Back face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, 0.0, -1.0]);
        }
        // Top face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, 1.0, 0.0]);
        }
        // Bottom face (4 verts)
        for _ in 0..4 {
            normals.push([0.0, -1.0, 0.0]);
        }
        // Left face (4 verts)
        for _ in 0..4 {
            normals.push([-1.0, 0.0, 0.0]);
        }
        // Right face (4 verts)
        for _ in 0..4 {
            normals.push([1.0, 0.0, 0.0]);
        }
    }

    // Handle any remaining vertices (shouldn't happen but just in case)
    while normals.len() < positions.len() {
        normals.push([0.0, 0.0, 1.0]);
    }

    let uvs: Vec<[f32; 2]> = positions.iter().map(|_| [0.5, 0.5]).collect();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn generate_number_geometry(value: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let digits: Vec<u32> = if value == 0 {
        vec![0]
    } else {
        let mut v = value;
        let mut d = Vec::new();
        while v > 0 {
            d.push(v % 10);
            v /= 10;
        }
        d.reverse();
        d
    };

    let num_digits = digits.len();
    let digit_width = 0.6;
    let spacing = 0.1;
    let total_width = num_digits as f32 * digit_width + (num_digits - 1) as f32 * spacing;
    let start_x = -total_width / 2.0 + digit_width / 2.0;

    for (i, &digit) in digits.iter().enumerate() {
        let offset_x = start_x + i as f32 * (digit_width + spacing);
        let base_idx = positions.len() as u32;

        let (digit_pos, digit_idx) = get_digit_geometry(digit, offset_x);

        for pos in digit_pos {
            positions.push(pos);
        }
        for idx in digit_idx {
            indices.push(base_idx + idx);
        }
    }

    (positions, indices)
}

fn get_digit_geometry(digit: u32, offset_x: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    // Smooth curved digit representation using rounded segments
    let stroke_width = 0.12; // Thinner stroke for cleaner look
    let h = 0.5; // Half height
    let w = 0.35; // Half width
    let d = 0.02; // Very thin depth - flat on surface
    let curve_segments = 6; // Segments for curves

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Helper to add a rounded rectangle segment (pill shape)
    let add_segment = |positions: &mut Vec<[f32; 3]>,
                       indices: &mut Vec<u32>,
                       x1: f32,
                       y1: f32,
                       x2: f32,
                       y2: f32| {
        let base_idx = positions.len() as u32;
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        // Perpendicular direction for width
        let px = -dy / len * stroke_width / 2.0;
        let py = dx / len * stroke_width / 2.0;

        // Front face - quad along the segment
        positions.push([offset_x + x1 - px, y1 - py, d / 2.0]);
        positions.push([offset_x + x1 + px, y1 + py, d / 2.0]);
        positions.push([offset_x + x2 + px, y2 + py, d / 2.0]);
        positions.push([offset_x + x2 - px, y2 - py, d / 2.0]);
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);

        // Back face
        let base_idx = positions.len() as u32;
        positions.push([offset_x + x1 + px, y1 + py, -d / 2.0]);
        positions.push([offset_x + x1 - px, y1 - py, -d / 2.0]);
        positions.push([offset_x + x2 - px, y2 - py, -d / 2.0]);
        positions.push([offset_x + x2 + px, y2 + py, -d / 2.0]);
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
    };

    // Helper to add curved segment
    let add_curve = |positions: &mut Vec<[f32; 3]>,
                     indices: &mut Vec<u32>,
                     cx: f32,
                     cy: f32,
                     radius: f32,
                     start_angle: f32,
                     end_angle: f32| {
        for i in 0..curve_segments {
            let t1 = i as f32 / curve_segments as f32;
            let t2 = (i + 1) as f32 / curve_segments as f32;
            let a1 = start_angle + (end_angle - start_angle) * t1;
            let a2 = start_angle + (end_angle - start_angle) * t2;
            let x1 = cx + radius * a1.cos();
            let y1 = cy + radius * a1.sin();
            let x2 = cx + radius * a2.cos();
            let y2 = cy + radius * a2.sin();
            add_segment(positions, indices, x1, y1, x2, y2);
        }
    };

    let pi = std::f32::consts::PI;
    let half_pi = std::f32::consts::FRAC_PI_2;

    // Define digit paths using line segments and curves
    match digit {
        0 => {
            // Oval shape
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.5,
                w * 0.6,
                half_pi,
                pi + half_pi,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.5,
                w * 0.6,
                -half_pi,
                half_pi,
            );
            add_segment(
                &mut positions,
                &mut indices,
                -w * 0.6,
                h * 0.5,
                -w * 0.6,
                -h * 0.5,
            );
            add_segment(
                &mut positions,
                &mut indices,
                w * 0.6,
                h * 0.5,
                w * 0.6,
                -h * 0.5,
            );
        }
        1 => {
            // Simple vertical line with small top serif
            add_segment(&mut positions, &mut indices, 0.0, h, 0.0, -h);
            add_segment(&mut positions, &mut indices, -w * 0.3, h * 0.6, 0.0, h);
        }
        2 => {
            // Top curve, diagonal, bottom
            add_curve(&mut positions, &mut indices, 0.0, h * 0.5, w * 0.5, 0.0, pi);
            add_segment(&mut positions, &mut indices, w * 0.5, h * 0.5, -w * 0.6, -h);
            add_segment(&mut positions, &mut indices, -w * 0.6, -h, w * 0.6, -h);
        }
        3 => {
            // Two curves stacked
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.5,
                w * 0.5,
                -half_pi,
                pi,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.5,
                w * 0.5,
                -pi,
                half_pi,
            );
        }
        4 => {
            // Angled line with vertical
            add_segment(&mut positions, &mut indices, -w * 0.6, h, -w * 0.6, 0.0);
            add_segment(&mut positions, &mut indices, -w * 0.6, 0.0, w * 0.6, 0.0);
            add_segment(&mut positions, &mut indices, w * 0.4, h, w * 0.4, -h);
        }
        5 => {
            // Top, down, curve bottom
            add_segment(&mut positions, &mut indices, w * 0.5, h, -w * 0.5, h);
            add_segment(&mut positions, &mut indices, -w * 0.5, h, -w * 0.5, 0.0);
            add_segment(&mut positions, &mut indices, -w * 0.5, 0.0, w * 0.3, 0.0);
            add_curve(
                &mut positions,
                &mut indices,
                w * 0.1,
                -h * 0.5,
                w * 0.5,
                half_pi,
                -pi,
            );
        }
        6 => {
            // Top curve into full bottom circle
            add_curve(&mut positions, &mut indices, 0.0, h * 0.3, w * 0.5, 0.0, pi);
            add_segment(
                &mut positions,
                &mut indices,
                -w * 0.5,
                h * 0.3,
                -w * 0.5,
                -h * 0.3,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.4,
                w * 0.5,
                0.0,
                2.0 * pi,
            );
        }
        7 => {
            // Top line with diagonal
            add_segment(&mut positions, &mut indices, -w * 0.5, h, w * 0.5, h);
            add_segment(&mut positions, &mut indices, w * 0.5, h, -w * 0.2, -h);
        }
        8 => {
            // Two stacked circles
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.5,
                w * 0.4,
                0.0,
                2.0 * pi,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.45,
                w * 0.5,
                0.0,
                2.0 * pi,
            );
        }
        9 => {
            // Top circle with tail
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                h * 0.4,
                w * 0.5,
                0.0,
                2.0 * pi,
            );
            add_segment(
                &mut positions,
                &mut indices,
                w * 0.5,
                h * 0.2,
                w * 0.5,
                -h * 0.3,
            );
            add_curve(
                &mut positions,
                &mut indices,
                0.0,
                -h * 0.3,
                w * 0.5,
                0.0,
                -pi,
            );
        }
        _ => {
            // Fallback: simple box
            add_segment(&mut positions, &mut indices, -w * 0.5, h, w * 0.5, h);
            add_segment(&mut positions, &mut indices, w * 0.5, h, w * 0.5, -h);
            add_segment(&mut positions, &mut indices, w * 0.5, -h, -w * 0.5, -h);
            add_segment(&mut positions, &mut indices, -w * 0.5, -h, -w * 0.5, h);
        }
    }

    (positions, indices)
}

pub fn check_dice_settled(
    mut roll_state: ResMut<RollState>,
    mut dice_results: ResMut<DiceResults>,
    dice_query: Query<(&Die, &Velocity, &Transform)>,
    time: Res<Time>,
) {
    if !roll_state.rolling {
        return;
    }

    let all_settled = dice_query
        .iter()
        .all(|(_, vel, _)| vel.linvel.length() < 0.1 && vel.angvel.length() < 0.1);

    if all_settled {
        roll_state.settle_timer += time.delta_seconds();

        if roll_state.settle_timer > 0.5 {
            roll_state.rolling = false;
            roll_state.settle_timer = 0.0;

            dice_results.results.clear();
            for (die, _, transform) in dice_query.iter() {
                let result = determine_dice_result(die, transform);
                dice_results.results.push((die.die_type, result));
            }
        }
    } else {
        roll_state.settle_timer = 0.0;
    }
}

fn determine_dice_result(die: &Die, transform: &Transform) -> u32 {
    let up = Vec3::Y;
    let mut best_match = 1;
    let mut best_dot = -2.0_f32;

    for (normal, value) in &die.face_normals {
        let world_normal = transform.rotation * *normal;
        let dot = world_normal.dot(up);

        if dot > best_dot {
            best_dot = dot;
            best_match = *value;
        }
    }

    best_match
}

pub fn update_results_display(
    dice_results: Res<DiceResults>,
    roll_state: Res<RollState>,
    dice_config: Res<DiceConfig>,
    character_data: Res<CharacterData>,
    mut text_query: Query<&mut Text, With<ResultsText>>,
) {
    for mut text in text_query.iter_mut() {
        // Character info header
        let char_info = if let Some(sheet) = &character_data.sheet {
            format!(
                "{} - {} {} (Level {})\n",
                sheet.character.name,
                sheet.character.race,
                sheet.character.class,
                sheet.character.level
            )
        } else {
            String::from("No character loaded\n")
        };

        if roll_state.rolling {
            text.sections[0].value = format!("{}Rolling...", char_info);
        } else if dice_results.results.is_empty() {
            let modifier_info = format_modifier_info(&dice_config);
            text.sections[0].value = format!(
                "{}{}\nPress SPACE to roll dice\nPress R to reset",
                char_info, modifier_info
            );
        } else {
            let mut result_text = format!("{}Results:\n", char_info);
            let mut total = 0i32;

            // Group results by die type using BTreeMap for stable ordering
            let mut grouped: std::collections::BTreeMap<u32, (DiceType, Vec<u32>)> =
                std::collections::BTreeMap::new();
            for (die_type, value) in &dice_results.results {
                // Key by max_value for consistent ordering (D4=4, D6=6, etc.)
                let key = die_type.max_value();
                grouped
                    .entry(key)
                    .or_insert_with(|| (*die_type, Vec::new()))
                    .1
                    .push(*value);
            }

            // Sort values within each group for consistent display
            for (_die_type, values) in grouped.values_mut() {
                values.sort();
            }

            for (die_type, values) in grouped.values() {
                let sum: u32 = values.iter().sum();
                total += sum as i32;
                if values.len() == 1 {
                    result_text.push_str(&format!("{}: {}\n", die_type.name(), values[0]));
                } else {
                    let values_str: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    result_text.push_str(&format!(
                        "{}x{}: {} = {}\n",
                        values.len(),
                        die_type.name(),
                        values_str.join(" + "),
                        sum
                    ));
                }
            }

            // Apply modifier
            let modifier = dice_config.modifier;
            let final_total = total + modifier;

            if modifier != 0 {
                let sign = if modifier >= 0 { "+" } else { "" };
                let mod_name = if !dice_config.modifier_name.is_empty() {
                    format!(" ({})", dice_config.modifier_name)
                } else {
                    String::new()
                };
                result_text.push_str(&format!(
                    "\nDice Total: {}\nModifier{}: {}{}\n\nFINAL TOTAL: {}",
                    total, mod_name, sign, modifier, final_total
                ));
            } else {
                result_text.push_str(&format!("\nTOTAL: {}", total));
            }

            result_text.push_str("\n\nPress SPACE to roll again\nPress R to reset");
            text.sections[0].value = result_text;
        }
    }
}

fn format_modifier_info(dice_config: &DiceConfig) -> String {
    if !dice_config.modifier_name.is_empty() {
        let sign = if dice_config.modifier >= 0 { "+" } else { "" };
        format!(
            "Modifier: {} ({}{})\n",
            dice_config.modifier_name, sign, dice_config.modifier
        )
    } else if dice_config.modifier != 0 {
        let sign = if dice_config.modifier >= 0 { "+" } else { "" };
        format!("Modifier: {}{}\n", sign, dice_config.modifier)
    } else {
        String::new()
    }
}

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut roll_state: ResMut<RollState>,
    mut dice_results: ResMut<DiceResults>,
    mut dice_query: Query<(&mut Transform, &mut Velocity), With<Die>>,
    dice_config: Res<DiceConfig>,
    command_input: Res<CommandInput>,
) {
    // Don't process game inputs when command input is active
    if command_input.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) && !roll_state.rolling {
        roll_state.rolling = true;
        dice_results.results.clear();

        let mut rng = rand::thread_rng();
        let num_dice = dice_config.dice_to_roll.len();

        for (i, (mut transform, mut velocity)) in dice_query.iter_mut().enumerate() {
            let position = calculate_dice_position(i, num_dice);
            // Add slight randomness to starting position
            transform.translation = position
                + Vec3::new(
                    rng.gen_range(-0.3..0.3),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(-0.3..0.3),
                );
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
            );

            // Throw dice with more energy so they bounce around
            velocity.linvel = Vec3::new(
                rng.gen_range(-3.0..3.0),
                rng.gen_range(-2.0..0.0), // Throw downward
                rng.gen_range(-3.0..3.0),
            );
            velocity.angvel = Vec3::new(
                rng.gen_range(-20.0..20.0),
                rng.gen_range(-20.0..20.0),
                rng.gen_range(-20.0..20.0),
            );
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        roll_state.rolling = false;
        dice_results.results.clear();

        let num_dice = dice_config.dice_to_roll.len();

        for (i, (mut transform, mut velocity)) in dice_query.iter_mut().enumerate() {
            let mut pos = calculate_dice_position(i, num_dice);
            pos.y = 0.3; // Rest on floor
            transform.translation = pos;
            transform.rotation = Quat::IDENTITY;
            velocity.linvel = Vec3::ZERO;
            velocity.angvel = Vec3::ZERO;
        }
    }
}

pub fn rotate_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut zoom_state: ResMut<ZoomState>,
    mut handle_query: Query<&mut Style, With<ZoomSliderHandle>>,
) {
    let rotation_speed = 1.0;
    let zoom_speed = 0.5;

    for mut transform in camera_query.iter_mut() {
        let mut angle = 0.0;

        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            angle += rotation_speed * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            angle -= rotation_speed * time.delta_seconds();
        }

        if angle != 0.0 {
            let rotation = Quat::from_rotation_y(angle);
            let pos = transform.translation;
            let new_pos = rotation * pos;
            transform.translation = new_pos;
            *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
        }

        // Keyboard zoom with updated limits
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            zoom_state.level = (zoom_state.level - zoom_speed * time.delta_seconds()).max(0.0);
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            zoom_state.level = (zoom_state.level + zoom_speed * time.delta_seconds()).min(1.0);
        }

        // Apply zoom to camera
        let target_distance = zoom_state.get_distance();
        let current_dir = transform.translation.normalize();
        transform.translation = current_dir * target_distance;
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);

        // Update slider handle position
        for mut style in handle_query.iter_mut() {
            style.top = Val::Percent(zoom_state.level * 100.0);
        }
    }
}

/// Handle mouse interaction with the zoom slider
pub fn handle_zoom_slider(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut zoom_state: ResMut<ZoomState>,
    track_query: Query<(&Node, &GlobalTransform), With<ZoomSliderTrack>>,
    mut handle_query: Query<&mut Style, With<ZoomSliderHandle>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    windows: Query<&Window>,
) {
    // Only handle when left mouse button is pressed
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Check if cursor is within the slider track area
    for (node, global_transform) in track_query.iter() {
        let track_rect = node.logical_rect(global_transform);

        // Expand the click area horizontally for easier interaction
        let expanded_rect = bevy::math::Rect {
            min: Vec2::new(track_rect.min.x - 15.0, track_rect.min.y),
            max: Vec2::new(track_rect.max.x + 15.0, track_rect.max.y),
        };

        if expanded_rect.contains(cursor_position) {
            // Calculate zoom level from cursor Y position within track
            let relative_y = cursor_position.y - track_rect.min.y;
            let track_height = track_rect.height();
            let new_level = (relative_y / track_height).clamp(0.0, 1.0);

            zoom_state.level = new_level;

            // Update handle position
            for mut style in handle_query.iter_mut() {
                style.top = Val::Percent(new_level * 100.0);
            }

            // Update camera position
            for mut transform in camera_query.iter_mut() {
                let target_distance = zoom_state.get_distance();
                let current_dir = transform.translation.normalize();
                transform.translation = current_dir * target_distance;
                *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_command_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut command_input: ResMut<CommandInput>,
    mut command_history: ResMut<CommandHistory>,
    mut input_text_query: Query<&mut Text, With<CommandInputText>>,
    mut history_text_query: Query<&mut Text, (With<CommandHistoryList>, Without<CommandInputText>)>,
    mut dice_config: ResMut<DiceConfig>,
    mut dice_results: ResMut<DiceResults>,
    mut roll_state: ResMut<RollState>,
    character_data: Res<CharacterData>,
    // For respawning dice
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dice_query: Query<Entity, With<Die>>,
) {
    // Handle number keys 1-9 to reroll from history (when not in input mode)
    if !command_input.active {
        let history_keys = [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
            (KeyCode::Digit4, 3),
            (KeyCode::Digit5, 4),
            (KeyCode::Digit6, 5),
            (KeyCode::Digit7, 6),
            (KeyCode::Digit8, 7),
            (KeyCode::Digit9, 8),
        ];

        for (key, index) in history_keys {
            if keyboard.just_pressed(key) {
                if let Some(cmd) = command_history.commands.get(index).cloned() {
                    // Execute the command from history
                    if let Some(new_config) = parse_command(&cmd, &character_data) {
                        // Remove old dice
                        for entity in dice_query.iter() {
                            commands.entity(entity).despawn_recursive();
                        }

                        // Update config
                        *dice_config = new_config;
                        dice_results.results.clear();
                        roll_state.rolling = false;

                        // Spawn new dice
                        for (i, die_type) in dice_config.dice_to_roll.iter().enumerate() {
                            let position =
                                calculate_dice_position(i, dice_config.dice_to_roll.len());
                            spawn_die(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                *die_type,
                                position,
                            );
                        }
                    }
                    return;
                }
            }
        }
    }

    // Toggle command input with / or Enter when not active
    if !command_input.active
        && (keyboard.just_pressed(KeyCode::Slash) || keyboard.just_pressed(KeyCode::Enter))
    {
        command_input.active = true;
        command_input.text.clear();
        for mut text in input_text_query.iter_mut() {
            text.sections[0].value = "> ".to_string();
            text.sections[0].style.color = Color::srgb(1.0, 1.0, 0.5);
        }
        return;
    }

    if !command_input.active {
        return;
    }

    // Handle escape to cancel
    if keyboard.just_pressed(KeyCode::Escape) {
        command_input.active = false;
        command_input.text.clear();
        for mut text in input_text_query.iter_mut() {
            text.sections[0].value =
                "> Type command: --dice 2d6 --checkon stealth  |  Press 1-9 to reroll from history"
                    .to_string();
            text.sections[0].style.color = Color::srgba(0.7, 0.7, 0.7, 0.8);
        }
        return;
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        command_input.text.pop();
        for mut text in input_text_query.iter_mut() {
            text.sections[0].value = format!("> {}_", command_input.text);
        }
        return;
    }

    // Handle enter to submit
    if keyboard.just_pressed(KeyCode::Enter) {
        let cmd = command_input.text.clone();
        command_input.active = false;
        command_input.text.clear();

        // Parse and apply the command
        if let Some(new_config) = parse_command(&cmd, &character_data) {
            // Add to command history (only unique commands)
            command_history.add_command(cmd.clone());

            // Update history display
            update_history_display(&command_history, &mut history_text_query);

            // Remove old dice
            for entity in dice_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Update config
            *dice_config = new_config;
            dice_results.results.clear();
            roll_state.rolling = false;

            // Spawn new dice
            for (i, die_type) in dice_config.dice_to_roll.iter().enumerate() {
                let position = calculate_dice_position(i, dice_config.dice_to_roll.len());
                spawn_die(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    *die_type,
                    position,
                );
            }
        }

        for mut text in input_text_query.iter_mut() {
            text.sections[0].value =
                "> Type command: --dice 2d6 --checkon stealth  |  Press 1-9 to reroll from history"
                    .to_string();
            text.sections[0].style.color = Color::srgba(0.7, 0.7, 0.7, 0.8);
        }
        return;
    }

    // Handle character input
    for event in char_events.read() {
        if event.state == bevy::input::ButtonState::Pressed {
            // Map key codes to characters
            let c = match event.key_code {
                KeyCode::Space => ' ',
                KeyCode::Minus => '-',
                KeyCode::Equal => '=',
                KeyCode::Digit0 => '0',
                KeyCode::Digit1 => '1',
                KeyCode::Digit2 => '2',
                KeyCode::Digit3 => '3',
                KeyCode::Digit4 => '4',
                KeyCode::Digit5 => '5',
                KeyCode::Digit6 => '6',
                KeyCode::Digit7 => '7',
                KeyCode::Digit8 => '8',
                KeyCode::Digit9 => '9',
                KeyCode::KeyA => 'a',
                KeyCode::KeyB => 'b',
                KeyCode::KeyC => 'c',
                KeyCode::KeyD => 'd',
                KeyCode::KeyE => 'e',
                KeyCode::KeyF => 'f',
                KeyCode::KeyG => 'g',
                KeyCode::KeyH => 'h',
                KeyCode::KeyI => 'i',
                KeyCode::KeyJ => 'j',
                KeyCode::KeyK => 'k',
                KeyCode::KeyL => 'l',
                KeyCode::KeyM => 'm',
                KeyCode::KeyN => 'n',
                KeyCode::KeyO => 'o',
                KeyCode::KeyP => 'p',
                KeyCode::KeyQ => 'q',
                KeyCode::KeyR => 'r',
                KeyCode::KeyS => 's',
                KeyCode::KeyT => 't',
                KeyCode::KeyU => 'u',
                KeyCode::KeyV => 'v',
                KeyCode::KeyW => 'w',
                KeyCode::KeyX => 'x',
                KeyCode::KeyY => 'y',
                KeyCode::KeyZ => 'z',
                _ => continue,
            };
            command_input.text.push(c);
        }
    }

    // Update display
    for mut text in input_text_query.iter_mut() {
        text.sections[0].value = format!("> {}_", command_input.text);
    }
}

fn parse_command(cmd: &str, character_data: &CharacterData) -> Option<DiceConfig> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut dice_to_roll = Vec::new();
    let mut modifier = 0i32;
    let mut modifier_name = String::new();
    let mut checkon: Option<String> = None;

    let mut i = 0;
    while i < parts.len() {
        let part = parts[i];

        if part == "--dice" || part == "-d" {
            if i + 1 < parts.len() {
                i += 1;
                if let Some((count, die_type)) = parse_dice_str(parts[i]) {
                    for _ in 0..count {
                        dice_to_roll.push(die_type);
                    }
                }
            }
        } else if part == "--checkon" {
            if i + 1 < parts.len() {
                i += 1;
                checkon = Some(parts[i].to_string());
            }
        } else if part == "--modifier" || part == "-m" {
            if i + 1 < parts.len() {
                i += 1;
                if let Ok(m) = parts[i].parse::<i32>() {
                    modifier += m;
                }
            }
        } else if part.contains('d') && !part.starts_with('-') {
            // Direct dice notation like "2d6"
            if let Some((count, die_type)) = parse_dice_str(part) {
                for _ in 0..count {
                    dice_to_roll.push(die_type);
                }
            }
        }

        i += 1;
    }

    // Apply checkon modifier
    if let Some(check) = checkon {
        let check_lower = check.to_lowercase();

        if let Some(skill_mod) = character_data.get_skill_modifier(&check_lower) {
            modifier += skill_mod;
            modifier_name = check.clone();
        } else if let Some(ability_mod) = character_data.get_ability_modifier(&check_lower) {
            modifier += ability_mod;
            modifier_name = format!("{} check", check);
        } else if let Some(save_mod) = character_data.get_saving_throw_modifier(&check_lower) {
            modifier += save_mod;
            modifier_name = format!("{} save", check);
        } else {
            modifier_name = check;
        }
    }

    // Default to 1d20 if no dice specified
    if dice_to_roll.is_empty() {
        dice_to_roll.push(DiceType::D20);
    }

    Some(DiceConfig {
        dice_to_roll,
        modifier,
        modifier_name,
    })
}

fn parse_dice_str(s: &str) -> Option<(usize, DiceType)> {
    let s = s.to_lowercase();

    let (count_str, die_str) = if s.starts_with('d') {
        ("1", s.as_str())
    } else if let Some(pos) = s.find('d') {
        (&s[..pos], &s[pos..])
    } else {
        return None;
    };

    let count: usize = count_str.parse().ok()?;
    let die_type = DiceType::parse(die_str)?;

    Some((count, die_type))
}

fn update_history_display(
    history: &CommandHistory,
    history_text_query: &mut Query<
        &mut Text,
        (With<CommandHistoryList>, Without<CommandInputText>),
    >,
) {
    let mut history_text = String::from("Command History:\n");

    if history.commands.is_empty() {
        history_text.push_str("(no commands yet)");
    } else {
        for (i, cmd) in history.commands.iter().enumerate().take(9) {
            history_text.push_str(&format!("[{}] {}\n", i + 1, cmd));
        }
    }

    for mut text in history_text_query.iter_mut() {
        text.sections[0].value = history_text.clone();
    }
}
