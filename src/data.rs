//! Bundled data files, compiled into the binary (ADR 0003, WASM first) and
//! deserialized into resources.

use crate::combat::{DamageType, Resistances};
use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;

const TUNING_RON: &str = include_str!("../assets/tuning.ron");
const WEAPONS_RON: &str = include_str!("../assets/weapons.ron");
const ENEMIES_RON: &str = include_str!("../assets/enemies.ron");

/// Global balance values from `assets/tuning.ron`.
#[derive(Resource, Debug, Clone, Deserialize)]
pub struct Tuning {
    pub player: PlayerTuning,
    pub room: RoomTuning,
    pub floor: FloorTuning,
    pub combat: CombatTuning,
    pub spawns: SpawnTuning,
    pub juice: JuiceTuning,
    /// Longest frame the gameplay clock will advance by, in seconds.
    pub max_frame_delta: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerTuning {
    pub move_speed: f32,
    pub snappiness: f32,
    pub max_hp: f32,
    pub hurt_invulnerability: f32,
    pub radius: f32,
    /// Weapon id in `weapons.ron` every Run starts with.
    pub starting_weapon: String,
    pub dodge_roll: DodgeRollTuning,
    #[serde(default)]
    pub resistances: Resistances,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CombatTuning {
    /// The most of any damage type a Resistance can ignore, however stacked.
    pub max_resistance: f32,
}

/// Where enemies appear in a Room.
#[derive(Debug, Clone, Deserialize)]
pub struct SpawnTuning {
    /// Enemies never appear closer than this to the player, px.
    pub min_distance: f32,
    /// Enemy ids in `enemies.ron`, one per enemy in the Run's opening wave.
    pub wave: Vec<String>,
}

/// Screen shake (px), freeze frames (s), hit flashes (s) and knockback.
#[derive(Debug, Clone, Deserialize)]
pub struct JuiceTuning {
    pub shake_on_hit: f32,
    pub shake_on_kill: f32,
    pub shake_on_hurt: f32,
    /// Shake lost per second.
    pub shake_decay: f32,
    pub freeze_on_kill: f32,
    pub freeze_on_hurt: f32,
    /// How long a hit enemy flashes white.
    pub hit_flash: f32,
    /// How long the hurt player flashes white.
    pub hurt_flash: f32,
    /// Speed a hit adds to an enemy along the shot, px/s.
    pub knockback: f32,
    /// Exponential decay rate of knockback speed, 1/s.
    pub knockback_decay: f32,
}

/// Dodge Roll timings, in seconds from the start of the roll.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DodgeRollTuning {
    /// px/s while rolling.
    pub speed: f32,
    /// How long the roll moves the player.
    pub duration: f32,
    /// How long the player cannot be hurt.
    pub invincibility: f32,
    /// How long until the next roll can start.
    pub cooldown: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoomTuning {
    pub interior_width: f32,
    pub interior_height: f32,
    pub wall_thickness: f32,
}

/// Floor generation: how many Rooms, on how large a grid.
#[derive(Debug, Clone, Deserialize)]
pub struct FloorTuning {
    pub min_rooms: usize,
    pub max_rooms: usize,
    /// Grid size in Rooms.
    pub grid_width: i32,
    pub grid_height: i32,
    /// Chance that a Room grows a neighbour in each direction; lower means
    /// sparser, more branching Floors.
    pub expansion_chance: f32,
    /// Encounter Rooms per Floor, placed on spare dead ends first.
    pub encounter_rooms: usize,
}

impl RoomTuning {
    pub fn interior_size(&self) -> Vec2 {
        Vec2::new(self.interior_width, self.interior_height)
    }
}

impl Tuning {
    /// Parses the bundled `tuning.ron`.
    pub fn bundled() -> Result<Self, ron::error::SpannedError> {
        ron::from_str(TUNING_RON)
    }
}

/// Every weapon from `assets/weapons.ron`, by weapon id.
#[derive(Resource, Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct Weapons(HashMap<String, WeaponData>);

#[derive(Debug, Clone, Deserialize)]
pub struct WeaponData {
    /// Shown on the HUD.
    pub name: String,
    pub damage: f32,
    pub damage_type: DamageType,
    /// Seconds between shots while fire is held.
    pub interval: f32,
    /// Projectile speed, px/s.
    pub speed: f32,
    /// Degrees either side of the aim each shot may stray (±spread).
    pub spread: f32,
    pub projectile_radius: f32,
    pub profile: WeaponProfile,
}

/// What stops a weapon firing forever.
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum WeaponProfile {
    Overheat(OverheatData),
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct OverheatData {
    pub heat_per_shot: f32,
    /// Reaching it overheats the weapon.
    pub max_heat: f32,
    /// Heat lost per second.
    pub cooling: f32,
    /// Seconds unable to fire after overheating; heat is 0 afterwards.
    pub lockout: f32,
}

impl Weapons {
    /// Parses the bundled `weapons.ron`.
    pub fn bundled() -> Result<Self, ron::error::SpannedError> {
        ron::from_str(WEAPONS_RON)
    }

    pub fn get(&self, id: &str) -> Option<&WeaponData> {
        self.0.get(id)
    }
}

/// Every enemy type from `assets/enemies.ron`, by enemy id.
#[derive(Resource, Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct Enemies(HashMap<String, EnemyData>);

#[derive(Debug, Clone, Deserialize)]
pub struct EnemyData {
    pub name: String,
    pub max_hp: f32,
    /// Top speed, px/s.
    pub speed: f32,
    /// Exponential approach rate toward the wanted velocity, 1/s.
    pub steering: f32,
    /// Collision circle at the feet, px.
    pub radius: f32,
    #[serde(default)]
    pub resistances: Resistances,
    pub behaviour: EnemyBehaviour,
}

/// How an enemy fights.
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum EnemyBehaviour {
    /// Charges the player and hurts on contact.
    Rusher(ContactData),
    /// Keeps its distance, strafes, and fires telegraphed bullets.
    Gunner(GunnerData),
}

/// How a gunner keeps its distance and shoots.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct GunnerData {
    /// Distance it keeps from the player, px.
    pub range: f32,
    /// How far from its range it strafes rather than approaching or
    /// backing off, px; it blends between the two across this band.
    pub range_tolerance: f32,
    /// Fewest seconds between switches of strafing direction.
    pub strafe_switch_min: f32,
    /// Most seconds between switches of strafing direction.
    pub strafe_switch_max: f32,
    /// Seconds from one aim warning to the next.
    pub interval: f32,
    /// Seconds the aim warning shows before the shot; the shot's direction
    /// is locked when the warning starts.
    pub aim_warning: f32,
    /// px/s.
    pub bullet_speed: f32,
    /// Damage of each bullet, before Resistances.
    pub bullet_damage: f32,
    pub bullet_damage_type: DamageType,
    /// Collision circle, px.
    pub bullet_radius: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ContactData {
    pub damage: f32,
    pub damage_type: DamageType,
    /// Seconds before this enemy's contact can hurt again.
    pub cooldown: f32,
    /// Speed the enemy bounces back with after hurting, px/s.
    pub recoil: f32,
}

impl Enemies {
    /// Parses the bundled `enemies.ron`.
    pub fn bundled() -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ENEMIES_RON)
    }

    pub fn get(&self, id: &str) -> Option<&EnemyData> {
        self.0.get(id)
    }
}
