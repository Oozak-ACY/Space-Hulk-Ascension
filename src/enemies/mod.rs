//! Enemies: spawning them, moving them, and how each kind fights.

pub mod gunner;
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

/// Rolls enemies' choices (when a gunner switches strafing direction…);
/// reseeded from the Run seed so a Run replays exactly.
#[derive(Resource, Debug)]
pub struct AiRng(pub fastrand::Rng);

impl Default for AiRng {
    fn default() -> Self {
        Self(fastrand::Rng::with_seed(0))
    }
}

/// Tries at a spot far enough from the player before settling for the farthest.
const SPAWN_TRIES: usize = 40;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<SpawnEnemy>()
        .init_resource::<SpawnRng>()
        .init_resource::<AiRng>()
        .add_systems(
            Update,
            (
                (spawn_opening_wave, spawn_enemies)
                    .chain()
                    .after(start_run)
                    .in_set(GameSystemSet::InputAndIntent),
                (
                    rusher::steer,
                    gunner::steer,
                    move_enemies,
                    separate_enemies,
                    rusher::touch,
                    gunner::shoot,
                )
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
    for id in &spawn.wave {
        let data = catalog.get(id).expect("wave enemy id in enemies.ron");
        let walkable = room.interior.inflate(-data.radius);
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
            id: id.clone(),
            position: best,
        });
    }
}

fn spawn_enemies(
    mut requests: MessageReader<SpawnEnemy>,
    catalog: Res<Enemies>,
    mut rng: ResMut<AiRng>,
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
            EnemyBehaviour::Gunner(gunner) => {
                enemy.insert(gunner::Gunner::new(&gunner, &mut rng.0))
            }
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

/// Overlapping enemies are pushed apart, each by half the overlap, so groups
/// never stack. Two on the very same spot part along X.
pub(crate) fn separate_enemies(
    catalog: Res<Enemies>,
    room: Single<&Room>,
    mut enemies: Query<(&Enemy, &mut Position)>,
) {
    let mut pairs = enemies.iter_combinations_mut();
    while let Some([(a, mut at_a), (b, mut at_b)]) = pairs.fetch_next() {
        let radius_a = catalog.get(a.id()).expect("enemy id in enemies.ron").radius;
        let radius_b = catalog.get(b.id()).expect("enemy id in enemies.ron").radius;
        let offset = at_b.0 - at_a.0;
        let overlap = radius_a + radius_b - offset.length();
        if overlap <= 0.0 {
            continue;
        }
        let apart = offset.try_normalize().unwrap_or(Vec2::X) * overlap / 2.0;
        at_a.0 = (at_a.0 - apart).clamp(room.interior.min + radius_a, room.interior.max - radius_a);
        at_b.0 = (at_b.0 + apart).clamp(room.interior.min + radius_b, room.interior.max - radius_b);
    }
}
