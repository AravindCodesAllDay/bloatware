// ============================================================================
// collision.rs
use bevy::prelude::*;
use crate::audio::GameAssets;
use crate::constants::*;
use crate::game::{Game, GameState};
use crate::chunk::{ChunkWall, ChunkEnemy};
use crate::player::Player;
use crate::projectile::Projectile;
use crate::enemy::Enemy;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            apply_player_movement,
            apply_enemy_movement,
            check_projectile_wall_collision,
            check_projectile_enemy_collision,
            check_player_enemy_collision,
        ));
    }
}

fn apply_player_movement(
    time: Res<Time>,
    mut player_q: Query<(&mut Transform, &Player)>,
    wall_q: Query<(&Transform, &Sprite), (With<ChunkWall>, Without<Player>)>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok((mut p_t, p)) = player_q.single_mut() else { return };
    
    if p.is_dashing {
        return;
    }

    if p.velocity == Vec3::ZERO {
        return;
    }

    let tentative_pos = p_t.translation + p.velocity * time.delta_secs();
    let p_rad = PLAYER_SIZE / 2.0;

    let mut final_pos = tentative_pos;

    for (w_t, w_s) in wall_q.iter() {
        let w_pos = w_t.translation;
        let w_size = w_s.custom_size.unwrap();
        let w_half = w_size / 2.0;

        let dx = (final_pos.x - w_pos.x).abs();
        let dy = (final_pos.y - w_pos.y).abs();

        if dx < (p_rad + w_half.x) && dy < (p_rad + w_half.y) {
            let ox = (p_rad + w_half.x) - dx;
            let oy = (p_rad + w_half.y) - dy;

            if ox < oy {
                final_pos.x += if final_pos.x < w_pos.x { -ox } else { ox };
            } else {
                final_pos.y += if final_pos.y < w_pos.y { -oy } else { oy };
            }
        }
    }

    p_t.translation = final_pos;
}

fn apply_enemy_movement(
    time: Res<Time>,
    mut enemy_q: Query<(&mut Transform, &Enemy)>,
    wall_q: Query<(&Transform, &Sprite), (With<ChunkWall>, Without<Enemy>)>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    for (mut e_t, enemy) in enemy_q.iter_mut() {
        if enemy.velocity == Vec3::ZERO {
            continue;
        }

        let tentative_pos = e_t.translation + enemy.velocity * time.delta_secs();
        let e_half = ENEMY_SIZE / 2.0;

        let mut final_pos = tentative_pos;

        for (w_t, w_s) in wall_q.iter() {
            let w_pos = w_t.translation;
            let w_size = w_s.custom_size.unwrap();
            let w_half = w_size / 2.0;

            let dx = (final_pos.x - w_pos.x).abs();
            let dy = (final_pos.y - w_pos.y).abs();

            if dx < (e_half + w_half.x) && dy < (e_half + w_half.y) {
                let ox = (e_half + w_half.x) - dx;
                let oy = (e_half + w_half.y) - dy;

                if ox < oy {
                    final_pos.x += if final_pos.x < w_pos.x { -ox } else { ox };
                } else {
                    final_pos.y += if final_pos.y < w_pos.y { -oy } else { oy };
                }
            }
        }

        e_t.translation = final_pos;
    }
}

fn check_projectile_wall_collision(
    mut commands: Commands,
    assets: Res<GameAssets>,
    proj_q: Query<(Entity, &Transform), With<Projectile>>,
    wall_q: Query<(&Transform, &Sprite), With<ChunkWall>>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    for (p_e, p_t) in proj_q.iter() {
        let p_pos = p_t.translation;
        let p_rad = PROJECTILE_RADIUS;

        for (w_t, w_s) in wall_q.iter() {
            let w_pos = w_t.translation;
            let w_size = w_s.custom_size.unwrap();
            let w_half = w_size / 2.0;

            let dx = (p_pos.x - w_pos.x).abs();
            let dy = (p_pos.y - w_pos.y).abs();

            if dx < (p_rad + w_half.x) && dy < (p_rad + w_half.y) {
                commands.entity(p_e).despawn();
                commands.spawn(AudioPlayer(assets.hit.clone()));
                break;
            }
        }
    }
}

fn check_projectile_enemy_collision(
    mut commands: Commands,
    assets: Res<GameAssets>,
    proj_q: Query<(Entity, &Transform), With<Projectile>>,
    enemy_q: Query<(Entity, &Transform, &ChunkEnemy), With<Enemy>>,
    mut game: ResMut<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    for (p_e, p_t) in proj_q.iter() {
        let p_pos = p_t.translation;
        let p_rad = PROJECTILE_RADIUS;

        for (e_e, e_t, _chunk_enemy) in enemy_q.iter() {
            let e_pos = e_t.translation;
            let e_half = ENEMY_SIZE / 2.0;

            let dx = (p_pos.x - e_pos.x).abs();
            let dy = (p_pos.y - e_pos.y).abs();

            if dx < (p_rad + e_half) && dy < (p_rad + e_half) {
                commands.entity(p_e).despawn();
                commands.entity(e_e).despawn();

                game.score += 1;

                commands.spawn(AudioPlayer(assets.hit.clone()));

                break;
            }
        }
    }
}

fn check_player_enemy_collision(
    mut game: ResMut<Game>,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok(player_t) = player_q.single() else { return };

    for enemy_t in enemy_q.iter() {
        let p_pos = player_t.translation;
        let e_pos = enemy_t.translation;

        let p_half = PLAYER_SIZE / 2.0;
        let e_half = ENEMY_SIZE / 2.0;

        let dx = (p_pos.x - e_pos.x).abs();
        let dy = (p_pos.y - e_pos.y).abs();

        if dx < (p_half + e_half) && dy < (p_half + e_half) {
            game.state = GameState::GameOver;
            break;
        }
    }
}