//movements, walls, bullets, dash, audio
use bevy::prelude::*;

const PLAYER_SPEED: f32 = 300.0;
const PROJECTILE_SPEED: f32 = 700.0;
const PROJECTILE_MAX_DISTANCE: f32 = 600.0;
const DASH_DISTANCE: f32 = 200.0;
const DASH_DURATION: f32 = 0.2;
const SHOOT_COOLDOWN: f32 = 0.3; 
const DASH_COOLDOWN: f32 = 0.6;
const STEP_INTERVAL: f32 = 0.35;

const PLAYER_SIZE: f32 = 50.0;
const PROJECTILE_RADIUS: f32 = 7.5;
const WALL_THICKNESS: f32 = 20.0;

// --- Colors ---
const PLAYER_READY: Color = Color::srgb(0.9, 0.2, 0.2);
const PLAYER_EMPTY: Color = Color::srgb(0.2, 0.2, 0.2);
const GUN_READY: Color = Color::srgb(1.0, 0.8, 0.0);
const GUN_EMPTY: Color = Color::srgb(0.3, 0.3, 0.1);
const PROJECTILE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);

#[derive(Resource)]
struct GameAssets {
    shoot: Handle<AudioSource>,
    dash: Handle<AudioSource>,
    hit: Handle<AudioSource>,
    step: Handle<AudioSource>,
}

#[derive(Component)]
struct Player {
    last_direction: Vec3,
    shoot_cooldown: f32,
    dash_cooldown: f32,
    is_dashing: bool,
    dash_timer: f32,
    dash_direction: Vec3,
    dash_start_pos: Vec3,
    step_timer: f32,
}

#[derive(Component)]
struct GunIndicator;

#[derive(Component)]
struct DashIndicatorFill;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Projectile {
    direction: Vec3,
    distance_traveled: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bloatware".into(),
                resolution: (900, 600).into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, (setup_camera, load_assets, spawn_walls, spawn_player))
        .add_systems(Update, (
            player_movement, 
            dash_handler,
            projectile_movement, 
            check_collisions_player,
            check_collisions_projectile,
            update_visuals,
        ))
        .run();
}


// --- Setup Systems ---
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        shoot: asset_server.load("shoot.ogg"),
        dash: asset_server.load("dash.ogg"),
        hit: asset_server.load("hit.ogg"),
        step: asset_server.load("step.ogg"),
    });
}

fn spawn_player(mut commands: Commands) {
    // Create player body (empty background)
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
        // Dash indicator fill (fills from bottom to top)
        parent.spawn((
            Sprite {
                color: PLAYER_READY,
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 0.5),
            DashIndicatorFill,
        ));

        // Gun background (empty)
        parent.spawn((
            Sprite {
                color: GUN_EMPTY,
                custom_size: Some(Vec2::new(PLAYER_SIZE * 0.4, PLAYER_SIZE * 0.4)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));

        // Gun fill indicator (fills from bottom to top) - changes color based on cooldown
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

fn spawn_walls(mut commands: Commands) {
    let w = 900.0;
    let h = 600.0;
    let wall_color = Color::srgb(0.3, 0.3, 0.3);

    let walls = [
        (Vec2::new(w, WALL_THICKNESS), Vec3::new(0.0, h/2.0 - WALL_THICKNESS/2.0, 0.0)),
        (Vec2::new(w, WALL_THICKNESS), Vec3::new(0.0, -h/2.0 + WALL_THICKNESS/2.0, 0.0)),
        (Vec2::new(WALL_THICKNESS, h), Vec3::new(-w/2.0 + WALL_THICKNESS/2.0, 0.0, 0.0)),
        (Vec2::new(WALL_THICKNESS, h), Vec3::new(w/2.0 - WALL_THICKNESS/2.0, 0.0, 0.0)),
    ];

    for (size, pos) in walls {
        commands.spawn((
            Sprite {
                color: wall_color,
                custom_size: Some(size),
                ..Default::default()
            },
            Transform::from_translation(pos),
            Wall,
        ));
    }
}


// --- Gameplay Systems ---
fn player_movement(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player)>,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((mut transform, mut player)) = query.single_mut() else { return; };

    if player.is_dashing { return; }

    // Tick cooldowns
    player.shoot_cooldown = (player.shoot_cooldown - time.delta_secs()).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - time.delta_secs()).max(0.0);

    // Movement input
    let mut dir = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) { dir.y += 1.0; }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) { dir.y -= 1.0; }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) { dir.x -= 1.0; }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }

    if dir != Vec3::ZERO {
        dir = dir.normalize();
        player.last_direction = dir;
        transform.translation += dir * PLAYER_SPEED * time.delta_secs();

        // Footstep audio
        player.step_timer -= time.delta_secs();
        if player.step_timer <= 0.0 {
            commands.spawn(AudioPlayer(assets.step.clone()));
            player.step_timer = STEP_INTERVAL;
        }
    } else {
        // Reset step timer when not moving to stop audio
        player.step_timer = STEP_INTERVAL;
    }

    // Dash
    if player.dash_cooldown <= 0.0 && (input.just_pressed(KeyCode::ShiftLeft) || input.just_pressed(KeyCode::ShiftRight)) {
        player.is_dashing = true;
        player.dash_timer = DASH_DURATION;
        player.dash_direction = if dir != Vec3::ZERO { dir } else { player.last_direction };
        player.dash_start_pos = transform.translation;
        player.dash_cooldown = DASH_COOLDOWN;

        commands.spawn(AudioPlayer(assets.dash.clone()));
    }

    // Shoot
    if player.shoot_cooldown <= 0.0 && input.just_pressed(KeyCode::Space) {
        let shoot_dir = if dir != Vec3::ZERO { dir } else { player.last_direction };
        
        // Spawn circle projectile using Mesh2d
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

fn dash_handler(time: Res<Time>, mut query: Query<(&mut Transform, &mut Player)>) {
    let Ok((mut transform, mut player)) = query.single_mut() else { return; };
    if !player.is_dashing { return; }

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

fn projectile_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>
) {
    for (e, mut t, mut p) in query.iter_mut() {
        let dist = PROJECTILE_SPEED * time.delta_secs();
        t.translation += p.direction * dist;
        p.distance_traveled += dist;

        if p.distance_traveled >= PROJECTILE_MAX_DISTANCE {
            commands.entity(e).despawn();
        }
    }
}


// --- Visuals System ---
fn update_visuals(
    player_query: Query<(&Player, &Children)>,
    mut dash_fill_query: Query<(&mut Sprite, &mut Transform), (With<DashIndicatorFill>, Without<GunIndicator>)>,
    mut gun_query: Query<(&mut Sprite, &mut Transform), (With<GunIndicator>, Without<DashIndicatorFill>)>,
) {
    let Ok((player, children)) = player_query.single() else { return; };

    // Update dash fill height
    for child in children.iter() {
        if let Ok((mut dash_sprite, mut dash_transform)) = dash_fill_query.get_mut(child) {
            let fill_ratio = if player.dash_cooldown > 0.0 {
                1.0 - (player.dash_cooldown / DASH_COOLDOWN).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let new_height = PLAYER_SIZE * fill_ratio;
            dash_sprite.custom_size = Some(Vec2::new(PLAYER_SIZE, new_height));
            // Adjust Y position to keep bottom edge at bottom of player
            dash_transform.translation.y = (new_height - PLAYER_SIZE) / 2.0;
        }

        // Update gun fill height
        if let Ok((mut gun_sprite, mut gun_transform)) = gun_query.get_mut(child) {
            let fill_ratio = if player.shoot_cooldown > 0.0 {
                1.0 - (player.shoot_cooldown / SHOOT_COOLDOWN).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let gun_size = PLAYER_SIZE * 0.4;
            let new_height = gun_size * fill_ratio;
            gun_sprite.custom_size = Some(Vec2::new(gun_size, new_height));
            // Adjust Y position to keep bottom edge at bottom of gun
            gun_transform.translation.y = (new_height - gun_size) / 2.0;
        }
    }
}


// --- Collision Systems ---
fn check_collisions_player(
    mut player_q: Query<(&mut Transform, &mut Player)>,
    wall_q: Query<(&Transform, &Sprite), (With<Wall>, Without<Player>)>,
) {
    let Ok((mut p_t, mut p)) = player_q.single_mut() else { return; };
    let p_pos = p_t.translation;
    let p_rad = PLAYER_SIZE / 2.0;

    for (w_t, w_s) in wall_q.iter() {
        let w_pos = w_t.translation;
        let w_size = w_s.custom_size.unwrap();
        let w_half = w_size / 2.0;

        let dx = (p_pos.x - w_pos.x).abs();
        let dy = (p_pos.y - w_pos.y).abs();

        if dx < (p_rad + w_half.x) && dy < (p_rad + w_half.y) {
            let ox = (p_rad + w_half.x) - dx;
            let oy = (p_rad + w_half.y) - dy;

            if ox < oy {
                p_t.translation.x += if p_pos.x < w_pos.x { -ox } else { ox };
            } else {
                p_t.translation.y += if p_pos.y < w_pos.y { -oy } else { oy };
            }

            if p.is_dashing {
                p.is_dashing = false;
            }
        }
    }
}

fn check_collisions_projectile(
    mut commands: Commands,
    assets: Res<GameAssets>,
    proj_q: Query<(Entity, &Transform), With<Projectile>>,
    wall_q: Query<(&Transform, &Sprite), With<Wall>>,
) {
    for (p_e, p_t) in proj_q.iter() {
        let p_pos = p_t.translation;
        let p_rad = PROJECTILE_RADIUS;

        for (w_t, w_s) in wall_q.iter() {
            let w_pos = w_t.translation;
            let w_size = w_s.custom_size.unwrap();
            let w_half = w_size / 2.0;

            let dx = (p_pos.x - w_pos.x).abs();
            let dy = (p_pos.y - w_pos.y).abs();

            if dx < (p_rad + w_half.x) && dy < (p_rad + w_half.y) {
                commands.entity(p_e).despawn();
                commands.spawn(AudioPlayer(assets.hit.clone()));
                break;
            }
        }
    }
}