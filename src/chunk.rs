// ============================================================================
// chunk.rs
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::constants::*;
use crate::game::{Game, GameState};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

impl ChunkCoord {
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / (CHUNK_SIZE as f32 * TILE_SIZE)).floor() as i32,
            y: (pos.y / (CHUNK_SIZE as f32 * TILE_SIZE)).floor() as i32,
        }
    }

    pub fn world_position(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
            self.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
            0.0,
        )
    }
}

#[derive(Component)]
pub struct ChunkEntity {
    #[allow(dead_code)]
    pub coord: ChunkCoord,
}

#[derive(Component)]
pub struct ChunkWall;

#[derive(Component)]
pub struct ChunkEnemy {
    #[allow(dead_code)]
    pub chunk: ChunkCoord,
}

#[derive(Resource)]
pub struct WorldGen {
    pub seed: u64,
    pub loaded_chunks: HashSet<ChunkCoord>,
    pub chunk_entities: HashMap<ChunkCoord, Vec<Entity>>,
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldGen {
            seed: 12345,
            loaded_chunks: HashSet::new(),
            chunk_entities: HashMap::new(),
        })
        .add_systems(Update, (
            update_chunks,
            cleanup_far_chunks,
        ));
    }
}

fn update_chunks(
    mut commands: Commands,
    mut world_gen: ResMut<WorldGen>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation;
    let player_chunk = ChunkCoord::from_world_pos(player_pos);

    for dx in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
        for dy in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
            let chunk_coord = ChunkCoord {
                x: player_chunk.x + dx,
                y: player_chunk.y + dy,
            };

            if !world_gen.loaded_chunks.contains(&chunk_coord) {
                generate_chunk(&mut commands, &mut world_gen, chunk_coord);
            }
        }
    }
}

fn generate_chunk(
    commands: &mut Commands,
    world_gen: &mut WorldGen,
    coord: ChunkCoord,
) {
    let mut entities = Vec::new();
    let base_pos = coord.world_position();

    let chunk_seed = hash_chunk(coord, world_gen.seed);
    let mut rng = StdRng::seed_from_u64(chunk_seed);

    // Store wall positions for spawn validation
    let mut wall_positions = Vec::new();

    // Generate walls with better spacing
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let world_x = base_pos.x + (x as f32 * TILE_SIZE);
            let world_y = base_pos.y + (y as f32 * TILE_SIZE);

            if should_spawn_wall(&mut rng, coord, x, y) {
                wall_positions.push(Vec3::new(world_x, world_y, 0.0));
                
                let entity = commands.spawn((
                    Sprite {
                        color: WALL_COLOR,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                    Transform::from_translation(Vec3::new(world_x, world_y, 0.0)),
                    ChunkWall,
                    ChunkEntity { coord },
                )).id();
                entities.push(entity);
            }
        }
    }

    let num_enemies = rng.random_range(MIN_ENEMIES_PER_CHUNK..=MAX_ENEMIES_PER_CHUNK);
    
    // Spawn enemies in valid positions (not on walls)
    let mut spawn_attempts = 0;
    let mut spawned_enemies = 0;
    
    while spawned_enemies < num_enemies && spawn_attempts < 50 {
        spawn_attempts += 1;
        
        let x_offset = rng.random::<f32>() * (CHUNK_SIZE as f32 * TILE_SIZE);
        let y_offset = rng.random::<f32>() * (CHUNK_SIZE as f32 * TILE_SIZE);
        
        let spawn_pos = Vec3::new(
            base_pos.x + x_offset,
            base_pos.y + y_offset,
            0.0,
        );

        // Check if spawn position is valid (not too close to walls)
        if is_valid_spawn_position(spawn_pos, &wall_positions, ENEMY_SIZE) {
            let entity = commands.spawn((
                Sprite {
                    color: ENEMY_COLOR,
                    custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
                    ..Default::default()
                },
                Transform::from_translation(spawn_pos),
                crate::enemy::Enemy {
                    velocity: Vec3::ZERO,
                    chosen_direction: Vec3::ZERO,
                    path_timer: 0.0,
                },
                ChunkEnemy { chunk: coord },
                ChunkEntity { coord },
            )).id();
            entities.push(entity);
            spawned_enemies += 1;
        }
    }

    world_gen.loaded_chunks.insert(coord);
    world_gen.chunk_entities.insert(coord, entities);
}

fn cleanup_far_chunks(
    mut commands: Commands,
    mut world_gen: ResMut<WorldGen>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let player_chunk = ChunkCoord::from_world_pos(player_transform.translation);

    let mut chunks_to_remove = Vec::new();

    for chunk_coord in world_gen.loaded_chunks.iter() {
        let dx = (chunk_coord.x - player_chunk.x).abs();
        let dy = (chunk_coord.y - player_chunk.y).abs();

        if dx > CHUNK_LOAD_RADIUS + CHUNK_UNLOAD_BUFFER || dy > CHUNK_LOAD_RADIUS + CHUNK_UNLOAD_BUFFER {
            chunks_to_remove.push(*chunk_coord);
        }
    }

    for chunk_coord in chunks_to_remove {
        if let Some(entities) = world_gen.chunk_entities.remove(&chunk_coord) {
            for entity in entities {
                commands.entity(entity).despawn();
            }
        }
        world_gen.loaded_chunks.remove(&chunk_coord);
    }
}

fn hash_chunk(coord: ChunkCoord, world_seed: u64) -> u64 {
    let mut seed = world_seed;
    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(coord.x as u64);
    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(coord.y as u64);
    seed
}

// Check if a spawn position is valid (not overlapping walls)
fn is_valid_spawn_position(pos: Vec3, walls: &[Vec3], entity_size: f32) -> bool {
    let half_size = entity_size / 2.0;
    let wall_half = TILE_SIZE / 2.0;
    let min_distance = half_size + wall_half + 5.0; // 5.0 pixels buffer

    for wall_pos in walls {
        let distance = pos.distance(*wall_pos);
        if distance < min_distance {
            return false;
        }
    }
    true
}

// Improved wall generation - creates rooms and corridors
fn should_spawn_wall(rng: &mut StdRng, chunk: ChunkCoord, x: usize, y: usize) -> bool {
    // Never spawn walls in the center chunk (0,0) to ensure player spawn is clear
    if chunk.x == 0 && chunk.y == 0 {
        // Keep center area of spawn chunk clear
        let center_x = CHUNK_SIZE / 2;
        let center_y = CHUNK_SIZE / 2;
        let clear_radius = 3;
        
        if (x as i32 - center_x as i32).abs() <= clear_radius 
            && (y as i32 - center_y as i32).abs() <= clear_radius {
            return false;
        }
    }

    // Only spawn walls on chunk boundaries (every 4th chunk creates a border)
    if (chunk.x % 4 == 0 && x == 0) || (chunk.y % 4 == 0 && y == 0) {
        // But leave openings
        let opening_chance: f32 = rng.random();
        if opening_chance < 0.3 {
            return false;
        }
        return true;
    }

    // Create scattered obstacles (pillars/walls) inside chunks
    let noise_x = (chunk.x * CHUNK_SIZE as i32 + x as i32) as f32 * 0.15;
    let noise_y = (chunk.y * CHUNK_SIZE as i32 + y as i32) as f32 * 0.15;
    
    let noise_value = ((noise_x.sin() * noise_y.cos()).abs() * 10.0) % 1.0;
    
    // Create small wall clusters occasionally
    if noise_value > 0.85 {
        return true;
    }
    
    // Very low random wall density
    let random_chance: f32 = rng.random();
    random_chance < WALL_DENSITY
}