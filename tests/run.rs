//! The "start Run" request resets the Run and records its seed.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::{headless, run::RunSeed};

#[test]
fn starting_a_run_spawns_one_player_and_records_the_seed() {
    let mut app = app_in_run(42);
    assert_eq!(app.world().resource::<RunSeed>().0, 42);
    assert_eq!(player_count(&mut app), 1);
}

#[test]
fn starting_a_new_run_resets_the_previous_one() {
    let mut app = app_in_run(1);
    let start = player_position(&mut app);
    set_move_intent(&mut app, Vec2::X);
    step(&mut app, 30);
    assert_ne!(player_position(&mut app), start);

    start_run(&mut app, Some(2));
    assert_eq!(player_count(&mut app), 1);
    assert_eq!(player_position(&mut app), start);
    assert_eq!(app.world().resource::<RunSeed>().0, 2);
}

#[test]
fn a_run_without_a_seed_picks_one() {
    let mut app = headless::app();
    start_run(&mut app, None);
    assert!(app.world().get_resource::<RunSeed>().is_some());
}
