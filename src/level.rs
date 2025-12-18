// ============================================================================
// level.rs
use bevy::prelude::*;
use crate::constants::*;

#[derive(Component)]
pub struct Wall;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_walls);
    }
}

fn spawn_walls(mut commands: Commands) {
    let lines: Vec<&str> = LEVEL_MAP.lines().collect();
    let height = lines.len() as f32;
    let width = if let Some(line) = lines.first() {
        line.len() as f32
    } else {
        0.0
    };

    let offset_x = -(width * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let offset_y = (height * TILE_SIZE) / 2.0 - TILE_SIZE / 2.0;

    for (row, line) in lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == '#' {
                let x = offset_x + (col as f32 * TILE_SIZE);
                let y = offset_y - (row as f32 * TILE_SIZE);

                commands.spawn((
                    Sprite {
                        color: WALL_COLOR,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                    Transform::from_translation(Vec3::new(x, y, 0.0)),
                    Wall,
                ));
            }
        }
    }
}

pub fn get_valid_spawn_positions(player_pos: Vec3) -> Vec<Vec3> {
    let lines: Vec<&str> = LEVEL_MAP.lines().collect();
    let height = lines.len() as f32;
    let width = if let Some(line) = lines.first() {
        line.len() as f32
    } else {
        0.0
    };

    let offset_x = -(width * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0;
    let offset_y = (height * TILE_SIZE) / 2.0 - TILE_SIZE / 2.0;

    let mut valid_positions = Vec::new();

    for (row, line) in lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == '.' {
                let x = offset_x + (col as f32 * TILE_SIZE);
                let y = offset_y - (row as f32 * TILE_SIZE);
                let pos = Vec3::new(x, y, 0.0);
                
                if player_pos.distance(pos) >= MIN_SPAWN_DISTANCE {
                    valid_positions.push(pos);
                }
            }
        }
    }

    valid_positions
}