// ============================================================================
// projectile.rs
use bevy::prelude::*;
use crate::constants::*;
use crate::game::{Game, GameState};

#[derive(Component)]
pub struct Projectile {
    pub direction: Vec3,
    pub distance_traveled: f32,
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, projectile_movement);
    }
}

fn projectile_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    game: Res<Game>,
) {
    if game.state != GameState::Playing {
        return;
    }

    for (entity, mut transform, mut projectile) in query.iter_mut() {
        let dist = PROJECTILE_SPEED * time.delta_secs();
        transform.translation += projectile.direction * dist;
        projectile.distance_traveled += dist;

        if projectile.distance_traveled >= PROJECTILE_MAX_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}