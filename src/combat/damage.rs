//! Damage types, which Resistances will mitigate.

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum DamageType {
    /// Las weapons.
    Energy,
}
