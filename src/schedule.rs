//! The fixed gameplay system order (spec v3 §6).

use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSystemSet {
    /// Keyboard/mouse or Hijack → intents and requests.
    InputAndIntent,
    /// Blessings and weapons mutate shot requests.
    ShotModification,
    /// Movement, projectile spawn, custom collision.
    PhysicsAndCombat,
    /// Apply damage, fatal damage → death interception.
    DamageAndDeathHooks,
    /// Psyche, Allegiance, Active Quests.
    EventEvaluation,
    /// Doors, Room clear.
    WorldReaction,
    /// Drawing follows the world state.
    RenderSync,
}

pub(crate) fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (
            GameSystemSet::InputAndIntent,
            GameSystemSet::ShotModification,
            GameSystemSet::PhysicsAndCombat,
            GameSystemSet::DamageAndDeathHooks,
            GameSystemSet::EventEvaluation,
            GameSystemSet::WorldReaction,
            GameSystemSet::RenderSync,
        )
            .chain(),
    );
}
