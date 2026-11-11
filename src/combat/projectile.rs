//! Shot requests become projectiles, which fly straight, hit the first
//! target on their path and vanish at the Room's walls. The player's shots
//! hit enemies; enemy bullets hit the player, unless the player is Dodge
//! Rolling through its invincibility.

use crate::{
    combat::{DamageRequest, DamageType, Hits, ShotRequest},
    data::{Enemies, Tuning},
    enemies::{Enemy, Knockback},
    kinematics::{Position, Velocity},
    player::{DodgeRoll, Player},
    run::RunScoped,
    schedule::GameSystemSet,
    world::room::Room,
};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Projectile {
    pub shooter: Entity,
    /// The weapon id it left, if any.
    pub weapon: Option<String>,
    pub hits: Hits,
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
            .after(crate::enemies::move_enemies)
            .after(crate::enemies::gunner::shoot)
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
        let deviation = (rng.0.f32() * 2.0 - 1.0) * shot.spread.to_radians();
        let direction = Vec2::from_angle(deviation).rotate(shot.direction);
        commands.spawn((
            Projectile {
                shooter: shot.shooter,
                weapon: shot.weapon.clone(),
                hits: shot.hits,
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

/// Whether the segment from `a` to `b` passes within `radius` of `centre`.
/// Sweeping the whole frame's path means fast shots never tunnel.
pub fn segment_hits_circle(a: Vec2, b: Vec2, centre: Vec2, radius: f32) -> bool {
    let path = b - a;
    let length_squared = path.length_squared();
    let t = if length_squared > 0.0 {
        ((centre - a).dot(path) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (a + path * t).distance_squared(centre) <= radius * radius
}

type Target<'a> = (Entity, &'a Position, Option<&'a DodgeRoll>);
type Flying<'a> = (Entity, &'a Projectile, &'a Velocity, &'a mut Position);

/// Each projectile sweeps its frame's path. A shot at enemies hurts the one
/// nearest its start and knocks it back along the shot; an enemy bullet
/// hurts the player. Either is spent on a hit, even on a player still
/// invulnerable after being hurt. Bullets pass through a player Dodge
/// Rolling through its invincibility.
#[allow(clippy::too_many_arguments)]
fn move_projectiles(
    time: Res<Time>,
    tuning: Res<Tuning>,
    catalog: Res<Enemies>,
    room: Single<&Room>,
    player: Option<Single<Target, With<Player>>>,
    projectiles: Query<Flying, (Without<Enemy>, Without<Player>)>,
    mut enemies: Query<(Entity, &Enemy, &Position, &mut Knockback), Without<Player>>,
    mut damage: MessageWriter<DamageRequest>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let player = player.map(|player| *player);
    let rolling_through = player.is_some_and(|(_, _, roll)| {
        roll.is_some_and(|roll| roll.is_invincible(&tuning.player.dodge_roll))
    });
    for (entity, projectile, velocity, mut position) in projectiles {
        let from = position.0;
        position.0 += velocity.0 * dt;
        let hit = match projectile.hits {
            Hits::Enemies => enemies
                .iter()
                .filter(|(target, enemy, at, _)| {
                    let data = catalog.get(enemy.id()).expect("enemy id in enemies.ron");
                    *target != projectile.shooter
                        && segment_hits_circle(
                            from,
                            position.0,
                            at.0,
                            data.radius + projectile.radius,
                        )
                })
                .min_by(|(_, _, a, _), (_, _, b, _)| {
                    from.distance_squared(a.0)
                        .total_cmp(&from.distance_squared(b.0))
                })
                .map(|(target, ..)| target),
            Hits::Player => player
                .filter(|(_, at, _)| {
                    !rolling_through
                        && segment_hits_circle(
                            from,
                            position.0,
                            at.0,
                            tuning.player.radius + projectile.radius,
                        )
                })
                .map(|(target, ..)| target),
        };
        if let Some(target) = hit {
            damage.write(DamageRequest {
                target,
                amount: projectile.damage,
                damage_type: projectile.damage_type,
                weapon: projectile.weapon.clone(),
            });
            if let Ok((.., mut knockback)) = enemies.get_mut(target) {
                knockback.0 += velocity.0.normalize_or_zero() * tuning.juice.knockback;
            }
            commands.entity(entity).despawn();
        } else if !room.interior.contains(position.0) {
            commands.entity(entity).despawn();
        }
    }
}
