// ============================================================================
// enemy.rs
use bevy::prelude::*;
use crate::common::constants::*;
use crate::game::{Game, GameState};

#[derive(Component)]
pub struct Enemy {
    pub velocity: Vec3,
    pub chosen_direction: Vec3,
    pub path_timer: f32,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, enemy_ai);
    }
}

fn enemy_ai(
    time: Res<Time>,
    mut enemy_q: Query<(&Transform, &mut Enemy)>,
    player_q: Query<&Transform, With<crate::player::Player>>,
    wall_q: Query<(&Transform, &Sprite), With<crate::world::chunk::ChunkWall>>,
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

        if distance >= ENEMY_TRACK_RANGE {
            enemy.velocity = Vec3::ZERO;
            enemy.path_timer = 0.0;
            enemy.chosen_direction = Vec3::ZERO;
            continue;
        }

        // Update path timer
        enemy.path_timer -= time.delta_secs();

        // Only recalculate path if timer expired or no direction set
        if enemy.path_timer <= 0.0 || enemy.chosen_direction == Vec3::ZERO {
            // Direct path to player
            let direct_direction = (player_pos - enemy_pos).normalize();
            
            // Check if there's a wall blocking the direct path
            let lookahead_distance = 50.0;
            let check_pos = enemy_pos + direct_direction * lookahead_distance;
            
            let mut wall_blocking = false;
            let enemy_radius = ENEMY_SIZE / 2.0;
            
            // Check for walls in the path ahead
            for (wall_t, _wall_s) in wall_q.iter() {
                let wall_pos = wall_t.translation;
                let dist_to_wall = check_pos.distance(wall_pos);
                
                if dist_to_wall < (TILE_SIZE / 2.0 + enemy_radius + 10.0) {
                    wall_blocking = true;
                    break;
                }
            }
            
            let mut final_direction = direct_direction;
            
            // If wall is blocking, try moving around it
            if wall_blocking {
                // Try perpendicular directions
                let perp_right = Vec3::new(-direct_direction.y, direct_direction.x, 0.0);
                let perp_left = Vec3::new(direct_direction.y, -direct_direction.x, 0.0);
                
                // Calculate potential positions for both directions
                let right_check = enemy_pos + perp_right * lookahead_distance;
                let left_check = enemy_pos + perp_left * lookahead_distance;
                
                let mut right_blocked = false;
                let mut left_blocked = false;
                
                for (wall_t, _wall_s) in wall_q.iter() {
                    let wall_pos = wall_t.translation;
                    
                    if right_check.distance(wall_pos) < (TILE_SIZE / 2.0 + enemy_radius + 10.0) {
                        right_blocked = true;
                    }
                    if left_check.distance(wall_pos) < (TILE_SIZE / 2.0 + enemy_radius + 10.0) {
                        left_blocked = true;
                    }
                    
                    if right_blocked && left_blocked {
                        break;
                    }
                }
                
                // Choose direction based on which gets closer to player
                if !right_blocked && !left_blocked {
                    // Both clear - pick the one that gets us closer to player
                    let right_pos = enemy_pos + perp_right * lookahead_distance;
                    let left_pos = enemy_pos + perp_left * lookahead_distance;
                    
                    let right_dist_to_player = right_pos.distance(player_pos);
                    let left_dist_to_player = left_pos.distance(player_pos);
                    
                    // If distances are very similar (within 5 pixels), consistently pick right
                    if (right_dist_to_player - left_dist_to_player).abs() < 5.0 {
                        final_direction = (direct_direction * 0.3 + perp_right * 0.7).normalize();
                    } else if right_dist_to_player < left_dist_to_player {
                        final_direction = (direct_direction * 0.3 + perp_right * 0.7).normalize();
                    } else {
                        final_direction = (direct_direction * 0.3 + perp_left * 0.7).normalize();
                    }
                } else if !right_blocked {
                    final_direction = (direct_direction * 0.3 + perp_right * 0.7).normalize();
                } else if !left_blocked {
                    final_direction = (direct_direction * 0.3 + perp_left * 0.7).normalize();
                } else {
                    // Both blocked, try to back up
                    final_direction = -direct_direction * 0.5;
                }
            }

            // Commit to this direction for a short time
            enemy.chosen_direction = final_direction;
            enemy.path_timer = 0.3; // Re-evaluate path every 0.3 seconds
        }

        // Use the committed direction
        enemy.velocity = enemy.chosen_direction * ENEMY_SPEED;
    }
}