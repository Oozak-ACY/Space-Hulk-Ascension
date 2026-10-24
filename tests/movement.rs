//! The Mortal walks with the validated feel and is stopped by the Room's walls.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::headless::FRAME;

#[test]
fn player_accelerates_with_snappiness_toward_move_speed() {
    let mut app = app_in_run(1);
    let speed = tuning(&app).player.move_speed;
    let snappiness = tuning(&app).player.snappiness;
    let dt = FRAME.as_secs_f32();

    set_move_intent(&mut app, Vec2::X);
    let start = player_position(&mut app);
    step(&mut app, 1);
    let first_step = player_position(&mut app).x - start.x;
    let expected_first = speed * (1.0 - (-snappiness * dt).exp()) * dt;
    assert!(
        (first_step - expected_first).abs() < 1e-3,
        "first step {first_step}, expected {expected_first}"
    );

    step(&mut app, 59);
    let before = player_position(&mut app);
    step(&mut app, 1);
    let cruising = (player_position(&mut app) - before).length() / dt;
    assert!(
        (cruising - speed).abs() < 0.5,
        "cruising at {cruising} px/s"
    );
}

#[test]
fn diagonal_movement_is_no_faster_than_straight() {
    let mut app = app_in_run(1);
    let speed = tuning(&app).player.move_speed;
    set_move_intent(&mut app, Vec2::new(1.0, 1.0));
    step(&mut app, 60);
    let before = player_position(&mut app);
    step(&mut app, 1);
    let cruising = (player_position(&mut app) - before).length() / FRAME.as_secs_f32();
    assert!(
        (cruising - speed).abs() < 0.5,
        "diagonal cruising at {cruising} px/s"
    );
}

#[test]
fn no_intent_brings_the_player_to_a_stop() {
    let mut app = app_in_run(1);
    set_move_intent(&mut app, Vec2::X);
    step(&mut app, 30);
    set_move_intent(&mut app, Vec2::ZERO);
    step(&mut app, 60);
    let before = player_position(&mut app);
    step(&mut app, 1);
    assert!((player_position(&mut app) - before).length() < 1e-3);
}

#[test]
fn walls_keep_the_player_inside_the_room_interior() {
    let mut app = app_in_run(1);
    let t = tuning(&app);
    let max = t.room.interior_size() / 2.0 - Vec2::splat(t.player.radius);
    let centre = player_position(&mut app);

    for (direction, expected) in [
        (Vec2::X, Vec2::new(max.x, 0.0)),
        (Vec2::NEG_X, Vec2::new(-max.x, 0.0)),
        (Vec2::Y, Vec2::new(0.0, max.y)),
        (Vec2::NEG_Y, Vec2::new(0.0, -max.y)),
    ] {
        start_run(&mut app, Some(1));
        set_move_intent(&mut app, direction);
        step(&mut app, 600);
        let offset = player_position(&mut app) - centre;
        assert!(
            (offset - expected).length() < 1e-3,
            "pushing {direction}: ended at {offset}, expected {expected}"
        );
    }
}
