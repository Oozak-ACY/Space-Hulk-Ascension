//! The Mortal's laspistol: held fire on an interval, las bolts that vanish at
//! the walls, and the overheat lockout.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::{
    combat::DamageType,
    data::{OverheatData, WeaponData, WeaponProfile},
    headless::FRAME,
    pause::PauseRequest,
};

fn frames(seconds: f32) -> usize {
    (seconds / FRAME.as_secs_f32()).round() as usize
}

fn laspistol(app: &App) -> WeaponData {
    weapon_data(app, "laspistol")
}

fn overheat(weapon: &WeaponData) -> OverheatData {
    match weapon.profile {
        WeaponProfile::Overheat(overheat) => overheat,
    }
}

#[test]
fn a_fresh_run_holds_a_cool_laspistol() {
    let mut app = app_in_run(1);
    let weapon = player_weapon(&mut app);
    assert_eq!(weapon.id(), "laspistol");
    assert_eq!(weapon.heat(), 0.0);
    assert!(!weapon.is_overheated());
}

#[test]
fn holding_fire_shoots_once_per_interval() {
    let mut app = app_in_run(1);
    let interval = laspistol(&app).interval;
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);

    let shots = step_counting_shots(&mut app, frames(2.0));
    assert_eq!(shots[0], 0, "the first shot leaves on the first frame");
    let gaps: Vec<f32> = shots
        .windows(2)
        .map(|w| (w[1] - w[0]) as f32 * FRAME.as_secs_f32())
        .collect();
    for gap in &gaps {
        assert!(
            *gap >= interval - FRAME.as_secs_f32() && *gap <= interval + FRAME.as_secs_f32(),
            "{gap} s between shots"
        );
    }
    let mean = (shots.last().unwrap() - shots[0]) as f32 * FRAME.as_secs_f32() / gaps.len() as f32;
    assert!(
        (mean - interval).abs() < 1e-2,
        "fired every {mean} s on average"
    );
}

#[test]
fn releasing_fire_stops_shooting() {
    let mut app = app_in_run(1);
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);
    step(&mut app, 1);
    set_fire_intent(&mut app, false);
    assert!(step_counting_shots(&mut app, frames(1.0)).is_empty());
}

#[test]
fn bolts_fly_along_the_aim_within_the_spread() {
    let mut app = app_in_run(1);
    let weapon = laspistol(&app);
    let aim = Vec2::new(1.0, 1.0);
    set_aim_intent(&mut app, aim);
    set_fire_intent(&mut app, true);
    step(&mut app, 1);

    let bolt = only_projectile(&mut app);
    assert!((bolt.velocity.length() - weapon.speed).abs() < 1e-2);
    let deviation = bolt.velocity.angle_to(aim).to_degrees().abs();
    assert!(
        deviation <= weapon.spread + 1e-3,
        "{deviation}° off the aim"
    );
    assert_eq!(bolt.projectile.damage, weapon.damage);
    assert_eq!(bolt.projectile.damage_type, DamageType::Energy);
}

/// Spread is ± its value: shots stray up to that many degrees either side.
#[test]
fn spread_scatters_shots_up_to_its_angle_either_side() {
    let spread = laspistol(&app_in_run(1)).spread;
    let deviations: Vec<f32> = (0..40)
        .map(|seed| {
            let mut app = app_in_run(seed);
            set_aim_intent(&mut app, Vec2::X);
            set_fire_intent(&mut app, true);
            step(&mut app, 1);
            only_projectile(&mut app).velocity.to_angle().to_degrees()
        })
        .collect();

    assert!(deviations.iter().all(|d| d.abs() <= spread + 1e-3));
    assert!(
        deviations.iter().any(|d| *d > spread / 2.0),
        "none strayed far left"
    );
    assert!(
        deviations.iter().any(|d| *d < -spread / 2.0),
        "none strayed far right"
    );
}

#[test]
fn each_shot_adds_heat_which_cools_over_time() {
    let mut app = app_in_run(1);
    let overheat = overheat(&laspistol(&app));
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);
    step(&mut app, 1);
    set_fire_intent(&mut app, false);
    assert_eq!(player_weapon(&mut app).heat(), overheat.heat_per_shot);

    step(&mut app, frames(0.25));
    let expected =
        overheat.heat_per_shot - overheat.cooling * frames(0.25) as f32 * FRAME.as_secs_f32();
    let heat = player_weapon(&mut app).heat();
    assert!(
        (heat - expected).abs() < 1e-2,
        "heat {heat}, expected {expected}"
    );

    step(&mut app, frames(1.0));
    assert_eq!(player_weapon(&mut app).heat(), 0.0);
}

#[test]
fn reaching_max_heat_locks_firing_then_resets_heat() {
    let mut app = app_in_run(1);
    let overheat = overheat(&laspistol(&app));
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);

    let mut held = 0;
    while !player_weapon(&mut app).is_overheated() {
        step(&mut app, 1);
        held += 1;
        assert!(held < frames(10.0), "never overheated");
    }
    assert_eq!(player_weapon(&mut app).heat(), overheat.max_heat);

    // Still holding fire: nothing leaves the barrel during the lockout…
    let lockout = frames(overheat.lockout);
    assert!(step_counting_shots(&mut app, lockout - 1).is_empty());
    assert!(player_weapon(&mut app).is_overheated());

    // …then the laspistol is cool and fires straight away.
    let shots = step_counting_shots(&mut app, 2);
    let weapon = player_weapon(&mut app);
    assert!(!weapon.is_overheated());
    assert_eq!(shots.len(), 1, "fires once the lockout ends");
    assert!(weapon.heat() <= overheat.heat_per_shot);
}

#[test]
fn continuous_fire_overheats_after_the_expected_burst() {
    let mut app = app_in_run(1);
    let weapon = laspistol(&app);
    let overheat = overheat(&weapon);
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);

    let mut shots = 0;
    while !player_weapon(&mut app).is_overheated() {
        shots += step_counting_shots(&mut app, 1).len();
        assert!(shots < 100, "never overheated");
    }
    // Each shot after the first nets heat_per_shot minus what cools in one interval.
    let net = overheat.heat_per_shot - overheat.cooling * weapon.interval;
    let expected = 1 + ((overheat.max_heat - overheat.heat_per_shot) / net).ceil() as usize;
    assert!(
        shots.abs_diff(expected) <= 1,
        "overheated after {shots} shots, expected about {expected}"
    );
}

#[test]
fn bolts_vanish_at_the_room_wall() {
    let mut app = app_in_run(1);
    let weapon = laspistol(&app);
    let interior = room_interior(&mut app);
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);
    step(&mut app, 1);
    set_fire_intent(&mut app, false);
    assert_eq!(projectiles(&mut app).len(), 1);

    let to_wall = interior.max.x / weapon.speed;
    step(&mut app, frames(to_wall) - 2);
    let bolt = only_projectile(&mut app);
    assert!(interior.contains(bolt.position));

    step(&mut app, 4);
    assert!(projectiles(&mut app).is_empty());
}

#[test]
fn cannot_shoot_while_dodge_rolling() {
    let mut app = app_in_run(1);
    let roll = tuning(&app).player.dodge_roll;
    set_aim_intent(&mut app, Vec2::X);
    request_dodge_roll(&mut app);
    step(&mut app, 1);

    set_fire_intent(&mut app, true);
    let mut shots_while_rolling = 0;
    while dodge_roll_state(&mut app).is_rolling(&roll) {
        shots_while_rolling += step_counting_shots(&mut app, 1).len();
    }
    assert_eq!(shots_while_rolling, 0);
    assert_eq!(player_weapon(&mut app).heat(), 0.0);

    assert_eq!(
        step_counting_shots(&mut app, 1).len(),
        1,
        "fires once the roll ends"
    );
}

#[test]
fn cannot_shoot_while_paused() {
    let mut app = app_in_run(1);
    set_aim_intent(&mut app, Vec2::X);
    app.world_mut().write_message(PauseRequest::Toggle);
    step(&mut app, 1);

    set_fire_intent(&mut app, true);
    assert!(step_counting_shots(&mut app, 10).is_empty());
}

#[test]
fn a_new_run_clears_bolts_and_heat() {
    let mut app = app_in_run(1);
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);
    step(&mut app, 3);
    assert!(!projectiles(&mut app).is_empty());

    start_run(&mut app, Some(2));
    assert!(projectiles(&mut app).is_empty());
    assert_eq!(player_weapon(&mut app).heat(), 0.0);
}
