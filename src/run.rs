//! Run lifecycle: a "start Run" request clears every Run-scoped entity and
//! sets up a fresh Run.

use crate::{
    data::Tuning,
    kinematics::{Position, Velocity},
    player::{MoveIntent, Player},
    schedule::GameSystemSet,
    world::room::Room,
};
use bevy::prelude::*;

/// Starts a new Run, from `seed` when given (to reproduce a Run) or a random one.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartRun {
    pub seed: Option<u64>,
}

/// The seed of the current Run.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeed(pub u64);

/// Despawned when a new Run starts.
#[derive(Component, Debug, Default)]
pub struct RunScoped;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<StartRun>()
        .add_systems(Update, start_run.in_set(GameSystemSet::InputAndIntent));
}

fn start_run(
    mut requests: MessageReader<StartRun>,
    run_scoped: Query<Entity, With<RunScoped>>,
    tuning: Res<Tuning>,
    mut commands: Commands,
) {
    let Some(request) = requests.read().last() else {
        return;
    };
    for entity in &run_scoped {
        commands.entity(entity).despawn();
    }
    commands.insert_resource(RunSeed(request.seed.unwrap_or_else(|| fastrand::u64(..))));

    commands.spawn((
        Room {
            interior: Rect::from_center_size(Vec2::ZERO, tuning.room.interior_size()),
        },
        RunScoped,
    ));
    commands.spawn((
        Player,
        MoveIntent::default(),
        Position(Vec2::ZERO),
        Velocity::default(),
        RunScoped,
    ));
}
