use crate::audio::GameAssets;
use crate::common::constants::{
    DASH_COOLDOWN, DASH_DISTANCE, DASH_DURATION, PLAYER_SIZE, PLAYER_SPEED, PROJECTILE_COLOR,
    PROJECTILE_RADIUS, SHOOT_COOLDOWN, STEP_INTERVAL,
};
use crate::game::{Game, GameState};
use crate::projectile::Projectile;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub last_direction: Vec3,
    pub shoot_cooldown: f32,
    pub dash_cooldown: f32,
    pub is_dashing: bool,
    pub dash_timer: f32,
    pub dash_direction: Vec3,
    pub dash_start_pos: Vec3,
    pub step_timer: f32,
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub enum PlayerLayer {
    Base,
    DashOverlay,
    IdleOverlay,
    DashTimer,
    ReloadTimer,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_input,
                dash_handler,
                animate_player,
                update_player_timers,
            ),
        );
    }
}

pub fn spawn_player(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("sprits/player.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(500), 6, 6, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    commands
        .spawn((
            Sprite {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: layout_handle.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            Player {
                last_direction: Vec3::Y,
                shoot_cooldown: 0.0,
                dash_cooldown: 0.0,
                is_dashing: false,
                dash_timer: 0.0,
                dash_direction: Vec3::ZERO,
                dash_start_pos: Vec3::ZERO,
                step_timer: 0.0,
                velocity: Vec3::ZERO,
            },
            PlayerLayer::Base,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        ))
        .with_children(|parent| {
            // Dash Overlay (Row 3, Index 12)
            parent.spawn((
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: layout_handle.clone(),
                        index: 12,
                    }),
                    custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                    ..Default::default()
                },
                Visibility::Hidden,
                Transform::from_xyz(0.0, 0.0, 0.1),
                PlayerLayer::DashOverlay,
                AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            ));

            // Idle Overlay (Row 4, Index 18)
            parent.spawn((
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: layout_handle.clone(),
                        index: 18,
                    }),
                    custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                    ..Default::default()
                },
                Visibility::Hidden,
                Transform::from_xyz(0.0, 0.0, 0.2),
                PlayerLayer::IdleOverlay,
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ));

            // Dash Timer Overlay (Row 5, Index 24)
            parent.spawn((
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: layout_handle.clone(),
                        index: 24,
                    }),
                    custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 0.0, 0.3),
                PlayerLayer::DashTimer,
            ));

            // Reload Timer Overlay (Row 6, Index 30)
            parent.spawn((
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: layout_handle,
                        index: 30,
                    }),
                    custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 0.0, 0.4),
                PlayerLayer::ReloadTimer,
            ));
        });
}

fn player_input(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Player)>,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok((transform, mut player)) = query.single_mut() else {
        return;
    };

    if player.is_dashing {
        player.velocity = Vec3::ZERO;
        return;
    }

    player.shoot_cooldown = (player.shoot_cooldown - time.delta_secs()).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - time.delta_secs()).max(0.0);

    let mut dir = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    if dir != Vec3::ZERO {
        dir = dir.normalize();
        player.last_direction = dir;
        player.velocity = dir * PLAYER_SPEED;

        player.step_timer -= time.delta_secs();
        if player.step_timer <= 0.0 {
            commands.spawn(AudioPlayer(assets.step.clone()));
            player.step_timer = STEP_INTERVAL;
        }
    } else {
        player.velocity = Vec3::ZERO;
        player.step_timer = STEP_INTERVAL;
    }

    if player.dash_cooldown <= 0.0
        && (input.just_pressed(KeyCode::ShiftLeft) || input.just_pressed(KeyCode::ShiftRight))
    {
        player.is_dashing = true;
        player.dash_timer = DASH_DURATION;
        player.dash_direction = if dir != Vec3::ZERO {
            dir
        } else {
            player.last_direction
        };
        player.dash_start_pos = transform.translation;
        player.dash_cooldown = DASH_COOLDOWN;

        commands.spawn(AudioPlayer(assets.dash.clone()));
    }

    if player.shoot_cooldown <= 0.0 && input.just_pressed(KeyCode::Space) {
        let shoot_dir = if dir != Vec3::ZERO {
            dir
        } else {
            player.last_direction
        };

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PROJECTILE_RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from(PROJECTILE_COLOR))),
            Transform::from_translation(transform.translation),
            Projectile {
                direction: shoot_dir,
                distance_traveled: 0.0,
            },
        ));

        commands.spawn(AudioPlayer(assets.shoot.clone()));
        player.shoot_cooldown = SHOOT_COOLDOWN;
    }
}

fn dash_handler(time: Res<Time>, mut query: Query<(&mut Transform, &mut Player)>, game: Res<Game>) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok((mut transform, mut player)) = query.single_mut() else {
        return;
    };
    if !player.is_dashing {
        return;
    }

    player.dash_timer -= time.delta_secs();

    if player.dash_timer <= 0.0 {
        player.is_dashing = false;
        transform.translation = player.dash_start_pos + player.dash_direction * DASH_DISTANCE;
    } else {
        let t = 1.0 - (player.dash_timer / DASH_DURATION);
        let target = player.dash_start_pos + player.dash_direction * DASH_DISTANCE;
        transform.translation = player.dash_start_pos.lerp(target, t);
    }
}

fn animate_player(
    time: Res<Time>,
    mut player_query: Query<
        (&Player, &mut Sprite, &mut Transform, &mut AnimationTimer),
        With<PlayerLayer>,
    >,
    mut overlay_query: Query<
        (
            &mut Sprite,
            &mut AnimationTimer,
            &mut Visibility,
            &PlayerLayer,
        ),
        Without<Player>,
    >,
    player_parent_query: Query<&Children, With<Player>>,
) {
    let Ok((player, mut base_sprite, mut base_transform, mut base_timer)) =
        player_query.single_mut()
    else {
        return;
    };
    let Ok(children) = player_parent_query.single() else {
        return;
    };

    // Base Player and Movement Animation
    base_timer.0.tick(time.delta());

    let is_moving = player.velocity.length() > 0.1;
    let mut rotation = 0.0;

    if is_moving {
        // Row 2 movement
        if base_timer.0.just_finished() {
            if let Some(atlas) = &mut base_sprite.texture_atlas {
                atlas.index = if atlas.index == 6 { 7 } else { 6 };
            }
        }

        // Handle rotation
        let dir = player.velocity.normalize();
        if dir.y > 0.5 {
            // Up
            rotation = 0.0;
            if dir.x > 0.5 {
                rotation = -std::f32::consts::FRAC_PI_4;
            } else if dir.x < -0.5 {
                rotation = std::f32::consts::FRAC_PI_4;
            }
        } else if dir.y < -0.5 {
            // Down
            rotation = std::f32::consts::PI;
            if dir.x > 0.5 {
                rotation = std::f32::consts::PI + std::f32::consts::FRAC_PI_4;
            } else if dir.x < -0.5 {
                rotation = std::f32::consts::PI - std::f32::consts::FRAC_PI_4;
            }
        } else if dir.x > 0.5 {
            // Right
            rotation = -std::f32::consts::FRAC_PI_2;
        } else if dir.x < -0.5 {
            // Left
            rotation = std::f32::consts::FRAC_PI_2;
        }

        // If ordinal (top-left etc), use frame 7
        if dir.x.abs() > 0.5 && dir.y.abs() > 0.5 {
            if let Some(atlas) = &mut base_sprite.texture_atlas {
                atlas.index = 7;
            }
        } else {
            if let Some(atlas) = &mut base_sprite.texture_atlas {
                if atlas.index != 6 && atlas.index != 7 {
                    atlas.index = 6;
                }
            }
        }
    } else {
        // Row 1 base
        if let Some(atlas) = &mut base_sprite.texture_atlas {
            atlas.index = 0;
        }
        rotation = 0.0;
    }

    base_transform.rotation = Quat::from_rotation_z(rotation);

    // Overlays
    for child in children.iter() {
        if let Ok((mut sprite, mut timer, mut visibility, layer)) = overlay_query.get_mut(child) {
            match layer {
                PlayerLayer::DashOverlay => {
                    if player.is_dashing {
                        *visibility = Visibility::Visible;
                        timer.0.tick(time.delta());
                        if timer.0.just_finished() {
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.index = 12 + (atlas.index - 12 + 1) % 5;
                            }
                        }
                    } else {
                        *visibility = Visibility::Hidden;
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.index = 12;
                        }
                    }
                }
                PlayerLayer::IdleOverlay => {
                    let is_idle =
                        !is_moving && player.dash_cooldown <= 0.0 && player.shoot_cooldown <= 0.0;
                    if is_idle {
                        *visibility = Visibility::Visible;
                        timer.0.tick(time.delta());
                        if timer.0.just_finished() {
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.index = 18 + (atlas.index - 18 + 1) % 6;
                            }
                        }
                    } else {
                        *visibility = Visibility::Hidden;
                    }
                }
                _ => {}
            }
        }
    }
}

fn update_player_timers(
    player_query: Query<&Player>,
    mut timer_overlay_query: Query<(&mut Sprite, &PlayerLayer), Without<Player>>,
    player_children: Query<&Children, With<Player>>,
) {
    let Ok(player) = player_query.single() else {
        return;
    };
    let Ok(children) = player_children.single() else {
        return;
    };

    for child in children.iter() {
        if let Ok((mut sprite, layer)) = timer_overlay_query.get_mut(child) {
            match layer {
                PlayerLayer::DashTimer => {
                    let ratio = if player.dash_cooldown > 0.0 {
                        1.0 - (player.dash_cooldown / DASH_COOLDOWN).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let frame = (ratio * 5.0).round() as usize;
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.index = 24 + frame;
                    }
                }
                PlayerLayer::ReloadTimer => {
                    let ratio = if player.shoot_cooldown > 0.0 {
                        1.0 - (player.shoot_cooldown / SHOOT_COOLDOWN).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let frame = (ratio * 5.0).round() as usize;
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.index = 30 + frame;
                    }
                }
                _ => {}
            }
        }
    }
}
