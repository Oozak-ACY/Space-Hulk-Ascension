//! The rusher: charges straight at the player and hurts on contact.

use super::{Enemy, Knockback};
use crate::{
    combat::{DamageRequest, HurtInvulnerability, health::is_untouchable},
    data::{Enemies, EnemyBehaviour, Tuning},
    kinematics::{Position, Velocity},
    player::{DodgeRoll, Player},
};
use bevy::prelude::*;

/// Seconds until this rusher's contact can hurt again.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct ContactCooldown(pub f32);

type Rusher<'a> = (&'a Enemy, &'a Position, &'a mut Velocity);

/// Heads for the player at full speed, easing into turns.
pub(super) fn steer(
    time: Res<Time>,
    catalog: Res<Enemies>,
    player: Single<&Position, With<Player>>,
    mut rushers: Query<Rusher, (With<ContactCooldown>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    for (enemy, position, mut velocity) in &mut rushers {
        let data = catalog.get(enemy.id()).expect("enemy id in enemies.ron");
        let wanted = (player.0 - position.0).normalize_or_zero() * data.speed;
        velocity.0 = velocity.0.lerp(wanted, 1.0 - (-data.steering * dt).exp());
    }
}

type Target<'a> = (
    Entity,
    &'a Position,
    Option<&'a DodgeRoll>,
    Option<&'a HurtInvulnerability>,
);

/// Touching the player hurts them, then waits out the contact cooldown.
/// A touch that lands bounces the rusher back.
pub(super) fn touch(
    time: Res<Time>,
    tuning: Res<Tuning>,
    catalog: Res<Enemies>,
    player: Single<Target, With<Player>>,
    mut rushers: Query<(&Enemy, &Position, &mut ContactCooldown, &mut Knockback), Without<Player>>,
    mut damage: MessageWriter<DamageRequest>,
) {
    let dt = time.delta_secs();
    let (target, target_position, roll, hurt) = *player;
    for (enemy, position, mut cooldown, mut knockback) in &mut rushers {
        cooldown.0 = (cooldown.0 - dt).max(0.0);
        let data = catalog.get(enemy.id()).expect("enemy id in enemies.ron");
        let EnemyBehaviour::Rusher(contact) = data.behaviour;
        let reach = data.radius + tuning.player.radius;
        if cooldown.0 > 0.0 || position.0.distance(target_position.0) >= reach {
            continue;
        }
        cooldown.0 = contact.cooldown;
        if is_untouchable(roll, hurt, &tuning.player.dodge_roll) {
            continue;
        }
        damage.write(DamageRequest {
            target,
            amount: contact.damage,
            damage_type: contact.damage_type,
            weapon: None,
        });
        knockback.0 += (position.0 - target_position.0).normalize_or_zero() * contact.recoil;
    }
}
