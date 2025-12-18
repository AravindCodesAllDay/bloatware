// ============================================================================
// target.rs
use bevy::prelude::*;
use crate::constants::*;
use crate::level::get_valid_spawn_positions;

#[derive(Component)]
pub struct Target;

pub fn spawn_target(commands: &mut Commands, player_query: &Query<&Transform, With<crate::player::Player>>) {
    use rand::Rng;
    let mut rng = rand::rng();

    // Get player position if available
    let player_pos = if let Ok(transform) = player_query.single() {
        transform.translation
    } else {
        Vec3::ZERO
    };

    let valid_positions = get_valid_spawn_positions(player_pos);
    
    let spawn_pos = if !valid_positions.is_empty() {
        valid_positions[rng.random_range(0..valid_positions.len())]
    } else {
        Vec3::new(100.0, 100.0, 0.0)
    };

    commands.spawn((
        Sprite {
            color: TARGET_COLOR,
            custom_size: Some(Vec2::new(TARGET_SIZE, TARGET_SIZE)),
            ..Default::default()
        },
        Transform::from_translation(spawn_pos),
        Target,
    ));
}

