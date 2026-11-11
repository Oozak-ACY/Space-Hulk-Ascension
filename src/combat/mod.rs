//! Weapons, shots and the projectiles they fire.

pub mod damage;
pub mod projectile;
pub mod weapons;

pub use damage::DamageType;
pub use projectile::{Projectile, ShotRng};
pub use weapons::{FireIntent, ShotRequest, Weapon};

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((weapons::plugin, projectile::plugin));
}
