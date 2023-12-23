use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Resource, Default)]
pub struct MouseWorldCoords(pub Vec2);

pub fn mouse_world_coords(
    mut mouse_coords: ResMut<MouseWorldCoords>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mouse_coords.0 = world_position;
    }
}
