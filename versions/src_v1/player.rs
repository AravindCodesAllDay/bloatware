// ============================================================================
// player.rs
use bevy::prelude::*;
use crate::audio::GameAssets;
use crate::constants::*;
use crate::game::{Game, GameState};

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
}

#[derive(Component)]
pub struct GunIndicator;

#[derive(Component)]
pub struct DashIndicatorFill;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            player_movement,
            dash_handler,
            update_player_visuals,
        ));
    }
}

pub fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Sprite {
            color: PLAYER_EMPTY,
            custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player {
            last_direction: Vec3::Y,
            shoot_cooldown: 0.0,
            dash_cooldown: 0.0,
            is_dashing: false,
            dash_timer: 0.0,
            dash_direction: Vec3::ZERO,
            dash_start_pos: Vec3::ZERO,
            step_timer: 0.0,
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            Sprite {
                color: PLAYER_READY,
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 0.5),
            DashIndicatorFill,
        ));

        parent.spawn((
            Sprite {
                color: GUN_EMPTY,
                custom_size: Some(Vec2::new(PLAYER_SIZE * 0.4, PLAYER_SIZE * 0.4)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));

        parent.spawn((
            Sprite {
                color: GUN_READY,
                custom_size: Some(Vec2::new(PLAYER_SIZE * 0.4, PLAYER_SIZE * 0.4)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
            GunIndicator,
        ));
    });
}

fn player_movement(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player)>,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok((mut transform, mut player)) = query.single_mut() else { return };

    if player.is_dashing {
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
        transform.translation += dir * PLAYER_SPEED * time.delta_secs();

        player.step_timer -= time.delta_secs();
        if player.step_timer <= 0.0 {
            commands.spawn(AudioPlayer(assets.step.clone()));
            player.step_timer = STEP_INTERVAL;
        }
    } else {
        player.step_timer = STEP_INTERVAL;
    }

    if player.dash_cooldown <= 0.0
        && (input.just_pressed(KeyCode::ShiftLeft) || input.just_pressed(KeyCode::ShiftRight))
    {
        player.is_dashing = true;
        player.dash_timer = DASH_DURATION;
        player.dash_direction = if dir != Vec3::ZERO { dir } else { player.last_direction };
        player.dash_start_pos = transform.translation;
        player.dash_cooldown = DASH_COOLDOWN;

        commands.spawn(AudioPlayer(assets.dash.clone()));
    }

    if player.shoot_cooldown <= 0.0 && input.just_pressed(KeyCode::Space) {
        let shoot_dir = if dir != Vec3::ZERO { dir } else { player.last_direction };

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PROJECTILE_RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from(PROJECTILE_COLOR))),
            Transform::from_translation(transform.translation),
            crate::projectile::Projectile {
                direction: shoot_dir,
                distance_traveled: 0.0,
            },
        ));

        commands.spawn(AudioPlayer(assets.shoot.clone()));
        player.shoot_cooldown = SHOOT_COOLDOWN;
    }
}

fn dash_handler(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Player)>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    let Ok((mut transform, mut player)) = query.single_mut() else { return };
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

fn update_player_visuals(
    player_query: Query<(&Player, &Children)>,
    mut dash_fill_query: Query<
        (&mut Sprite, &mut Transform),
        (With<DashIndicatorFill>, Without<GunIndicator>),
    >,
    mut gun_query: Query<
        (&mut Sprite, &mut Transform),
        (With<GunIndicator>, Without<DashIndicatorFill>),
    >,
) {
    let Ok((player, children)) = player_query.single() else { return };

    for child in children.iter() {
        if let Ok((mut dash_sprite, mut dash_transform)) = dash_fill_query.get_mut(child) {
            let fill_ratio = if player.dash_cooldown > 0.0 {
                1.0 - (player.dash_cooldown / DASH_COOLDOWN).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let new_height = PLAYER_SIZE * fill_ratio;
            dash_sprite.custom_size = Some(Vec2::new(PLAYER_SIZE, new_height));
            dash_transform.translation.y = (new_height - PLAYER_SIZE) / 2.0;
        }

        if let Ok((mut gun_sprite, mut gun_transform)) = gun_query.get_mut(child) {
            let fill_ratio = if player.shoot_cooldown > 0.0 {
                1.0 - (player.shoot_cooldown / SHOOT_COOLDOWN).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let gun_size = PLAYER_SIZE * 0.4;
            let new_height = gun_size * fill_ratio;
            gun_sprite.custom_size = Some(Vec2::new(gun_size, new_height));
            gun_transform.translation.y = (new_height - gun_size) / 2.0;
        }
    }
}