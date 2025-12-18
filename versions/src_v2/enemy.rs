// ============================================================================
// enemy.rs
use bevy::prelude::*;
use crate::constants::*;
use crate::game::{Game, GameState};
use crate::level::get_valid_spawn_positions;

#[derive(Component)]
pub struct Enemy {
    pub velocity: Vec3,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, enemy_ai);
    }
}

pub fn spawn_enemy(commands: &mut Commands, player_query: &Query<&Transform, With<crate::player::Player>>) {
    use rand::Rng;
    let mut rng = rand::rng();

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
            color: ENEMY_COLOR,
            custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
            ..Default::default()
        },
        Transform::from_translation(spawn_pos),
        Enemy {
            velocity: Vec3::ZERO,
        },
    ));
}

fn enemy_ai(
    mut enemy_q: Query<(&Transform, &mut Enemy)>,
    player_q: Query<&Transform, With<crate::player::Player>>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok(player_t) = player_q.single() else { return };
    let player_pos = player_t.translation;

    for (enemy_t, mut enemy) in enemy_q.iter_mut() {
        let enemy_pos = enemy_t.translation;
        let distance = enemy_pos.distance(player_pos);

        // Track player if within range
        if distance < ENEMY_TRACK_RANGE {
            let direction = (player_pos - enemy_pos).normalize();
            enemy.velocity = direction * ENEMY_SPEED;
        } else {
            enemy.velocity = Vec3::ZERO;
        }
    }
}