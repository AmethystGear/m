use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::{
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::RapierContext,
};

use crate::{bullet::bullet_mesh, mouse::MouseWorldCoords};

#[derive(Debug)]
pub struct Bounce {
    pt: Vec2,
    dir: Vec2,
}

#[derive(Component, Debug)]
pub struct Player {
    pub speed: f32,
    pub dir: Vec2,
    pub radius: f32,
    pub bullet_time: bool,
    pub bounce: Option<Bounce>,
    pub max_bullet_time_dist : f32,
    pub min_bullet_time_dist : f32,
    pub bounces_since_bullet_time : usize,
    pub health : f32,
}

pub fn player_control(
    mut time: ResMut<Time<Virtual>>,
    mouse: Res<Input<MouseButton>>,
    mouse_coords: Res<MouseWorldCoords>,
    rapier_context: Res<RapierContext>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut line: Query<(&mut Transform, &mut Path), Without<Player>>,
) {
    let (mut transform, mut player) = player.single_mut();
    transform.rotation = Quat::from_axis_angle(
        Vec3::new(0., 0., 1.),
        player.dir.y.atan2(player.dir.x) - std::f32::consts::PI / 2.0,
    );

    let position = Vec2::new(transform.translation.x, transform.translation.y);

    let shape_cast = rapier_context.cast_shape(
        position,
        0.0,
        player.dir.normalize(),
        &Collider::ball(player.radius),
        f32::MAX,
        false,
        QueryFilter::only_fixed(),
    );

    if let Some(bounce) = &player.bounce {
        let (mut loc, mut line) = line.single_mut();
        loc.translation = Vec3::new(bounce.pt.x, bounce.pt.y, 5.0);
        let mut path_builder = PathBuilder::new();
        if player.bullet_time {
            path_builder.move_to(Vec2::ZERO);
            path_builder.line_to(bounce.dir.normalize() * 25.0);
            path_builder.close();
        }
        path_builder.close();
        *line = path_builder.build();


        if player.dir.dot(bounce.pt - position) < 0.0 {
            transform.translation = Vec3::new(bounce.pt.x, bounce.pt.y, 0.0);
            player.dir = bounce.dir;
            player.bounce = None;
            let delta = player.dir.normalize() * player.speed * time.delta_seconds();
            transform.translation += Vec3::new(delta.x, delta.y, 0.0);

            if player.bullet_time {
                player.bullet_time = false;
                player.bounces_since_bullet_time = 0;
                time.set_relative_speed(1.0);
            } else {
                player.bounces_since_bullet_time += 1;
            }
            return
        }
    } else {
        if let Some((_, ray_intersection)) = &shape_cast {
            if let Some(details) = &ray_intersection.details {
                player.bounce = Some(Bounce {
                    pt: position + ray_intersection.toi * player.dir.normalize(),
                    dir: get_bounce_vector(player.dir, details.normal2),
                });
                return
            } else {
                if let Some((_, ray_intersection)) = rapier_context.cast_ray_and_get_normal(
                    position,
                    player.dir.normalize(),
                    f32::MAX,
                    false,
                    QueryFilter::only_fixed(),
                ) {
                    player.bounce = Some(Bounce {
                        pt: position + ray_intersection.toi * player.dir.normalize(),
                        dir: get_bounce_vector(player.dir, ray_intersection.normal),
                    });
                    return
                } else {
                    panic!("UH OH! CAN'T FIND BOUNCE!")
                }
            }
        }
    }

    if player.bullet_time {
        if let Some(bounce) = &mut player.bounce {
            let dir = (mouse_coords.0 - bounce.pt).normalize();
            if rapier_context.cast_ray(
                bounce.pt,
                dir,
                25.0,
                true,
                QueryFilter::only_fixed(),
            ).is_none() {
                bounce.dir = dir;
            }
        }
        if mouse.just_pressed(MouseButton::Left) {
            player.bullet_time = false;
            player.bounces_since_bullet_time = 0;
            time.set_relative_speed(1.0);
        }
    } else {
        if let Some(bounce) = &mut player.bounce {
            let dist = bounce.pt.distance(transform.translation.xy().into());
            if dist < player.max_bullet_time_dist && dist > player.min_bullet_time_dist && player.bounces_since_bullet_time > 2 {
                player.bullet_time = true;
                time.set_relative_speed(0.005);
            }
        }
    }

    let delta = player.dir.normalize() * player.speed * time.delta_seconds();
    transform.translation += Vec3::new(delta.x, delta.y, 0.0);


}

impl Player {
    pub fn new() -> Self {
        Self {
            speed: 900.0,
            bullet_time: false,
            max_bullet_time_dist : 200.0,
            min_bullet_time_dist : 50.0,
            bounce: None,
            dir: Vec2::new(1.0, -0.5),
            radius: 5.,
            bounces_since_bullet_time : 0,
            health : 2.0
        }
    }
}

pub fn get_bounce_vector(dir: Vec2, normal: Vec2) -> Vec2 {
    dir - 2.0 * dir.dot(normal) * normal
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player = Player::new();
    let mesh = bullet_mesh();
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            transform: Transform::from_translation(Vec3::new(10.0, -10.0, 1.0)),
            ..default()
        },
        player,
    ));
}
