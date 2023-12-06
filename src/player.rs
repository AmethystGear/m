struct PlayerHealth {
    health : f64
}

#[derive(Component)]
struct Player {
    speed: f32,
    health : PlayerHealth
}
