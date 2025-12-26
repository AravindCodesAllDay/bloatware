// ============================================================================
// constants.rs
use bevy::prelude::Color;

pub const PLAYER_SPEED: f32 = 300.0;
pub const ENEMY_SPEED: f32 = 180.0;
pub const ENEMY_TRACK_RANGE: f32 = 400.0;
pub const PROJECTILE_SPEED: f32 = 700.0;
pub const PROJECTILE_MAX_DISTANCE: f32 = 600.0;
pub const DASH_DISTANCE: f32 = 200.0;
pub const DASH_DURATION: f32 = 0.2;
pub const SHOOT_COOLDOWN: f32 = 0.3;
pub const DASH_COOLDOWN: f32 = 0.6;
pub const STEP_INTERVAL: f32 = 0.35;

pub const PLAYER_SIZE: f32 = 35.0; // Smaller than tile size
pub const PROJECTILE_RADIUS: f32 = 7.5;
pub const TILE_SIZE: f32 = 50.0; // Larger tiles for easier navigation
pub const ENEMY_SIZE: f32 = 35.0; // Also smaller than tiles

// Chunk system constants
pub const CHUNK_SIZE: usize = 32; // 16x16 tiles per chunk (more manageable)
pub const CHUNK_LOAD_RADIUS: i32 = 2; // Load chunks 2 chunks away from player
pub const CHUNK_UNLOAD_BUFFER: i32 = 1; // Keep chunks 1 extra chunk before unloading
pub const MIN_ENEMIES_PER_CHUNK: u32 = 1; // Minimum enemies per chunk
pub const MAX_ENEMIES_PER_CHUNK: u32 = 2; // Maximum enemies per chunk
pub const WALL_DENSITY: f32 = 0.01; // % chance for random walls

// Colors
pub const PROJECTILE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
