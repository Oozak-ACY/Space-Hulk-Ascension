//! Run lifecycle: a "start Run" request clears every Run-scoped entity and
//! sets up a fresh Run; death ends it in Game over.

use crate::{
    combat::{FireIntent, Health, HitFlash, HurtInvulnerability, ShotRng, Weapon},
    data::Tuning,
    enemies::SpawnRng,
    kinematics::{Position, Velocity},
    player::{AimIntent, DodgeIntent, DodgeRoll, MoveIntent, Player},
    schedule::GameSystemSet,
    world::room::Room,
};
use bevy::prelude::*;

/// Starts a new Run, from `seed` when given (to reproduce a Run) or a random one.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartRun {
    pub seed: Option<u64>,
}

/// Gameplay systems run only in game.
#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    InGame,
    /// The Run ended in death; a "start Run" request begins the next one.
    GameOver,
}

/// How deep the current Run has reached.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Depth {
    /// From 1, the Floor the player is on.
    pub floor: u32,
}

/// The seed of the current Run.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeed(pub u64);

/// Mixed into the Run seed so spawns and shot spread draw different numbers.
const SPAWN_STREAM: u64 = 0x5EED_5EED_5EED_5EED;

/// Despawned when a new Run starts.
#[derive(Component, Debug, Default)]
pub struct RunScoped;

pub(crate) fn plugin(app: &mut App) {
    app.init_state::<AppState>()
        .add_message::<StartRun>()
        .add_systems(Update, start_run.in_set(GameSystemSet::InputAndIntent));
}

pub(crate) fn start_run(
    mut requests: MessageReader<StartRun>,
    run_scoped: Query<Entity, With<RunScoped>>,
    tuning: Res<Tuning>,
    mut state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    let Some(request) = requests.read().last() else {
        return;
    };
    state.set(AppState::InGame);
    commands.insert_resource(Depth { floor: 1 });
    for entity in &run_scoped {
        commands.entity(entity).despawn();
    }
    let seed = request.seed.unwrap_or_else(|| fastrand::u64(..));
    commands.insert_resource(RunSeed(seed));
    commands.insert_resource(ShotRng(fastrand::Rng::with_seed(seed)));
    commands.insert_resource(SpawnRng(fastrand::Rng::with_seed(seed ^ SPAWN_STREAM)));

    commands.spawn((
        Room {
            interior: Rect::from_center_size(Vec2::ZERO, tuning.room.interior_size()),
        },
        RunScoped,
    ));
    commands.spawn((
        Player,
        MoveIntent::default(),
        AimIntent::default(),
        DodgeIntent::default(),
        DodgeRoll::default(),
        FireIntent::default(),
        Weapon::new(tuning.player.starting_weapon.clone()),
        Health::new(tuning.player.max_hp),
        tuning.player.resistances.clone(),
        HurtInvulnerability::default(),
        HitFlash::default(),
        Position(Vec2::ZERO),
        Velocity::default(),
        RunScoped,
    ));
}
