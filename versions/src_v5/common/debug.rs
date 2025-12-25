use crate::common::constants::{ENEMY_SIZE, PLAYER_SIZE, TILE_SIZE};
use crate::enemy::Enemy;
use crate::player::Player;
use crate::world::chunk::ChunkWall;
use bevy::prelude::*;

#[derive(Resource)]
pub struct DebugOptions {
    pub show_collisions: bool,
    pub show_sprites: bool,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            show_collisions: false,
            show_sprites: true,
        }
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOptions>()
            .add_systems(Update, (toggle_debug, update_visibility, draw_collisions));
    }
}

fn toggle_debug(input: Res<ButtonInput<KeyCode>>, mut debug_options: ResMut<DebugOptions>) {
    if input.just_pressed(KeyCode::F1) {
        debug_options.show_collisions = !debug_options.show_collisions;
    }
    if input.just_pressed(KeyCode::F2) {
        debug_options.show_sprites = !debug_options.show_sprites;
    }
}

fn update_visibility(
    debug_options: Res<DebugOptions>,
    mut sprite_query: Query<&mut Visibility, With<Sprite>>,
) {
    if debug_options.is_changed() {
        for mut visibility in sprite_query.iter_mut() {
            *visibility = if debug_options.show_sprites {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn draw_collisions(
    debug_options: Res<DebugOptions>,
    mut gizmos: Gizmos,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    wall_query: Query<&Transform, With<ChunkWall>>,
) {
    if !debug_options.show_collisions {
        return;
    }

    // Draw player collision
    for transform in player_query.iter() {
        gizmos.rect_2d(
            transform.translation.xy(),
            Vec2::splat(PLAYER_SIZE),
            Color::srgb(0.0, 1.0, 0.0),
        );
    }

    // Draw enemy collisions
    for transform in enemy_query.iter() {
        gizmos.rect_2d(
            transform.translation.xy(),
            Vec2::splat(ENEMY_SIZE),
            Color::srgb(1.0, 0.0, 0.0),
        );
    }

    // Draw wall collisions
    for transform in wall_query.iter() {
        gizmos.rect_2d(
            transform.translation.xy(),
            Vec2::splat(TILE_SIZE),
            Color::srgb(0.0, 0.0, 1.0),
        );
    }
}
