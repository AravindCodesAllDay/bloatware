// ============================================================================
// chunk.rs
use crate::common::constants::{
    CHUNK_LOAD_RADIUS, CHUNK_SIZE, CHUNK_UNLOAD_BUFFER, ENEMY_SIZE, MAX_ENEMIES_PER_CHUNK,
    MIN_ENEMIES_PER_CHUNK, TILE_SIZE, WALL_DENSITY,
};
use crate::enemy::Enemy;
use crate::game::{Game, GameState};
use crate::player::Player;
use bevy::prelude::{
    App, AssetServer, Commands, Component, Entity, Plugin, Query, Res, ResMut, Resource, Sprite,
    Transform, Update, Vec2, Vec3, With,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

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
        .add_systems(Update, (update_chunks, cleanup_far_chunks));
    }
}

fn update_chunks(
    mut commands: Commands,
    mut world_gen: ResMut<WorldGen>,
    player_query: Query<&Transform, With<Player>>,
    game: Res<Game>,
    asset_server: Res<AssetServer>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;
    let player_chunk = ChunkCoord::from_world_pos(player_pos);

    for dx in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
        for dy in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
            let chunk_coord = ChunkCoord {
                x: player_chunk.x + dx,
                y: player_chunk.y + dy,
            };

            if !world_gen.loaded_chunks.contains(&chunk_coord) {
                generate_chunk(&mut commands, &mut world_gen, chunk_coord, &asset_server);
            }
        }
    }
}

fn generate_chunk(
    commands: &mut Commands,
    world_gen: &mut WorldGen,
    coord: ChunkCoord,
    asset_server: &AssetServer,
) {
    let mut entities = Vec::new();
    let base_pos = coord.world_position();

    let chunk_seed = hash_chunk(coord, world_gen.seed);
    let mut rng = StdRng::seed_from_u64(chunk_seed);

    let wall_texture = asset_server.load("sprits/wall.png");
    let ground_texture = asset_server.load("sprits/ground.png");
    let enemy_texture = asset_server.load("sprits/enemy.png");

    let mut wall_positions = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let world_x = base_pos.x + (x as f32 * TILE_SIZE);
            let world_y = base_pos.y + (y as f32 * TILE_SIZE);

            // Ground
            let ground_entity = commands
                .spawn((
                    Sprite {
                        image: ground_texture.clone(),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                    Transform::from_translation(Vec3::new(world_x, world_y, -1.0)),
                    ChunkEntity { coord },
                ))
                .id();
            entities.push(ground_entity);

            if should_spawn_wall(&mut rng, coord, x, y) {
                wall_positions.push(Vec3::new(world_x, world_y, 0.0));

                let entity = commands
                    .spawn((
                        Sprite {
                            image: wall_texture.clone(),
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                        Transform::from_translation(Vec3::new(world_x, world_y, 0.0)),
                        ChunkWall,
                        ChunkEntity { coord },
                    ))
                    .id();
                entities.push(entity);
            }
        }
    }

    let num_enemies = rng.random_range(MIN_ENEMIES_PER_CHUNK..=MAX_ENEMIES_PER_CHUNK);
    let mut spawn_attempts = 0;
    let mut spawned_enemies = 0;

    while spawned_enemies < num_enemies && spawn_attempts < 50 {
        spawn_attempts += 1;
        let x_offset = rng.random::<f32>() * (CHUNK_SIZE as f32 * TILE_SIZE);
        let y_offset = rng.random::<f32>() * (CHUNK_SIZE as f32 * TILE_SIZE);
        let spawn_pos = Vec3::new(base_pos.x + x_offset, base_pos.y + y_offset, 0.0);

        if is_valid_spawn_position(spawn_pos, &wall_positions, ENEMY_SIZE) {
            let entity = commands
                .spawn((
                    Sprite {
                        image: enemy_texture.clone(),
                        custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
                        ..Default::default()
                    },
                    Transform::from_translation(spawn_pos),
                    Enemy {
                        velocity: Vec3::ZERO,
                        chosen_direction: Vec3::ZERO,
                        path_timer: 0.0,
                    },
                    ChunkEnemy { chunk: coord },
                    ChunkEntity { coord },
                ))
                .id();
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
    player_query: Query<&Transform, With<Player>>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_chunk = ChunkCoord::from_world_pos(player_transform.translation);

    let mut chunks_to_remove = Vec::new();

    for chunk_coord in world_gen.loaded_chunks.iter() {
        let dx = (chunk_coord.x - player_chunk.x).abs();
        let dy = (chunk_coord.y - player_chunk.y).abs();

        if dx > CHUNK_LOAD_RADIUS + CHUNK_UNLOAD_BUFFER
            || dy > CHUNK_LOAD_RADIUS + CHUNK_UNLOAD_BUFFER
        {
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
    seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(coord.x as u64);
    seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(coord.y as u64);
    seed
}

fn is_valid_spawn_position(pos: Vec3, walls: &[Vec3], entity_size: f32) -> bool {
    let half_size = entity_size / 2.0;
    let wall_half = TILE_SIZE / 2.0;
    let min_distance = half_size + wall_half + 5.0;

    for wall_pos in walls {
        let distance = pos.distance(*wall_pos);
        if distance < min_distance {
            return false;
        }
    }
    true
}

fn should_spawn_wall(rng: &mut StdRng, chunk: ChunkCoord, x: usize, y: usize) -> bool {
    if chunk.x == 0 && chunk.y == 0 {
        let center_x = CHUNK_SIZE / 2;
        let center_y = CHUNK_SIZE / 2;
        let clear_radius = 3;
        if (x as i32 - center_x as i32).abs() <= clear_radius
            && (y as i32 - center_y as i32).abs() <= clear_radius
        {
            return false;
        }
    }

    if (chunk.x % 4 == 0 && x == 0) || (chunk.y % 4 == 0 && y == 0) {
        let opening_chance: f32 = rng.random();
        if opening_chance < 0.3 {
            return false;
        }
        return true;
    }

    let noise_x = (chunk.x * CHUNK_SIZE as i32 + x as i32) as f32 * 0.15;
    let noise_y = (chunk.y * CHUNK_SIZE as i32 + y as i32) as f32 * 0.15;
    let noise_value = ((noise_x.sin() * noise_y.cos()).abs() * 10.0) % 1.0;

    if noise_value > 0.85 {
        return true;
    }
    let random_chance: f32 = rng.random();
    random_chance < WALL_DENSITY
}
