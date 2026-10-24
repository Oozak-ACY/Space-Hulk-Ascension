//! Helpers that drive the headless game app the way a player would.

#![allow(dead_code)]

use bevy::prelude::*;
use space_hulk_ascension::{
    data::Tuning,
    headless,
    kinematics::Position,
    player::{MoveIntent, Player},
    run::StartRun,
};

/// A headless app with a Run started from `seed`.
pub fn app_in_run(seed: u64) -> App {
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
