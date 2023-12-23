use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use bevy_prototype_lyon::{
    draw::{Fill, Stroke},
    entity::ShapeBundle,
    path::PathBuilder,
    plugin::ShapePlugin,
};
use bullet::bullet_system;
use mesh::{mesh_to_collider, verts_to_mesh};
use mouse::{mouse_world_coords, MouseWorldCoords};
use noise::{Fbm, NoiseFn, Simplex};

use bevy_rapier2d::prelude::*;
use level_gen::{marching_squares::marching_squares, matrix::Matrix, point::Point, tiles::Tiles};
use player::{player_control, setup_player, Player};
use turret::{turret_system, Turret};

mod bullet;
mod level_gen;
mod mesh;
mod mouse;
mod player;
mod turret;

#[derive(Component)]
pub struct Environment;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(MouseWorldCoords(Vec2::ZERO))
        .add_plugins((
            DefaultPlugins,
            ShapePlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0),
        ))
        .add_systems(
            Startup,
            (setup_camera, setup_env, setup_player, setup_trajectory_line),
        )
        .add_systems(
            Update,
            (
                zoom,
                player_control,
                camera_follow,
                mouse_world_coords.before(player_control),
                bullet_system,
                turret_system
            ),
        )
        .run();
}

fn setup_trajectory_line(mut commands: Commands) {
    let mut path_builder = PathBuilder::new();
    path_builder.close();
    let path = path_builder.build();

    commands.spawn((
        ShapeBundle {
            path,
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 5.0),
                ..default()
            },
            ..default()
        },
        Stroke::new(Color::RED, 2.0),
        Fill::color(Color::RED),
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn zoom(mut query: Query<&mut OrthographicProjection, With<Camera2d>>) {
    let mut projection = query.single_mut();
    projection.scale = 1.0;
}

fn camera_follow(
    mut cameras: Query<&mut Transform, With<Camera>>,
    players: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let player = players.single();
    let mut transform = cameras.single_mut();
    transform.translation = player.translation;
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
        Environment,
        mesh_to_collider(&coll_mesh),
        Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Friction::coefficient(1.0),
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            ..default()
        },
    ));
}
