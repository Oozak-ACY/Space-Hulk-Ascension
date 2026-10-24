//! Long frames are clamped; gameplay can be paused.

mod common;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use common::*;
use space_hulk_ascension::pause::PauseRequest;
use std::time::Duration;

#[test]
fn a_long_frame_is_clamped_to_the_max_frame_delta() {
    let mut app = app_in_run(1);
    let t = tuning(&app);
    set_move_intent(&mut app, Vec2::X);
    step(&mut app, 60);

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
    let before = player_position(&mut app);
    step(&mut app, 1);
    let moved = player_position(&mut app).x - before.x;
    assert!(
        moved <= t.player.move_speed * t.max_frame_delta + 1e-3,
        "moved {moved} px in one long frame"
    );
    assert!(moved > 0.0);
}

#[test]
fn pause_toggle_freezes_and_resumes_gameplay() {
    let mut app = app_in_run(1);
    set_move_intent(&mut app, Vec2::X);
    step(&mut app, 10);

    app.world_mut().write_message(PauseRequest::Toggle);
    step(&mut app, 1);
    let frozen = player_position(&mut app);
    step(&mut app, 30);
    assert_eq!(player_position(&mut app), frozen);

    app.world_mut().write_message(PauseRequest::Toggle);
    step(&mut app, 2);
    assert!(player_position(&mut app).x > frozen.x);
}

#[test]
fn pause_request_keeps_a_paused_game_paused() {
    let mut app = app_in_run(1);
    set_move_intent(&mut app, Vec2::X);
    app.world_mut().write_message(PauseRequest::Pause);
    step(&mut app, 1);
    app.world_mut().write_message(PauseRequest::Pause);
    step(&mut app, 1);
    let frozen = player_position(&mut app);
    step(&mut app, 10);
    assert_eq!(player_position(&mut app), frozen);
}
