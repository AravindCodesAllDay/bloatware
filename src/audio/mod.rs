// ============================================================================
// audio.rs
use bevy::prelude::*;

#[derive(Resource)]
pub struct GameAssets {
    pub shoot: Handle<AudioSource>,
    pub dash: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub step: Handle<AudioSource>,
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_assets);
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        shoot: asset_server.load("shoot.ogg"),
        dash: asset_server.load("dash.ogg"),
        hit: asset_server.load("hit.ogg"),
        step: asset_server.load("step.ogg"),
    });
}