//! The player's intents, movement and Dodge Roll.

use crate::{
    data::{DodgeRollTuning, Tuning},
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

/// Where the player is aiming, from their feet; any length.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct AimIntent {
    pub direction: Vec2,
}

/// The Dodge Roll button was pressed. Consumed on the next frame: a press
/// during the cooldown is dropped, not queued.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DodgeIntent(pub bool);

/// The player's last Dodge Roll. Every phase is timed from its start.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct DodgeRoll {
    /// Seconds since the last roll started; `None` before the first.
    since_start: Option<f32>,
    direction: Vec2,
}

impl DodgeRoll {
    /// Moving at roll speed, ignoring steering.
    pub fn is_rolling(&self, tuning: &DodgeRollTuning) -> bool {
        self.since_start.is_some_and(|t| t < tuning.duration)
    }

    /// Cannot be hurt.
    pub fn is_invincible(&self, tuning: &DodgeRollTuning) -> bool {
        self.since_start.is_some_and(|t| t < tuning.invincibility)
    }

    pub fn is_ready(&self, tuning: &DodgeRollTuning) -> bool {
        self.cooldown_progress(tuning) >= 1.0
    }

    /// From 0 when a roll starts to 1 when the next one is ready.
    pub fn cooldown_progress(&self, tuning: &DodgeRollTuning) -> f32 {
        self.since_start
            .map_or(1.0, |t| (t / tuning.cooldown).min(1.0))
    }

    /// Advances the roll's clock by `dt`; returns how much of `dt` was spent rolling.
    fn advance(&mut self, dt: f32, tuning: &DodgeRollTuning) -> f32 {
        let Some(since_start) = self.since_start.as_mut() else {
            return 0.0;
        };
        let rolling_for = (tuning.duration - *since_start).clamp(0.0, dt);
        *since_start += dt;
        rolling_for
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (start_dodge_roll, move_player)
            .chain()
            .in_set(GameSystemSet::PhysicsAndCombat),
    );
}

/// Rolls in the movement direction, or toward the aim when standing still.
fn start_dodge_roll(
    time: Res<Time<Virtual>>,
    tuning: Res<Tuning>,
    mut players: Query<(&mut DodgeIntent, &MoveIntent, &AimIntent, &mut DodgeRoll), With<Player>>,
) {
    let tuning = &tuning.player.dodge_roll;
    for (mut intent, movement, aim, mut roll) in &mut players {
        // A press while paused is dropped, so unpausing never starts a roll.
        if !std::mem::take(&mut intent.0) || time.is_paused() || !roll.is_ready(tuning) {
            continue;
        }
        let direction = movement.0.try_normalize().or(aim.direction.try_normalize());
        if let Some(direction) = direction {
            *roll = DodgeRoll {
                since_start: Some(0.0),
                direction,
            };
        }
    }
}

fn move_player(
    time: Res<Time>,
    tuning: Res<Tuning>,
    room: Single<&Room>,
    mut players: Query<(&MoveIntent, &mut DodgeRoll, &mut Velocity, &mut Position), With<Player>>,
) {
    let dt = time.delta_secs();
    let player = &tuning.player;
    let walkable = room.interior.inflate(-player.radius);
    let approach = 1.0 - (-player.snappiness * dt).exp();
    for (intent, mut roll, mut velocity, mut position) in &mut players {
        let rolled_for = roll.advance(dt, &player.dodge_roll);
        if rolled_for > 0.0 {
            velocity.0 = roll.direction * player.dodge_roll.speed;
            position.0 += velocity.0 * rolled_for;
            if !roll.is_rolling(&player.dodge_roll) {
                // Leave the roll at walking speed, then ease as usual.
                velocity.0 = velocity.0.clamp_length_max(player.move_speed);
            }
        } else {
            let target = intent.0.clamp_length_max(1.0) * player.move_speed;
            velocity.0 = velocity.0.lerp(target, approach);
            position.0 += velocity.0 * dt;
        }
        position.0 = position.0.clamp(walkable.min, walkable.max);
    }
}
