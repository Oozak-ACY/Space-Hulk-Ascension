//! Enemies: spawning them, moving them, and how each kind fights.

pub mod rusher;

use crate::{
    combat::{Health, HitFlash},
    data::{Enemies, EnemyBehaviour, Tuning},
    kinematics::{Position, Velocity},
    player::Player,
    run::{RunScoped, StartRun, start_run},
    schedule::GameSystemSet,
    world::room::Room,
};
use bevy::prelude::*;

/// An enemy; its stats are the [`crate::data::EnemyData`] under `id`.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Enemy {
    id: String,
}

impl Enemy {
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Speed from being shot or from recoil, px/s, on top of the enemy's own
/// movement; decays over time.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Knockback(pub Vec2);

/// Puts an enemy of type `id` at `position`.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct SpawnEnemy {
    pub id: String,
    pub position: Vec2,
}

/// Rolls where enemies appear; reseeded from the Run seed so a Run replays exactly.
#[derive(Resource, Debug)]
pub struct SpawnRng(pub fastrand::Rng);

impl Default for SpawnRng {
    fn default() -> Self {
        Self(fastrand::Rng::with_seed(0))
    }
}

/// Tries at a spot far enough from the player before settling for the farthest.
const SPAWN_TRIES: usize = 40;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<SpawnEnemy>()
        .init_resource::<SpawnRng>()
        .add_systems(
            Update,
            (
                (spawn_opening_wave, spawn_enemies)
                    .chain()
                    .after(start_run)
                    .in_set(GameSystemSet::InputAndIntent),
                (rusher::steer, move_enemies, rusher::touch)
                    .chain()
                    .in_set(GameSystemSet::PhysicsAndCombat),
            ),
        );
}

/// Until Rooms hold their own fights, each Run opens with one wave of
/// enemies placed away from the player.
fn spawn_opening_wave(
    mut requests: MessageReader<StartRun>,
    tuning: Res<Tuning>,
    catalog: Res<Enemies>,
    mut rng: ResMut<SpawnRng>,
    room: Single<&Room>,
    player: Single<&Position, With<Player>>,
    mut spawns: MessageWriter<SpawnEnemy>,
) {
    if requests.read().last().is_none() {
        return;
    }
    let spawn = &tuning.spawns;
    let data = catalog
        .get(&spawn.wave_enemy)
        .expect("wave enemy id in enemies.ron");
    let walkable = room.interior.inflate(-data.radius);
    for _ in 0..spawn.wave_size {
        let mut best = walkable.center();
        for _ in 0..SPAWN_TRIES {
            let candidate = walkable.min + Vec2::new(rng.0.f32(), rng.0.f32()) * walkable.size();
            if candidate.distance(player.0) > best.distance(player.0) {
                best = candidate;
            }
            if best.distance(player.0) >= spawn.min_distance {
                break;
            }
        }
        spawns.write(SpawnEnemy {
            id: spawn.wave_enemy.clone(),
            position: best,
        });
    }
}

fn spawn_enemies(
    mut requests: MessageReader<SpawnEnemy>,
    catalog: Res<Enemies>,
    mut commands: Commands,
) {
    for request in requests.read() {
        let data = catalog.get(&request.id).expect("enemy id in enemies.ron");
        let mut enemy = commands.spawn((
            Enemy {
                id: request.id.clone(),
            },
            Health::new(data.max_hp),
            data.resistances.clone(),
            HitFlash::default(),
            Position(request.position),
            Velocity::default(),
            Knockback::default(),
            RunScoped,
        ));
        match data.behaviour {
            EnemyBehaviour::Rusher(_) => enemy.insert(rusher::ContactCooldown::default()),
        };
    }
}

/// Each enemy's own velocity plus its knockback, kept inside the Room.
pub(crate) fn move_enemies(
    time: Res<Time>,
    tuning: Res<Tuning>,
    catalog: Res<Enemies>,
    room: Single<&Room>,
    mut enemies: Query<(&Enemy, &Velocity, &mut Knockback, &mut Position)>,
) {
    let dt = time.delta_secs();
    let decay = (-tuning.juice.knockback_decay * dt).exp();
    for (enemy, velocity, mut knockback, mut position) in &mut enemies {
        let data = catalog.get(&enemy.id).expect("enemy id in enemies.ron");
        position.0 += (velocity.0 + knockback.0) * dt;
        knockback.0 *= decay;
        let walkable = room.interior.inflate(-data.radius);
        position.0 = position.0.clamp(walkable.min, walkable.max);
    }
}
