//! Screen shake and freeze frames on hits, kills and being hurt.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::{
    combat::DamageType,
    headless::FRAME,
    juice::{FreezeFrame, ScreenShake},
    pause::PauseRequest,
};

fn shake(app: &App) -> f32 {
    app.world().resource::<ScreenShake>().0
}

fn freeze(app: &App) -> f32 {
    app.world().resource::<FreezeFrame>().0
}

/// Frames during which gameplay time stands still for a freeze of `seconds`.
fn frozen_frames(seconds: f32) -> usize {
    (seconds / FRAME.as_secs_f32()).ceil() as usize
}

/// Walks right and checks the player stands still for exactly `frozen` frames.
fn assert_frozen_for(app: &mut App, frozen: usize) {
    for frame in 0..frozen {
        let before = player_position(app);
        step(app, 1);
        assert_eq!(
            player_position(app),
            before,
            "moved on frozen frame {frame}"
        );
    }
    let before = player_position(app);
    step(app, 1);
    assert!(
        player_position(app).x > before.x,
        "still frozen after {frozen} frames"
    );
}

#[test]
fn a_quiet_run_neither_shakes_nor_freezes() {
    let mut app = app_in_run(1);
    step(&mut app, 10);
    assert_eq!(shake(&app), 0.0);
    assert_eq!(freeze(&app), 0.0);
}

#[test]
fn a_hit_shakes_the_screen_a_little_without_freezing() {
    let mut app = app_in_run(1);
    let juice = tuning(&app).juice;
    let rusher = spawn_enemy(&mut app, RUSHER, Vec2::new(150.0, 0.0));
    request_damage(&mut app, rusher, 1.0, DamageType::Energy);
    step(&mut app, 1);

    assert_eq!(shake(&app), juice.shake_on_hit);
    assert_eq!(freeze(&app), 0.0);
    step(&mut app, 30);
    assert_eq!(shake(&app), 0.0, "the shake dies down");
}

#[test]
fn a_kill_shakes_the_screen_and_freezes_gameplay_briefly() {
    let mut app = app_in_run(1);
    let juice = tuning(&app).juice;
    let rusher = spawn_enemy(&mut app, RUSHER, Vec2::new(150.0, 0.0));
    set_move_intent(&mut app, Vec2::X);
    request_damage(&mut app, rusher, 100.0, DamageType::Energy);
    step(&mut app, 1);

    assert!(enemies(&mut app).is_empty());
    assert_eq!(shake(&app), juice.shake_on_kill);
    assert_eq!(freeze(&app), juice.freeze_on_kill);
    assert_frozen_for(&mut app, frozen_frames(juice.freeze_on_kill));
}

#[test]
fn being_hurt_shakes_the_screen_hard_and_freezes_gameplay_longer() {
    let mut app = app_in_run(1);
    let juice = tuning(&app).juice;
    let player = player(&mut app);
    set_move_intent(&mut app, Vec2::X);
    request_damage(&mut app, player, 10.0, DamageType::Physical);
    step(&mut app, 1);

    assert_eq!(shake(&app), juice.shake_on_hurt);
    assert_eq!(freeze(&app), juice.freeze_on_hurt);
    assert_frozen_for(&mut app, frozen_frames(juice.freeze_on_hurt));
}

#[test]
fn a_new_run_starts_without_shake_or_freeze() {
    let mut app = app_in_run(1);
    let player = player(&mut app);
    request_damage(&mut app, player, 10.0, DamageType::Physical);
    step(&mut app, 1);
    assert!(freeze(&app) > 0.0);

    start_run(&mut app, Some(2));
    assert_eq!(shake(&app), 0.0);
    assert_eq!(freeze(&app), 0.0);
    set_move_intent(&mut app, Vec2::X);
    let before = player_position(&mut app);
    step(&mut app, 1);
    assert!(
        player_position(&mut app).x > before.x,
        "a new Run is not frozen"
    );
}

#[test]
fn the_shake_dies_down_on_the_game_over_screen() {
    let mut app = app_in_run(1);
    let player = player(&mut app);
    request_damage(&mut app, player, 1000.0, DamageType::Physical);
    step(&mut app, 2);
    assert!(shake(&app) > 0.0);

    step(&mut app, 30);
    assert_eq!(shake(&app), 0.0);
}

#[test]
fn the_shake_dies_down_while_paused() {
    let mut app = app_in_run(1);
    let player = player(&mut app);
    request_damage(&mut app, player, 10.0, DamageType::Physical);
    step(&mut app, 1);
    app.world_mut().write_message(PauseRequest::Toggle);
    step(&mut app, 30);
    assert_eq!(shake(&app), 0.0);
}
