use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "D&D Dice Roller 3D".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(DiceResults::default())
        .insert_resource(RollState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            check_dice_settled,
            update_results_display,
            handle_input,
            rotate_camera,
        ))
        .run();
}

#[derive(Component)]
struct Die {
    die_type: DiceType,
    face_normals: Vec<(Vec3, u32)>,
}

#[derive(Component)]
struct DiceBox;

#[derive(Component)]
struct ResultsText;

#[derive(Component)]
struct MainCamera;

#[derive(Clone, Copy, Debug, PartialEq)]
enum DiceType {
    D4, D6, D8, D10, D12, D20,
}

impl DiceType {
    fn max_value(&self) -> u32 {
        match self {
            DiceType::D4 => 4, DiceType::D6 => 6, DiceType::D8 => 8,
            DiceType::D10 => 10, DiceType::D12 => 12, DiceType::D20 => 20,
        }
    }
    fn name(&self) -> &'static str {
        match self {
            DiceType::D4 => "D4", DiceType::D6 => "D6", DiceType::D8 => "D8",
            DiceType::D10 => "D10", DiceType::D12 => "D12", DiceType::D20 => "D20",
        }
    }
    fn color(&self) -> Color {
        match self {
            DiceType::D4 => Color::srgb(0.8, 0.2, 0.2),
            DiceType::D6 => Color::srgb(0.2, 0.8, 0.2),
            DiceType::D8 => Color::srgb(0.2, 0.2, 0.8),
            DiceType::D10 => Color::srgb(0.8, 0.8, 0.2),
            DiceType::D12 => Color::srgb(0.8, 0.2, 0.8),
            DiceType::D20 => Color::srgb(0.2, 0.8, 0.8),
        }
    }
}

#[derive(Resource, Default)]
struct DiceResults { results: Vec<(DiceType, u32)> }

#[derive(Resource, Default)]
struct RollState { rolling: bool, settle_timer: f32 }

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));
    commands.spawn((
        DirectionalLight { illuminance: 10000.0, shadows_enabled: true, ..default() },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight { color: Color::WHITE, brightness: 300.0 });

    let floor_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.15, 0.4, 0.15), ..default() });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 0.5, 16.0))),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, -0.25, 0.0),
        Collider::cuboid(8.0, 0.25, 8.0),
        RigidBody::Fixed,
        DiceBox,
    ));

    let wall_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.3, 0.2, 0.1), ..default() });
    for (pos, size) in [
        (Vec3::new(0.0, 1.0, -8.0), Vec3::new(16.0, 2.5, 0.5)),
        (Vec3::new(0.0, 1.0, 8.0), Vec3::new(16.0, 2.5, 0.5)),
        (Vec3::new(-8.0, 1.0, 0.0), Vec3::new(0.5, 2.5, 16.0)),
        (Vec3::new(8.0, 1.0, 0.0), Vec3::new(0.5, 2.5, 16.0)),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_translation(pos),
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            RigidBody::Fixed,
            DiceBox,
        ));
    }

    let dice = [
        (DiceType::D4, Vec3::new(-5.0, 5.0, -3.0)),
        (DiceType::D6, Vec3::new(-2.0, 5.0, -3.0)),
        (DiceType::D8, Vec3::new(1.0, 5.0, -3.0)),
        (DiceType::D10, Vec3::new(4.0, 5.0, -3.0)),
        (DiceType::D12, Vec3::new(-2.0, 5.0, 3.0)),
        (DiceType::D20, Vec3::new(2.0, 5.0, 3.0)),
    ];
    for (die_type, position) in dice {
        spawn_die(&mut commands, &mut meshes, &mut materials, die_type, position);
    }

    commands.spawn((
        Text::new("Press SPACE to roll dice\nPress R to reset"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), ..default() },
        ResultsText,
    ));
}

fn spawn_die(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    die_type: DiceType,
    position: Vec3,
) {
    let material = materials.add(StandardMaterial { base_color: die_type.color(), ..default() });
    let mut rng = rand::thread_rng();
    let angular_vel = Vec3::new(
        rng.gen_range(-10.0..10.0),
        rng.gen_range(-10.0..10.0),
        rng.gen_range(-10.0..10.0),
    );
    let (mesh, collider, face_normals) = create_die_mesh_and_collider(die_type);
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        RigidBody::Dynamic,
        collider,
        Velocity {
            linvel: Vec3::new(rng.gen_range(-2.0..2.0), 0.0, rng.gen_range(-2.0..2.0)),
            angvel: angular_vel,
        },
        Restitution::coefficient(0.3),
        Friction::coefficient(0.8),
        ColliderMassProperties::Density(2.0),
        Die { die_type, face_normals },
    ));
}

fn create_die_mesh_and_collider(die_type: DiceType) -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    match die_type {
        DiceType::D4 => create_d4(),
        DiceType::D6 => create_d6(),
        DiceType::D8 => create_d8(),
        DiceType::D10 => create_d10(),
        DiceType::D12 => create_d12(),
        DiceType::D20 => create_d20(),
    }
}

fn create_d4() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.8;
    let h = size * (2.0_f32 / 3.0).sqrt();
    let vertices = vec![
        Vec3::new(0.0, h, 0.0),
        Vec3::new(-size / 2.0, 0.0, size * 0.577),
        Vec3::new(size / 2.0, 0.0, size * 0.577),
        Vec3::new(0.0, 0.0, -size * 0.577),
    ];
    let face_normals = vec![
        (Vec3::new(0.0, -1.0, 0.0), 1),
        (Vec3::new(0.0, 0.333, 0.943), 2),
        (Vec3::new(0.816, 0.333, -0.471), 3),
        (Vec3::new(-0.816, 0.333, -0.471), 4),
    ];
    let mesh = Mesh::from(Tetrahedron::default()).scaled_by(Vec3::splat(size));
    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size / 2.0));
    (mesh, collider, face_normals)
}

fn create_d6() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.6;
    let face_normals = vec![
        (Vec3::Y, 6), (Vec3::NEG_Y, 1),
        (Vec3::X, 3), (Vec3::NEG_X, 4),
        (Vec3::Z, 2), (Vec3::NEG_Z, 5),
    ];
    let mesh = Mesh::from(Cuboid::new(size, size, size));
    let collider = Collider::cuboid(size / 2.0, size / 2.0, size / 2.0);
    (mesh, collider, face_normals)
}

fn create_d8() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.5;
    let vertices = vec![
        Vec3::new(0.0, size, 0.0), Vec3::new(0.0, -size, 0.0),
        Vec3::new(size, 0.0, 0.0), Vec3::new(-size, 0.0, 0.0),
        Vec3::new(0.0, 0.0, size), Vec3::new(0.0, 0.0, -size),
    ];
    let n = 0.577;
    let face_normals = vec![
        (Vec3::new(n, n, n), 1), (Vec3::new(-n, n, n), 2),
        (Vec3::new(n, n, -n), 3), (Vec3::new(-n, n, -n), 4),
        (Vec3::new(n, -n, n), 8), (Vec3::new(-n, -n, n), 7),
        (Vec3::new(n, -n, -n), 6), (Vec3::new(-n, -n, -n), 5),
    ];
    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size));
    let mesh = create_octahedron_mesh(size);
    (mesh, collider, face_normals)
}

fn create_d4() -> (Mesh, Collider, Vec<(Vec3, u32)>) {
    let size = 0.8;
    let h = size * (2.0_f32 / 3.0).sqrt();
    let vertices = vec![
        Vec3::new(0.0, h, 0.0),
        Vec3::new(-size / 2.0, 0.0, size * 0.577),
        Vec3::new(size / 2.0, 0.0, size * 0.577),
        Vec3::new(0.0, 0.0, -size * 0.577),
    ];
    let face_normals = vec![
        (Vec3::new(0.0, -1.0, 0.0), 1),
        (Vec3::new(0.0, 0.333, 0.943), 2),
        (Vec3::new(0.816, 0.333, -0.471), 3),
        (Vec3::new(-0.816, 0.333, -0.471), 4),
    ];
    let mesh = Mesh::from(Tetrahedron::default()).scaled_by(Vec3::splat(size));
    let collider = Collider::convex_hull(&vertices).unwrap_or(Collider::ball(size / 2.0));
    (mesh, collider, face_normals)
}
