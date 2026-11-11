//! Shot requests become projectiles, which fly straight and vanish at the
//! Room's walls.

use crate::{
    combat::{DamageType, ShotRequest},
    kinematics::{Position, Velocity},
    run::RunScoped,
    schedule::GameSystemSet,
    world::room::Room,
};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Projectile {
    pub shooter: Entity,
    pub damage: f32,
    pub damage_type: DamageType,
    /// Collision circle, px.
    pub radius: f32,
}

/// Rolls each shot's spread; reseeded from the Run seed so a Run replays exactly.
#[derive(Resource, Debug)]
pub struct ShotRng(pub fastrand::Rng);

impl Default for ShotRng {
    fn default() -> Self {
        Self(fastrand::Rng::with_seed(0))
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<ShotRng>().add_systems(
        Update,
        (move_projectiles, spawn_projectiles)
            .chain()
            .in_set(GameSystemSet::PhysicsAndCombat),
    );
}

/// New projectiles start moving on the next frame.
fn spawn_projectiles(
    mut requests: MessageReader<ShotRequest>,
    mut rng: ResMut<ShotRng>,
    mut commands: Commands,
) {
    for shot in requests.read() {
        let deviation = (rng.0.f32() - 0.5) * shot.spread.to_radians();
        let direction = Vec2::from_angle(deviation).rotate(shot.direction);
        commands.spawn((
            Projectile {
                shooter: shot.shooter,
                damage: shot.damage,
                damage_type: shot.damage_type,
                radius: shot.radius,
            },
            Position(shot.origin),
            Velocity(direction * shot.speed),
            RunScoped,
        ));
    }
}

fn move_projectiles(
    time: Res<Time>,
    room: Single<&Room>,
    mut projectiles: Query<(Entity, &Velocity, &mut Position), With<Projectile>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, velocity, mut position) in &mut projectiles {
        position.0 += velocity.0 * dt;
        if !room.interior.contains(position.0) {
            commands.entity(entity).despawn();
        }
    }
}
