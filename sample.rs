use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

// --- Constants ---
const PLAYER_SPEED: f32 = 250.0;
const ENEMY_SPEED: f32 = 150.0;
const ENEMY_DETECTION_RANGE: f32 = 250.0;
const PROJECTILE_SPEED: f32 = 700.0;
const SLASH_DURATION: f32 = 0.1;
const DASH_DISTANCE: f32 = 180.0;
const DASH_COOLDOWN: f32 = 0.8;
const GRID_SIZE: f32 = 60.0; // Size of procedural wall blocks
const WORLD_SIZE: i32 = 20;   // Blocks in each direction

// --- Components & Resources ---

#[derive(Resource, Serialize, Deserialize)]
struct GameState {
    score: u32,
    seed: u64,
    has_dash: bool,
    has_gun: bool,
    is_playing: bool,
}

#[derive(Component)]
struct Player {
    facing: Vec3,
    dash_cooldown: f32,
    slash_cooldown: f32,
}

#[derive(Component)]
struct Enemy {
    mode: EnemyMode,
    timer: f32,
    wander_dir: Vec3,
}

enum EnemyMode { Prowl, Chase }

#[derive(Component)]
struct Slash(f32); // Timer

#[derive(Component)]
struct Pickup(Ability);

enum Ability { Dash, Gun }

#[derive(Component)]
struct MainCamera;

// --- Main App ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameState {
            score: 0,
            seed: rand::random(),
            has_dash: false,
            has_gun: false,
            is_playing: false,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            camera_follow,
            player_actions,
            enemy_ai,
            collision_logic,
            save_load_system,
            slash_cleanup,
        ))
        .run();
}

fn setup(mut commands: Commands, mut game: GameState) {
    commands.spawn((Camera2d, MainCamera));
    generate_world(&mut commands, game.seed);
    spawn_player(&mut commands);
}

// --- Procedural Generation ---

fn generate_world(commands: &mut Commands, seed: u64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    for x in -WORLD_SIZE..WORLD_SIZE {
        for y in -WORLD_SIZE..WORLD_SIZE {
            // Random walls based on seed (Minecraft-like noise simulation)
            if rng.gen_bool(0.15) && (x.abs() > 2 || y.abs() > 2) {
                commands.spawn((
                    Sprite {
                        color: Color::srgb(0.2, 0.2, 0.25),
                        custom_size: Some(Vec2::splat(GRID_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(x as f32 * GRID_SIZE, y as f32 * GRID_SIZE, 0.0),
                )).insert(Wall);
            }
            
            // Randomly spawn enemies
            if rng.gen_bool(0.02) && (x.abs() > 5 || y.abs() > 5) {
                spawn_enemy(commands, Vec3::new(x as f32 * GRID_SIZE, y as f32 * GRID_SIZE, 0.0));
            }
        }
    }

    // Spawn Pickups at specific distances
    spawn_pickup(commands, Vec3::new(400.0, 400.0, 0.0), Ability::Dash);
    spawn_pickup(commands, Vec3::new(-400.0, -400.0, 0.0), Ability::Gun);
}

// --- Systems ---

fn camera_follow(
    player_q: Query<&Transform, With<Player>>,
    mut cam_q: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    if let Ok(p_t) = player_q.get_single() {
        if let Ok(mut c_t) = cam_q.get_single_mut() {
            // Smoothly lerp camera to player
            c_t.translation = c_t.translation.lerp(p_t.translation, 0.1);
        }
    }
}

fn player_actions(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<GameState>,
    mut query: Query<(&mut Transform, &mut Player)>,
) {
    let Ok((mut trans, mut player)) = query.get_single_mut() else { return };
    
    // Movement
    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { move_dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { move_dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }

    if move_dir != Vec3::ZERO {
        move_dir = move_dir.normalize();
        player.facing = move_dir;
        trans.translation += move_dir * PLAYER_SPEED * time.delta_secs();
    }

    // Slash (Default Attack)
    if keys.just_pressed(KeyCode::KeyF) {
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 1.0, 1.0),
                custom_size: Some(Vec2::new(60.0, 20.0)),
                ..default()
            },
            Transform {
                translation: trans.translation + player.facing * 40.0,
                rotation: Quat::from_rotation_z(player.facing.y.atan2(player.facing.x)),
                ..default()
            },
            Slash(SLASH_DURATION),
        ));
    }

    // Dash (Unlockable)
    if game.has_dash && keys.just_pressed(KeyCode::ShiftLeft) && player.dash_cooldown <= 0.0 {
        trans.translation += player.facing * DASH_DISTANCE;
        player.dash_cooldown = DASH_COOLDOWN;
    }
    player.dash_cooldown -= time.delta_secs();
}

fn enemy_ai(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut Transform, &mut Enemy), Without<Player>>,
) {
    let Ok(p_t) = player_q.get_single() else { return };

    for (mut e_t, mut enemy) in enemies.iter_mut() {
        let dist = e_t.translation.distance(p_t.translation);

        if dist < ENEMY_DETECTION_RANGE {
            enemy.mode = EnemyMode::Chase;
            let dir = (p_t.translation - e_t.translation).normalize();
            e_t.translation += dir * ENEMY_SPEED * time.delta_secs();
        } else {
            // Prowl logic
            enemy.mode = EnemyMode::Prowl;
            enemy.timer -= time.delta_secs();
            if enemy.timer <= 0.0 {
                let mut rng = rand::thread_rng();
                enemy.wander_dir = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0).normalize();
                enemy.timer = 2.0;
            }
            e_t.translation += enemy.wander_dir * (ENEMY_SPEED * 0.4) * time.delta_secs();
        }
    }
}

// --- Save / Load Logic ---

fn save_load_system(keys: Res<ButtonInput<KeyCode>>, game: Res<GameState>) {
    if keys.just_pressed(KeyCode::KeyS) {
        let json = serde_json::to_string(&*game).unwrap();
        let mut file = File::create("save.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
        println!("Game Saved!");
    }

    if keys.just_pressed(KeyCode::KeyL) {
        if let Ok(mut file) = File::open("save.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let _loaded_game: GameState = serde_json::from_str(&contents).unwrap();
            println!("Game Loaded! (Requires App Restart to rebuild world from seed)");
            // Note: In a full implementation, you would trigger a world rebuild system here
        }
    }
}

// --- Helper Spawners ---

fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.7, 0.9),
            custom_size: Some(Vec2::splat(30.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player {
            facing: Vec3::Y,
            dash_cooldown: 0.0,
            slash_cooldown: 0.0,
        },
    ));
}

fn spawn_enemy(commands: &mut Commands, pos: Vec3) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.1, 0.1),
            custom_size: Some(Vec2::splat(30.0)),
            ..default()
        },
        Transform::from_translation(pos),
        Enemy {
            mode: EnemyMode::Prowl,
            timer: 0.0,
            wander_dir: Vec3::ZERO,
        },
    ));
}

fn spawn_pickup(commands: &mut Commands, pos: Vec3, ability: Ability) {
    let color = match ability {
        Ability::Dash => Color::srgb(0.1, 0.9, 0.1),
        Ability::Gun => Color::srgb(0.9, 0.9, 0.1),
    };
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(20.0)),
            ..default()
        },
        Transform::from_translation(pos),
        Pickup(ability),
    ));
}

fn slash_cleanup(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Slash)>) {
    for (entity, mut slash) in query.iter_mut() {
        slash.0 -= time.delta_secs();
        if slash.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// placeholder components for existing collision logic
#[derive(Component)] struct Wall;
fn collision_logic() { /* Implement AABB collision between Player/Wall and Slash/Enemy */ }