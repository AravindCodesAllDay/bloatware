// ============================================================================
// ui.rs
use bevy::prelude::*;
use crate::game::{Game, GameState};

#[derive(Component)]
pub struct UIText;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui)
            .add_systems(Update, update_ui);
    }
}

fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        UIText,
    ));
}

fn update_ui(game: Res<Game>, mut query: Query<&mut Text, With<UIText>>) {
    for mut text in query.iter_mut() {
        let minutes = (game.timer as u32) / 60;
        let seconds = (game.timer as u32) % 60;
        let millis = ((game.timer % 1.0) * 100.0) as u32;

        match game.state {
            GameState::Menu => {
                text.0 = "BLOATWARE\n\nPress SPACE to Start".to_string();
            }
            GameState::Playing => {
                text.0 = format!(
                    "Score: {}\nTime: {:02}:{:02}.{:02}",
                    game.score, minutes, seconds, millis
                );
            }
            GameState::GameOver => {
                text.0 = format!(
                    "GAME OVER!\n\nScore: {}\nTime: {:02}:{:02}.{:02}\n\nPress SPACE to Restart",
                    game.score, minutes, seconds, millis
                );
            }
        }
    }
}