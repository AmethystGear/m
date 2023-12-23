use bevy::{
    math::{Vec2, Vec3},
    prelude::*,
    render::{
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
    },
};
use bevy_rapier2d::{plugin::RapierContext, geometry::Collider, pipeline::QueryFilter};
use itertools::Itertools;

use crate::player::Player;

pub fn bullet_mesh() -> Mesh {
    let scale = 10.0;
    let vertices = vec![
        (-0.5, -1.0),
        (0.5, -1.0),
        (0.5, 0.25),
        (-0.5, 0.25),
        (-0.3, 0.9),
        (0.3, 0.9),
        (0.0, 1.2),
    ]
    .into_iter()
    .map(|v| v.into())
    .map(|v: Vec2| v * scale)
    .map(|v| Vec3::new(v.x, v.y, 0.0))
    .collect_vec();

    let indices = vec![0, 1, 2, 0, 2, 3, 3, 2, 5, 3, 5, 4, 4, 5, 6];
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh
}

#[derive(Component, Debug)]
pub struct Bullet {
    pub dmg: f32,
    pub dir: Vec2,
    pub speed: f32,
    pub radius: f32,
}

pub fn bullet_system(
    rapier_context: Res<RapierContext>,
    time: Res<Time<Virtual>>,
    mut bullets: Query<(Entity, &mut Transform, &mut Bullet)>,
    mut player: Query<(&mut Transform, &mut Player), Without<Bullet>>,
    mut commands: Commands,
) {
    let (player_transform, mut player) = player.single_mut();
    bullets.for_each_mut(|(bullet_entity, mut bullet_transform, bullet)| {
        let rad = bullet.radius + player.radius;
        if bullet_transform
            .translation
            .distance_squared(player_transform.translation)
            < rad * rad
        {
            player.health -= bullet.dmg;
            commands.entity(bullet_entity).despawn();
            return;
        }

        if rapier_context.intersection_with_shape(
            bullet_transform.translation.xy().into(),
            0.0,
            &Collider::ball(bullet.radius),
            QueryFilter::new(),
        ).is_some() {
            commands.entity(bullet_entity).despawn();
            return;
        }

        let delta = bullet.dir.normalize() * bullet.speed * time.delta_seconds();
        bullet_transform.translation += Vec3::new(delta.x, delta.y, 0.0);

        bullet_transform.rotation = Quat::from_axis_angle(
            Vec3::new(0., 0., 1.),
            bullet.dir.y.atan2(bullet.dir.x) - std::f32::consts::PI / 2.0,
        );
    });
}
