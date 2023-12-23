use bevy::{
    math::{Vec2, Vec3},
    prelude::*,
    render::mesh::Mesh,
    sprite::MaterialMesh2dBundle,
};

use crate::{
    bullet::{bullet_mesh, Bullet},
    player::Player,
};

#[derive(Component, Debug)]
pub struct Turret {
    pub acc: f32,
    pub fire_rate: f32,
    pub rot_speed : f32,
}

pub fn turret_system(
    time: Res<Time<Virtual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut turrets: Query<(&mut Transform, &mut Turret)>,
    mut player: Query<&mut Transform, (With<Player>, Without<Turret>)>,
    mut commands: Commands,
) {
    let player = player.single_mut();
    turrets.for_each_mut(|(mut turret_transform, mut turret)| {
        let diff: Vec2 = (player.translation - turret_transform.translation)
            .xy()
            .into();
        turret_transform.rotation =
            Quat::from_axis_angle(Vec3::new(0., 0., 1.), diff.y.atan2(diff.x));
        turret.acc += time.delta_seconds();
        if turret.acc > turret.fire_rate {
            turret.acc = 0.0;
            let bullet = Bullet {
                dmg: 0.1,
                dir: diff.normalize(),
                speed: 200.0,
                radius: 6.,
            };
            let mesh = bullet_mesh();
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(mesh).into(),
                    material: materials.add(ColorMaterial::from(Color::RED)),
                    transform: turret_transform.clone(),
                    ..default()
                },
                bullet,
            ));
        }
    });
}
