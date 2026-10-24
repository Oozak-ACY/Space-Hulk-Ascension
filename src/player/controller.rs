//! The player's intents and movement.

use crate::{
    data::Tuning,
    kinematics::{Position, Velocity},
    schedule::GameSystemSet,
    world::room::Room,
};
use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Player;

/// Where the player wants to walk. Longer than 1 is treated as 1, so
/// diagonals are no faster than straight lines.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct MoveIntent(pub Vec2);

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, move_player.in_set(GameSystemSet::PhysicsAndCombat));
}

fn move_player(
    time: Res<Time>,
    tuning: Res<Tuning>,
    room: Single<&Room>,
    mut players: Query<(&MoveIntent, &mut Velocity, &mut Position), With<Player>>,
) {
    let dt = time.delta_secs();
    let player = &tuning.player;
    let walkable = room.interior.inflate(-player.radius);
    let approach = 1.0 - (-player.snappiness * dt).exp();
    for (intent, mut velocity, mut position) in &mut players {
        let target = intent.0.clamp_length_max(1.0) * player.move_speed;
        velocity.0 = velocity.0.lerp(target, approach);
        position.0 = (position.0 + velocity.0 * dt).clamp(walkable.min, walkable.max);
    }
}
