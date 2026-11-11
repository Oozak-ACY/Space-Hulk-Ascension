//! Damage types and the Resistances that mitigate them.

use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum DamageType {
    /// Las weapons.
    Energy,
    /// Blades, claws, fists and solid shot.
    Physical,
}

/// Fraction of each damage type's damage ignored (0.5 = half); a missing
/// type is 0. Capped at the tuned maximum Resistance when applied.
#[derive(Component, Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct Resistances(pub HashMap<DamageType, f32>);

impl Resistances {
    pub fn get(&self, damage_type: DamageType) -> f32 {
        self.0.get(&damage_type).copied().unwrap_or(0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// What is left of `amount` of `damage_type` once this Resistance,
    /// capped at `max_resistance`, has taken its share.
    pub fn mitigate(&self, amount: f32, damage_type: DamageType, max_resistance: f32) -> f32 {
        amount * (1.0 - self.get(damage_type).min(max_resistance))
    }
}
