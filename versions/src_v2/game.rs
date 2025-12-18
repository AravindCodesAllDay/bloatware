// ============================================================================
// game.rs
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct Game {
    pub state: GameState,
    pub score: u32,
    pub timer: f32,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Game {
            state: GameState::Menu,
            score: 0,
            timer: 0.0,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (update_timer, handle_state_transitions));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_timer(mut game: ResMut<Game>, time: Res<Time>) {
    if game.state == GameState::Playing {
        game.timer += time.delta_secs();
    }
}

fn handle_state_transitions(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    player_query: Query<Entity, With<crate::player::Player>>,
    enemy_query: Query<Entity, With<crate::enemy::Enemy>>,
    projectile_query: Query<Entity, With<crate::projectile::Projectile>>,
    player_transform_query: Query<&Transform, With<crate::player::Player>>,
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
            for e in enemy_query.iter() {
                commands.entity(e).despawn();
            }
            for e in projectile_query.iter() {
                commands.entity(e).despawn();
            }

            // ---- RESET GAME ----
            game.state = GameState::Playing;
            game.score = 0;
            game.timer = 0.0;

            // ---- SPAWN ----
            crate::player::spawn_player(&mut commands);
            crate::enemy::spawn_enemy(&mut commands, &player_transform_query);
        }
        _ => {}
    }
}
