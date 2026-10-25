//! Bundled data files, compiled into the binary (ADR 0003, WASM first) and
//! deserialized into resources.

use bevy::prelude::*;
use serde::Deserialize;

const TUNING_RON: &str = include_str!("../assets/tuning.ron");

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
