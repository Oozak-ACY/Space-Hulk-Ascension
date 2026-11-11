//! Fatal damage passes through death interception, then Game over; one
//! request starts a fresh Run.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::{
    combat::{DamageType, FatalDamage},
    pause::PauseRequest,
    run::{AppState, Depth},
};

fn state(app: &App) -> AppState {
    *app.world().resource::<State<AppState>>().get()
}

fn kill_player(app: &mut App) {
    let player = player(app);
    request_damage(app, player, 1000.0, DamageType::Physical);
    step(app, 2);
}

#[test]
fn a_run_is_played_in_game_from_the_first_floor() {
    let app = app_in_run(1);
    assert_eq!(state(&app), AppState::InGame);
    assert_eq!(app.world().resource::<Depth>().floor, 1);
}

#[test]
fn surviving_a_hit_is_not_death() {
    let mut app = app_in_run(1);
    let player = player(&mut app);
    let mut fatal = MessageLog::<FatalDamage>::open(&app);
    request_damage(&mut app, player, 99.0, DamageType::Physical);
    step(&mut app, 2);
    assert!(fatal.read(&app).is_empty());
    assert_eq!(state(&app), AppState::InGame);
}

#[test]
fn reaching_zero_hp_raises_fatal_damage_then_game_over() {
    let mut app = app_in_run(1);
    let player = player(&mut app);
    let mut fatal = MessageLog::<FatalDamage>::open(&app);

    kill_player(&mut app);
    assert_eq!(player_health(&mut app).current(), 0.0);
    assert_eq!(fatal.read(&app), vec![FatalDamage { entity: player }]);
    assert_eq!(state(&app), AppState::GameOver);
    assert_eq!(
        app.world().resource::<Depth>().floor,
        1,
        "shows how deep the player got"
    );
}

#[test]
fn several_blows_on_the_same_frame_are_one_death() {
    let mut app = app_in_run(1);
    let player = player(&mut app);
    let mut fatal = MessageLog::<FatalDamage>::open(&app);
    request_damage(&mut app, player, 1000.0, DamageType::Physical);
    request_damage(&mut app, player, 1000.0, DamageType::Energy);
    step(&mut app, 2);
    assert_eq!(fatal.read(&app).len(), 1);
}

#[test]
fn gameplay_stops_on_game_over() {
    let mut app = app_in_run(1);
    spawn_enemy(&mut app, RUSHER, Vec2::new(150.0, 0.0));
    kill_player(&mut app);

    let rusher = only_enemy(&mut app).position;
    let player = player_position(&mut app);
    set_move_intent(&mut app, Vec2::X);
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);
    step(&mut app, 30);
    assert_eq!(only_enemy(&mut app).position, rusher);
    assert_eq!(player_position(&mut app), player);
    assert!(projectiles(&mut app).is_empty());
}

#[test]
fn a_new_run_from_game_over_starts_fresh() {
    let mut app = app_in_run_with_wave(1);
    let t = tuning(&app);
    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);
    step(&mut app, 5);
    kill_player(&mut app);
    assert_eq!(state(&app), AppState::GameOver);

    start_run(&mut app, Some(2));
    step(&mut app, 1);
    assert_eq!(state(&app), AppState::InGame);
    assert_eq!(player_count(&mut app), 1);
    let health = player_health(&mut app);
    assert_eq!(health.current(), t.player.max_hp);
    let weapon = player_weapon(&mut app);
    assert_eq!(weapon.id(), t.player.starting_weapon);
    assert_eq!(enemies(&mut app).len(), t.spawns.wave.len());

    // And it plays: the fresh player walks.
    set_move_intent(&mut app, Vec2::X);
    let before = player_position(&mut app);
    step(&mut app, 2);
    assert!(player_position(&mut app).x > before.x);
}

#[test]
fn a_new_run_is_never_paused() {
    let mut app = app_in_run(1);
    kill_player(&mut app);
    app.world_mut().write_message(PauseRequest::Toggle);
    step(&mut app, 1);

    start_run(&mut app, Some(2));
    set_move_intent(&mut app, Vec2::X);
    let before = player_position(&mut app);
    step(&mut app, 2);
    assert!(
        player_position(&mut app).x > before.x,
        "the new Run is paused"
    );
}
