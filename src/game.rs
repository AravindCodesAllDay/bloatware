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
    player_query: Query<(Entity, &Transform), With<crate::player::Player>>,
    player_transform_query: Query<&Transform, With<crate::player::Player>>,
    target_query: Query<Entity, With<crate::target::Target>>,
    projectile_query: Query<Entity, With<crate::projectile::Projectile>>,
) {
    if input.just_pressed(KeyCode::Space) {
        match game.state {
            GameState::Menu => {
                game.state = GameState::Playing;
                game.score = 0;
                game.timer = 0.0;
                crate::player::spawn_player(&mut commands);
                crate::target::spawn_target(&mut commands, &player_transform_query);
            }
            GameState::GameOver => {
                // Clean up
                for (entity, _) in player_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in target_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in projectile_query.iter() {
                    commands.entity(entity).despawn();
                }

                game.state = GameState::Playing;
                game.score = 0;
                game.timer = 0.0;
                crate::player::spawn_player(&mut commands);
                crate::target::spawn_target(&mut commands, &player_transform_query);
            }
            _ => {}
        }
    }
}
