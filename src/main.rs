use std::time::SystemTime;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
    window::CursorGrabMode,
};

use noise::{Fbm, NoiseFn, Simplex};

use bevy_rapier2d::prelude::*;
use level_gen::{marching_squares::marching_squares, matrix::Matrix, point::Point, tiles::Tiles};

mod level_gen;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
        ))
        .add_systems(Startup, (setup_player, setup_camera, setup_env))
        .add_systems(
            Update,
            (
                grab_mouse,
                player_control,
                camera_follow.after(player_control),
            ),
        )
        .run();
}

#[derive(Component)]
struct Player {
    speed: f32,

}

fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn camera_follow(
    mut cameras: Query<&mut Transform, With<Camera>>,
    players: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let player = players.single();
    let mut transform = cameras.single_mut();
    transform.translation = player.translation;
}

/// generates the player's mesh
fn player_mesh() -> Mesh {
    let scale = 10.0;
    let vertices: Vec<Vec3> = vec![
        (-1.0, -1.0, 0.0),
        (1.0, -1.0, 0.0),
        (1.0, 1.0, 0.0),
        (-1.0, 1.0, 0.0),
        (-0.5, -0.5, 0.0),
        (0.5, -0.5, 0.0),
        (0.5, 0.5, 0.0),
        (-0.5, 0.5, 0.0),
    ]
    .into_iter()
    .map(|x| x.into())
    .map(|x: Vec3| x * scale)
    .collect();

    let indices = vec![
        0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3, 3, 7, 4, 3, 4, 0,
    ];
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh
}

/// converts a bevy mesh into a rapier2d trimesh collider
fn mesh_to_collider(mesh: &Mesh) -> Collider {
    let ind: Vec<_> = mesh.indices().unwrap().iter().collect();
    let vertices = get_mesh_verts(mesh);
    let mut indices = vec![];
    for i in 0..ind.len() / 3 {
        indices.push([
            ind[i * 3] as u32,
            ind[i * 3 + 1] as u32,
            ind[i * 3 + 2] as u32,
        ]);
    }
    Collider::trimesh(vertices, indices)
}

fn get_mesh_verts(mesh: &Mesh) -> Vec<Vec2> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|[x, y, _]| (*x, *y).into())
        .collect()
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player = Player {
        speed: 7.0,
    };

    let mesh = player_mesh();
    commands.spawn((
        RigidBody::Dynamic,
        Velocity::default(),
        GravityScale(0.0),
        Sleeping::disabled(),
        Ccd::enabled(),
        Collider::ball(10.0),
        Friction::coefficient(0.0),
        LockedAxes::ROTATION_LOCKED,
        ActiveEvents::COLLISION_EVENTS,
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        },
        ExternalImpulse::default(),
        player,
    ));
}

fn player_control(
    keyboard: Res<Input<KeyCode>>,
    mut windows: Query<&Window>,
    mut query: Query<(&mut ExternalImpulse, &mut Transform, &Velocity, &mut Player)>,
) {
    let (mut impulse, mut transform, vel, mut player) = query.single_mut();
    // movement
    let mut moving = false;
    let mut dir = Vec2::new(0.0, 0.0);
    if keyboard.pressed(KeyCode::W) {
        dir += Vec2::Y;
        moving = true;
    }
    if keyboard.pressed(KeyCode::A) {
        dir += Vec2::NEG_X;
        moving = true;
    }
    if keyboard.pressed(KeyCode::S) {
        dir += Vec2::NEG_Y;
        moving = true;
    }
    if keyboard.pressed(KeyCode::D) {
        dir += Vec2::X;
        moving = true;
    }


    impulse.impulse = dir * player.speed - vel.linvel * 0.01;

    // rotate to cursor
    let window = windows.single_mut();
    if let Some(cursor) = window.cursor_position() {
        let diff = cursor
            - Vec2 {
                x: window.width() / 2.0,
                y: window.height() / 2.0,
            };
        let angle = diff.y.atan2(diff.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

fn verts_to_mesh(verts: Vec<Vec3>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let num_verts = verts.len() as u32;
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.set_indices(Some(Indices::U32((0..num_verts).collect())));
    mesh
}

fn setup_env(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let fbm = Fbm::<Simplex>::new(0);
    let mut matrix = Matrix::new([100, 100]);
    for y in 0..matrix.dim()[1] {
        for x in 0..matrix.dim()[0] {
            let dim: Point<usize, 2> = matrix.dim().into();
            let pt = Point::new([x as f64, y as f64]) / Point::new([dim[0] as f64, dim[1] as f64]);
            let z = fbm.get(pt.v);
            if z < 0.0 {
                matrix.set([x, y], -1);
            } else {
                matrix.set([x, y], 1);
            }
        }
    }
    let tiles = Tiles::new(matrix, 20.0);
    let (verts, coll_verts) = marching_squares(&tiles);
    let mesh = verts_to_mesh(verts);
    let coll_mesh = verts_to_mesh(coll_verts.clone());
    commands.spawn((
        RigidBody::Fixed,
        mesh_to_collider(&coll_mesh),
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            ..default()
        },
    ));
}
