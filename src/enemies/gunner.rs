//! The gunner: holds its range from the player, strafes, and fires bullets
//! after an aim warning, along the direction locked when the warning began.

use super::{AiRng, Enemy};
use crate::{
    combat::{Hits, ShotRequest},
    data::{Enemies, EnemyBehaviour, EnemyData, GunnerData},
    kinematics::{Position, Velocity},
    player::Player,
};
use bevy::prelude::*;

/// A gunner's fire cycle and strafing.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Gunner {
    /// Seconds until the next aim warning starts.
    until_warning: f32,
    warning: Option<AimWarning>,
    /// +1 strafes counter-clockwise around the player, -1 clockwise.
    strafe_side: f32,
    until_strafe_switch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AimWarning {
    /// Unit length; the shot will fly this way.
    direction: Vec2,
    /// Seconds until the shot.
    left: f32,
}

impl Gunner {
    /// Its first aim warning comes one interval after it appears.
    pub(super) fn new(data: &GunnerData, rng: &mut fastrand::Rng) -> Self {
        Self {
            until_warning: data.interval,
            warning: None,
            strafe_side: if rng.bool() { 1.0 } else { -1.0 },
            until_strafe_switch: strafe_switch_time(data, rng),
        }
    }

    /// The locked direction of the shot it is about to fire, while the aim
    /// warning shows.
    pub fn aim_warning(&self) -> Option<Vec2> {
        self.warning.map(|warning| warning.direction)
    }

    /// +1 strafes counter-clockwise around the player, -1 clockwise.
    pub fn strafe_side(&self) -> f32 {
        self.strafe_side
    }
}

fn strafe_switch_time(data: &GunnerData, rng: &mut fastrand::Rng) -> f32 {
    data.strafe_switch_min + rng.f32() * (data.strafe_switch_max - data.strafe_switch_min)
}

fn gunner_data<'a>(catalog: &'a Enemies, enemy: &Enemy) -> (&'a EnemyData, &'a GunnerData) {
    let data = catalog.get(enemy.id()).expect("enemy id in enemies.ron");
    let EnemyBehaviour::Gunner(gunner) = &data.behaviour else {
        panic!("{} has a Gunner component but is no gunner", enemy.id());
    };
    (data, gunner)
}

type Steered<'a> = (&'a Enemy, &'a Position, &'a mut Velocity, &'a mut Gunner);

/// Approaches when too far, backs off when too close, and strafes around
/// the player in between, blending across the range tolerance band. Now
/// and then it switches strafing direction.
pub(super) fn steer(
    time: Res<Time>,
    catalog: Res<Enemies>,
    mut rng: ResMut<AiRng>,
    player: Single<&Position, With<Player>>,
    mut gunners: Query<Steered, Without<Player>>,
) {
    let dt = time.delta_secs();
    for (enemy, position, mut velocity, mut gunner) in &mut gunners {
        let (enemy_data, data) = gunner_data(&catalog, enemy);
        gunner.until_strafe_switch -= dt;
        if gunner.until_strafe_switch <= 0.0 {
            gunner.strafe_side = -gunner.strafe_side;
            gunner.until_strafe_switch += strafe_switch_time(data, &mut rng.0);
        }
        let offset = player.0 - position.0;
        let towards = offset.normalize_or_zero();
        // +1 approaches, -1 backs off, 0 strafes.
        let approach = ((offset.length() - data.range) / data.range_tolerance).clamp(-1.0, 1.0);
        let around = towards.perp() * gunner.strafe_side;
        let wanted = (towards * approach + around * (1.0 - approach.abs())).normalize_or_zero()
            * enemy_data.speed;
        velocity.0 = velocity
            .0
            .lerp(wanted, 1.0 - (-enemy_data.steering * dt).exp());
    }
}

/// Every interval, locks its aim on the player and shows the aim warning;
/// when the warning runs out, fires along the locked aim.
pub(crate) fn shoot(
    time: Res<Time>,
    catalog: Res<Enemies>,
    player: Single<&Position, With<Player>>,
    mut gunners: Query<(Entity, &Enemy, &Position, &mut Gunner), Without<Player>>,
    mut shots: MessageWriter<ShotRequest>,
) {
    let dt = time.delta_secs();
    for (shooter, enemy, position, mut gunner) in &mut gunners {
        let (_, data) = gunner_data(&catalog, enemy);
        gunner.until_warning -= dt;
        if let Some(mut warning) = gunner.warning {
            warning.left -= dt;
            gunner.warning = (warning.left > 0.0).then_some(warning);
            if gunner.warning.is_none() {
                shots.write(ShotRequest {
                    shooter,
                    weapon: None,
                    hits: Hits::Player,
                    origin: position.0,
                    direction: warning.direction,
                    speed: data.bullet_speed,
                    spread: 0.0,
                    damage: data.bullet_damage,
                    damage_type: data.bullet_damage_type,
                    radius: data.bullet_radius,
                });
            }
        }
        if gunner.until_warning <= 0.0 && gunner.warning.is_none() {
            gunner.until_warning += data.interval;
            gunner.warning = Some(AimWarning {
                // Standing on the player, it shoots toward the camera.
                direction: (player.0 - position.0)
                    .try_normalize()
                    .unwrap_or(Vec2::NEG_Y),
                left: data.aim_warning,
            });
        }
    }
}
