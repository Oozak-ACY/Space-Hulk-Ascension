//! Weapons, shots, the projectiles they fire, and the damage they deal.

pub mod damage;
pub mod health;
pub mod projectile;
pub mod weapons;

pub use damage::{DamageType, Resistances};
pub use health::{
    DamageRequest, DamageTaken, EnemyDamaged, EnemyKilled, FatalDamage, Health, HitFlash,
    HurtInvulnerability,
};
pub use projectile::{Projectile, ShotRng};
pub use weapons::{FireIntent, ShotRequest, Weapon};

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((weapons::plugin, projectile::plugin, health::plugin));
}
