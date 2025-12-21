// main.rs
use bevy::prelude::*;

mod audio;
mod common;
mod game;
mod world;
mod player;
mod projectile;
mod enemy;
mod ui;

use audio::AudioPlugin;
use common::collision::CollisionPlugin;
use game::GamePlugin;
use world::chunk::ChunkPlugin;
use player::PlayerPlugin;
use projectile::ProjectilePlugin;
use enemy::EnemyPlugin;
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
            ChunkPlugin,
            PlayerPlugin,
            EnemyPlugin,
            ProjectilePlugin,
            UIPlugin,
            CollisionPlugin,
        ))
        .run();
}