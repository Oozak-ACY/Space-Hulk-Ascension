//! Held fire → shot requests, paced by each weapon's interval and its
//! overheat profile.

use crate::{
    combat::DamageType,
    data::{Tuning, WeaponData, WeaponProfile, Weapons},
    kinematics::Position,
    player::{AimIntent, DodgeRoll},
    schedule::GameSystemSet,
};
use bevy::prelude::*;

/// The fire button is held. Unlike the Dodge Roll, it stays set until released.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FireIntent(pub bool);

/// A shot leaving a weapon, before the projectile exists and before spread
/// is rolled. Shot modifiers (Blessings) will hook in between this and the
/// projectile spawner.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ShotRequest {
    pub shooter: Entity,
    pub origin: Vec2,
    /// Unit length; the aim before spread.
    pub direction: Vec2,
    /// px/s.
    pub speed: f32,
    /// Full cone width in degrees.
    pub spread: f32,
    pub damage: f32,
    pub damage_type: DamageType,
    pub radius: f32,
}

/// The weapon in hand and its state; its stats are the [`WeaponData`] under `id`.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Weapon {
    id: String,
    /// Seconds until the next shot may leave. Held fire carries up to one
    /// frame of overshoot into the next interval, so the rhythm stays even.
    until_next_shot: f32,
    heat: f32,
    /// Seconds of overheat lockout left; 0 when not overheated.
    lockout_left: f32,
}

impl Weapon {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            until_next_shot: 0.0,
            heat: 0.0,
            lockout_left: 0.0,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// From 0 to the profile's max heat. Drains from max to 0 over the lockout.
    pub fn heat(&self) -> f32 {
        self.heat
    }

    /// Heat from 0 to 1 of the profile's max heat.
    pub fn heat_fraction(&self, data: &WeaponData) -> f32 {
        match data.profile {
            WeaponProfile::Overheat(overheat) => self.heat / overheat.max_heat,
        }
    }

    /// Locked out after reaching max heat.
    pub fn is_overheated(&self) -> bool {
        self.lockout_left > 0.0
    }

    fn cool(&mut self, dt: f32, data: &WeaponData) {
        self.until_next_shot -= dt;
        match data.profile {
            WeaponProfile::Overheat(overheat) => {
                if self.is_overheated() {
                    self.lockout_left = (self.lockout_left - dt).max(0.0);
                    self.heat = overheat.max_heat * self.lockout_left / overheat.lockout;
                } else {
                    self.heat = (self.heat - overheat.cooling * dt).max(0.0);
                }
            }
        }
    }

    /// Fires if the interval has elapsed and the weapon is not locked out.
    fn try_fire(&mut self, data: &WeaponData) -> bool {
        if self.until_next_shot > 0.0 || self.is_overheated() {
            return false;
        }
        self.until_next_shot += data.interval;
        match data.profile {
            WeaponProfile::Overheat(overheat) => {
                self.heat += overheat.heat_per_shot;
                if self.heat >= overheat.max_heat {
                    self.heat = overheat.max_heat;
                    self.lockout_left = overheat.lockout;
                }
            }
        }
        true
    }

    /// Not firing this frame: drop the overshoot, so a fresh press fires at
    /// once but never in a burst.
    fn rest(&mut self) {
        self.until_next_shot = self.until_next_shot.max(0.0);
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<ShotRequest>().add_systems(
        Update,
        (cool_weapons, pull_triggers)
            .chain()
            .in_set(GameSystemSet::ShotModification),
    );
}

fn cool_weapons(time: Res<Time>, catalog: Res<Weapons>, mut weapons: Query<&mut Weapon>) {
    let dt = time.delta_secs();
    for mut weapon in &mut weapons {
        let data = catalog.get(&weapon.id).expect("weapon id in weapons.ron");
        weapon.cool(dt, data);
    }
}

/// Anything that holds a weapon and can pull its trigger.
type Shooter<'a> = (
    Entity,
    &'a FireIntent,
    &'a AimIntent,
    &'a Position,
    Option<&'a DodgeRoll>,
    &'a mut Weapon,
);

/// Held fire shoots toward the aim, except while paused or Dodge Rolling.
fn pull_triggers(
    time: Res<Time<Virtual>>,
    tuning: Res<Tuning>,
    catalog: Res<Weapons>,
    mut shooters: Query<Shooter>,
    mut shots: MessageWriter<ShotRequest>,
) {
    let roll_tuning = &tuning.player.dodge_roll;
    for (shooter, fire, aim, position, roll, mut weapon) in &mut shooters {
        let rolling = roll.is_some_and(|roll| roll.is_rolling(roll_tuning));
        let data = catalog.get(&weapon.id).expect("weapon id in weapons.ron");
        if !fire.0 || time.is_paused() || rolling || !weapon.try_fire(data) {
            weapon.rest();
            continue;
        }
        shots.write(ShotRequest {
            shooter,
            origin: position.0,
            // Aiming at one's own feet shoots toward the camera.
            direction: aim.direction.try_normalize().unwrap_or(Vec2::NEG_Y),
            speed: data.speed,
            spread: data.spread,
            damage: data.damage,
            damage_type: data.damage_type,
            radius: data.projectile_radius,
        });
    }
}
