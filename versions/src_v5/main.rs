// main.rs
use bevy::prelude::{App, DefaultPlugins, PluginGroup, Window, WindowPlugin};

mod audio;
mod common;
mod enemy;
mod game;
mod player;
mod projectile;
mod ui;
mod world;

use audio::AudioPlugin;
use common::collision::CollisionPlugin;
use common::debug::DebugPlugin;
use enemy::EnemyPlugin;
use game::GamePlugin;
use player::PlayerPlugin;
use projectile::ProjectilePlugin;
use ui::UIPlugin;
use world::chunk::ChunkPlugin;

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
            ChunkPlugin,
            PlayerPlugin,
            EnemyPlugin,
            ProjectilePlugin,
            UIPlugin,
            CollisionPlugin,
            DebugPlugin,
        ))
        .run();
}
