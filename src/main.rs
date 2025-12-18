// main.rs
use bevy::prelude::*;

mod audio;
mod collision;
mod constants;
mod game;
mod level;
mod player;
mod projectile;
mod target;
mod ui;

use audio::AudioPlugin;
use collision::CollisionPlugin;
use game::GamePlugin;
use level::LevelPlugin;
use player::PlayerPlugin;
use projectile::ProjectilePlugin;
use target::TargetPlugin;
use ui::UIPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bloatware".into(),
                resolution: (920, 600).into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((
            GamePlugin,
            AudioPlugin,
            LevelPlugin,
            PlayerPlugin,
            ProjectilePlugin,
            TargetPlugin,
            UIPlugin,
            CollisionPlugin,
        ))
        .run();
}