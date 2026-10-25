//! The Mortal's Dodge Roll: a fast, briefly invincible dash on a cooldown.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::headless::FRAME;

/// Whole frames that fit in `seconds`.
fn frames(seconds: f32) -> usize {
    (seconds / FRAME.as_secs_f32()) as usize
}

#[test]
fn rolling_follows_the_movement_direction() {
    let mut app = app_in_run(1);
    set_aim_intent(&mut app, Vec2::X);
    set_move_intent(&mut app, Vec2::new(0.0, 1.0));
    let start = player_position(&mut app);

    request_dodge_roll(&mut app);
    step(&mut app, 1);
    let direction = (player_position(&mut app) - start).normalize();
    assert!(
        direction.distance(Vec2::Y) < 1e-4,
        "rolled toward {direction}"
    );
}

#[test]
fn standing_still_rolls_toward_the_aim() {
    let mut app = app_in_run(1);
    let aim = Vec2::new(-3.0, 4.0);
    set_aim_intent(&mut app, aim);
    let start = player_position(&mut app);

    request_dodge_roll(&mut app);
    step(&mut app, 1);
    let direction = (player_position(&mut app) - start).normalize();
    assert!(
        direction.distance(aim.normalize()) < 1e-4,
        "rolled toward {direction}"
    );
}

#[test]
fn with_no_movement_or_aim_the_first_roll_faces_the_camera() {
    let mut app = app_in_run(1);
    let start = player_position(&mut app);

    request_dodge_roll(&mut app);
    step(&mut app, 1);
    let travel = player_position(&mut app) - start;
    assert!(travel.length() > 0.0, "a pressed roll must fire");
    assert!(
        travel.normalize().distance(Vec2::NEG_Y) < 1e-4,
        "rolled toward {travel}"
    );
}

#[test]
fn with_no_movement_or_aim_a_roll_repeats_the_last_direction() {
    let mut app = app_in_run(1);
    let roll = tuning(&app).player.dodge_roll;
    set_move_intent(&mut app, Vec2::X);
    request_dodge_roll(&mut app);
    step(&mut app, 1);
    set_move_intent(&mut app, Vec2::ZERO);
    step(&mut app, frames(roll.cooldown) + 30);

    let start = player_position(&mut app);
    request_dodge_roll(&mut app);
    step(&mut app, 1);
    let direction = (player_position(&mut app) - start).normalize();
    assert!(
        direction.distance(Vec2::X) < 1e-4,
        "rolled toward {direction}"
    );
}

#[test]
fn a_roll_covers_speed_times_duration_at_roll_speed() {
    let mut app = app_in_run(1);
    let roll = tuning(&app).player.dodge_roll;
    set_aim_intent(&mut app, Vec2::X);
    let start = player_position(&mut app);

    request_dodge_roll(&mut app);
    step(&mut app, 1);
    let first_frame = (player_position(&mut app) - start).length() / FRAME.as_secs_f32();
    assert!(
        (first_frame - roll.speed).abs() < 1e-2,
        "rolling at {first_frame} px/s"
    );

    step(&mut app, frames(roll.duration) + 1);
    assert!(!dodge_roll_state(&mut app).is_rolling(&roll));
    let distance = (player_position(&mut app) - start).x;
    let expected = roll.speed * roll.duration;
    // Afterwards the player only eases to a stop at walking speed.
    let walk = tuning(&app).player.move_speed * FRAME.as_secs_f32();
    assert!(
        distance >= expected - 1e-2 && distance < expected + walk,
        "rolled {distance} px, expected {expected}"
    );
}

#[test]
fn rolling_ignores_steering_until_it_ends() {
    let mut app = app_in_run(1);
    let roll = tuning(&app).player.dodge_roll;
    set_move_intent(&mut app, Vec2::X);
    request_dodge_roll(&mut app);
    step(&mut app, 1);

    set_move_intent(&mut app, Vec2::Y);
    let before = player_position(&mut app);
    step(&mut app, frames(roll.duration) - 2);
    let travel = player_position(&mut app) - before;
    assert!(travel.y.abs() < 1e-4, "steered by {travel}");
}

#[test]
fn invincibility_covers_the_window_from_the_roll_start() {
    let mut app = app_in_run(1);
    let roll = tuning(&app).player.dodge_roll;
    set_aim_intent(&mut app, Vec2::X);
    assert!(!dodge_roll_state(&mut app).is_invincible(&roll));

    request_dodge_roll(&mut app);
    step(&mut app, 1);
    assert!(dodge_roll_state(&mut app).is_invincible(&roll));

    // Still invincible after the roll's movement has ended…
    step(&mut app, frames(roll.invincibility) - 2);
    let state = dodge_roll_state(&mut app);
    assert!(!state.is_rolling(&roll));
    assert!(state.is_invincible(&roll));

    // …but not past the window.
    step(&mut app, 2);
    assert!(!dodge_roll_state(&mut app).is_invincible(&roll));
}

#[test]
fn a_second_roll_waits_out_the_cooldown() {
    let mut app = app_in_run(1);
    let roll = tuning(&app).player.dodge_roll;
    set_aim_intent(&mut app, Vec2::X);
    request_dodge_roll(&mut app);
    step(&mut app, 1);
    assert!(!dodge_roll_state(&mut app).is_ready(&roll));

    // Pressing during the cooldown is dropped, not queued.
    step(&mut app, frames(roll.duration) + 1);
    request_dodge_roll(&mut app);
    step(&mut app, 1);
    assert!(!dodge_roll_state(&mut app).is_rolling(&roll));

    step(&mut app, frames(roll.cooldown) - frames(roll.duration) - 4);
    let state = dodge_roll_state(&mut app);
    assert!(!state.is_ready(&roll));
    assert!(state.cooldown_progress(&roll) > 0.9);
    assert!(state.cooldown_progress(&roll) < 1.0);
    step(&mut app, 3);
    assert!(dodge_roll_state(&mut app).is_ready(&roll));
    assert!(!dodge_roll_state(&mut app).is_rolling(&roll));

    request_dodge_roll(&mut app);
    step(&mut app, 1);
    assert!(dodge_roll_state(&mut app).is_rolling(&roll));
}

#[test]
fn a_fresh_run_has_the_roll_ready() {
    let mut app = app_in_run(1);
    let roll = tuning(&app).player.dodge_roll;
    let state = dodge_roll_state(&mut app);
    assert!(state.is_ready(&roll));
    assert_eq!(state.cooldown_progress(&roll), 1.0);
}
