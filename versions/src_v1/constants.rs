// ============================================================================
// constants.rs
use bevy::prelude::*;

pub const PLAYER_SPEED: f32 = 300.0;
pub const PROJECTILE_SPEED: f32 = 700.0;
pub const PROJECTILE_MAX_DISTANCE: f32 = 600.0;
pub const DASH_DISTANCE: f32 = 200.0;
pub const DASH_DURATION: f32 = 0.2;
pub const SHOOT_COOLDOWN: f32 = 0.3;
pub const DASH_COOLDOWN: f32 = 0.6;
pub const STEP_INTERVAL: f32 = 0.35;

pub const PLAYER_SIZE: f32 = 50.0;
pub const PROJECTILE_RADIUS: f32 = 7.5;
pub const TILE_SIZE: f32 = 40.0;
pub const TARGET_SIZE: f32 = 40.0;
pub const MIN_SPAWN_DISTANCE: f32 = 150.0;

// Colors
pub const PLAYER_READY: Color = Color::srgb(0.9, 0.2, 0.2);
pub const PLAYER_EMPTY: Color = Color::srgb(0.2, 0.2, 0.2);
pub const GUN_READY: Color = Color::srgb(1.0, 0.8, 0.0);
pub const GUN_EMPTY: Color = Color::srgb(0.3, 0.3, 0.1);
pub const PROJECTILE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
pub const TARGET_COLOR: Color = Color::srgb(0.2, 0.8, 0.3);
pub const WALL_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

pub const LEVEL_MAP: &str = "\
#######################
#.....................#
#.....................#
#.....................#
#.....................#
#........###..........#
#........#............#
#........#............#
#........###..........#
#.....................#
#.....................#
#.....................#
#.....................#
#.....................#
#######################";