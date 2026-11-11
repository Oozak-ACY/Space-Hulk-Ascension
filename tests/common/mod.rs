//! Helpers that drive the headless game app the way a player would.

#![allow(dead_code)]

use bevy::{
    ecs::message::{MessageCursor, Messages},
    prelude::*,
};
use space_hulk_ascension::{
    combat::{DamageRequest, DamageType, FireIntent, Health, Projectile, Weapon},
    data::{Tuning, WeaponData, Weapons},
    enemies::{Enemy, SpawnEnemy},
    headless,
    kinematics::{Position, Velocity},
    player::{AimIntent, DodgeIntent, DodgeRoll, MoveIntent, Player},
    run::StartRun,
    world::room::Room,
};
use std::collections::HashSet;

/// A headless app with a Run started from `seed`, its opening wave
/// removed so the Room is quiet.
pub fn app_in_run(seed: u64) -> App {
    let mut app = app_in_run_with_wave(seed);
    despawn_enemies(&mut app);
    app
}

/// A headless app with a Run started from `seed`, opening wave and all.
pub fn app_in_run_with_wave(seed: u64) -> App {
    let mut app = headless::app();
    start_run(&mut app, Some(seed));
    app
}

pub fn start_run(app: &mut App, seed: Option<u64>) {
    app.world_mut().write_message(StartRun { seed });
    app.update();
}

pub fn step(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

pub fn tuning(app: &App) -> Tuning {
    app.world().resource::<Tuning>().clone()
}

pub fn player(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<Player>>()
        .single(app.world())
        .expect("exactly one player")
}

pub fn player_count(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), With<Player>>()
        .iter(app.world())
        .count()
}

pub fn player_position(app: &mut App) -> Vec2 {
    let player = player(app);
    app.world().get::<Position>(player).unwrap().0
}

pub fn set_move_intent(app: &mut App, direction: Vec2) {
    let player = player(app);
    app.world_mut().get_mut::<MoveIntent>(player).unwrap().0 = direction;
}

pub fn set_aim_intent(app: &mut App, direction: Vec2) {
    let player = player(app);
    app.world_mut()
        .get_mut::<AimIntent>(player)
        .unwrap()
        .direction = direction;
}

/// Presses the Dodge Roll button for the next frame.
pub fn request_dodge_roll(app: &mut App) {
    let player = player(app);
    app.world_mut().get_mut::<DodgeIntent>(player).unwrap().0 = true;
}

pub fn dodge_roll_state(app: &mut App) -> DodgeRoll {
    let player = player(app);
    *app.world().get::<DodgeRoll>(player).unwrap()
}

/// Holds (or releases) the fire button.
pub fn set_fire_intent(app: &mut App, held: bool) {
    let player = player(app);
    app.world_mut().get_mut::<FireIntent>(player).unwrap().0 = held;
}

pub fn player_weapon(app: &mut App) -> Weapon {
    let player = player(app);
    app.world().get::<Weapon>(player).unwrap().clone()
}

pub fn weapon_data(app: &App, id: &str) -> WeaponData {
    app.world()
        .resource::<Weapons>()
        .get(id)
        .expect("weapon in weapons.ron")
        .clone()
}

pub fn room_interior(app: &mut App) -> Rect {
    app.world_mut()
        .query::<&Room>()
        .single(app.world())
        .expect("exactly one room")
        .interior
}

#[derive(Debug, Clone)]
pub struct ProjectileState {
    pub entity: Entity,
    pub projectile: Projectile,
    pub position: Vec2,
    pub velocity: Vec2,
}

pub fn projectiles(app: &mut App) -> Vec<ProjectileState> {
    app.world_mut()
        .query::<(Entity, &Projectile, &Position, &Velocity)>()
        .iter(app.world())
        .map(|(entity, projectile, position, velocity)| ProjectileState {
            entity,
            projectile: projectile.clone(),
            position: position.0,
            velocity: velocity.0,
        })
        .collect()
}

pub fn only_projectile(app: &mut App) -> ProjectileState {
    let mut all = projectiles(app);
    assert_eq!(all.len(), 1, "expected exactly one projectile");
    all.remove(0)
}

/// Steps `frames` frames; returns the index of each frame that fired a shot.
pub fn step_counting_shots(app: &mut App, frames: usize) -> Vec<usize> {
    let mut seen: HashSet<Entity> = projectiles(app).iter().map(|p| p.entity).collect();
    let mut fired = Vec::new();
    for frame in 0..frames {
        app.update();
        for projectile in projectiles(app) {
            if seen.insert(projectile.entity) {
                fired.push(frame);
            }
        }
    }
    fired
}

pub const RUSHER: &str = "cultist_rusher";

#[derive(Debug, Clone)]
pub struct EnemyState {
    pub entity: Entity,
    pub id: String,
    pub position: Vec2,
    pub hp: f32,
}

pub fn enemies(app: &mut App) -> Vec<EnemyState> {
    app.world_mut()
        .query::<(Entity, &Enemy, &Position, &Health)>()
        .iter(app.world())
        .map(|(entity, enemy, position, health)| EnemyState {
            entity,
            id: enemy.id().to_owned(),
            position: position.0,
            hp: health.current(),
        })
        .collect()
}

pub fn only_enemy(app: &mut App) -> EnemyState {
    let mut all = enemies(app);
    assert_eq!(all.len(), 1, "expected exactly one enemy");
    all.remove(0)
}

/// Spawns an enemy, which acts from the next frame on.
pub fn spawn_enemy(app: &mut App, id: &str, position: Vec2) -> Entity {
    app.world_mut().write_message(SpawnEnemy {
        id: id.to_owned(),
        position,
    });
    app.update();
    let spawned = enemies(app);
    spawned
        .iter()
        .find(|enemy| enemy.id == id)
        .map(|enemy| enemy.entity)
        .expect("the enemy spawned")
}

pub fn despawn_enemies(app: &mut App) {
    let all: Vec<Entity> = enemies(app).iter().map(|e| e.entity).collect();
    for entity in all {
        app.world_mut().despawn(entity);
    }
}

pub fn player_health(app: &mut App) -> Health {
    let player = player(app);
    *app.world().get::<Health>(player).unwrap()
}

pub fn set_player_position(app: &mut App, position: Vec2) {
    let player = player(app);
    app.world_mut().get_mut::<Position>(player).unwrap().0 = position;
}

/// Asks for `amount` of `damage_type` damage to `target`, applied on the next frame.
pub fn request_damage(app: &mut App, target: Entity, amount: f32, damage_type: DamageType) {
    app.world_mut().write_message(DamageRequest {
        target,
        amount,
        damage_type,
        weapon: None,
    });
}

/// Every message of type `M` written since the log was opened or last read.
pub struct MessageLog<M: Message> {
    cursor: MessageCursor<M>,
}

impl<M: Message + Clone> MessageLog<M> {
    /// Starts after the messages already written.
    pub fn open(app: &App) -> Self {
        Self {
            cursor: app.world().resource::<Messages<M>>().get_cursor_current(),
        }
    }

    pub fn read(&mut self, app: &App) -> Vec<M> {
        let messages = app.world().resource::<Messages<M>>();
        self.cursor.read(messages).cloned().collect()
    }
}
