//! Bundled data files, compiled into the binary (ADR 0003, WASM first) and
//! deserialized into resources.

use crate::combat::DamageType;
use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;

const TUNING_RON: &str = include_str!("../assets/tuning.ron");
const WEAPONS_RON: &str = include_str!("../assets/weapons.ron");

/// Global balance values from `assets/tuning.ron`.
#[derive(Resource, Debug, Clone, Deserialize)]
pub struct Tuning {
    pub player: PlayerTuning,
    pub room: RoomTuning,
    pub floor: FloorTuning,
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
