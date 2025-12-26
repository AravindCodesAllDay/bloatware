// ============================================================================
// game.rs
use crate::player::{spawn_player, Player};
use crate::projectile::Projectile;
use crate::world::chunk::{ChunkEntity, WorldGen};
use bevy::prelude::{
    App, AssetServer, Assets, ButtonInput, Camera2d, Commands, Component, Entity, KeyCode, Plugin,
    Query, Res, ResMut, Resource, Startup, TextureAtlasLayout, Time, Transform, Update, Vec3, With,
    Without,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Resource)]
pub struct Game {
    pub state: GameState,
    pub score: u32,
    pub timer: f32,
}

#[derive(Component)]
pub struct MainCamera;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Game {
            state: GameState::Menu,
            score: 0,
            timer: 0.0,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (
                update_timer,
                handle_state_transitions,
                handle_pause,
                camera_follow,
            ),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation = camera_transform
        .translation
        .lerp(player_transform.translation, 0.1);
}

fn update_timer(mut game: ResMut<Game>, time: Res<Time>) {
    if game.state == GameState::Playing {
        game.timer += time.delta_secs();
    }
}

fn handle_pause(input: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>) {
    if !input.just_pressed(KeyCode::Escape) {
        return;
    }

    match game.state {
        GameState::Playing => {
            game.state = GameState::Paused;
        }
        GameState::Paused => {
            game.state = GameState::Playing;
        }
        _ => {}
    }
}

fn handle_state_transitions(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut world_gen: ResMut<WorldGen>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    player_query: Query<Entity, With<Player>>,
    chunk_query: Query<Entity, With<ChunkEntity>>,
    projectile_query: Query<Entity, With<Projectile>>,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !input.just_pressed(KeyCode::Space) {
        return;
    }

    match game.state {
        GameState::Menu | GameState::GameOver => {
            // ---- CLEAN UP ----
            for e in player_query.iter() {
                commands.entity(e).despawn();
            }
            for e in chunk_query.iter() {
                commands.entity(e).despawn();
            }
            for e in projectile_query.iter() {
                commands.entity(e).despawn();
            }

            world_gen.loaded_chunks.clear();
            world_gen.chunk_entities.clear();

            // ---- RESET GAME ----
            game.state = GameState::Playing;
            game.score = 0;
            game.timer = 0.0;

            if let Ok(mut camera_transform) = camera_query.single_mut() {
                camera_transform.translation = Vec3::ZERO;
            }

            // ---- SPAWN PLAYER ----
            spawn_player(&mut commands, asset_server, texture_atlas_layouts);
        }
        _ => {}
    }
}
