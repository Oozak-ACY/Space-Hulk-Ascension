//! Cultist rushers: they appear away from the player, charge, and hurt on contact.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::{
    data::{Enemies, EnemyBehaviour},
    headless::FRAME,
};

fn frames(seconds: f32) -> usize {
    (seconds / FRAME.as_secs_f32()).round() as usize
}

#[test]
fn a_run_opens_with_its_wave_away_from_the_player() {
    let catalog = Enemies::bundled().unwrap();
    for seed in 0..20 {
        let mut app = app_in_run_with_wave(seed);
        let spawns = tuning(&app).spawns;
        let player = player_position(&mut app);
        let interior = room_interior(&mut app);
        let wave = enemies(&mut app);
        let mut ids: Vec<String> = wave.iter().map(|enemy| enemy.id.clone()).collect();
        ids.sort();
        let mut wanted = spawns.wave.clone();
        wanted.sort();
        assert_eq!(ids, wanted);
        assert!(ids.iter().any(|id| id == RUSHER));
        for enemy in wave {
            assert_eq!(enemy.hp, catalog.get(&enemy.id).unwrap().max_hp);
            assert!(interior.contains(enemy.position));
            assert!(
                enemy.position.distance(player) >= spawns.min_distance,
                "seed {seed}: spawned {} px away",
                enemy.position.distance(player)
            );
        }
    }
}

#[test]
fn a_rusher_charges_the_player_at_its_speed() {
    let mut app = app_in_run(1);
    spawn_enemy(&mut app, RUSHER, Vec2::new(150.0, 0.0));
    let start = only_enemy(&mut app).position;

    step(&mut app, frames(1.0));
    let after = only_enemy(&mut app).position;
    assert!(after.x < start.x, "moved toward the player");
    assert!(after.y.abs() < 1e-3, "straight at the player");
    let moved = start.x - after.x;
    assert!(moved <= 70.0 + 1e-3, "{moved} px in 1 s");
    assert!(moved > 50.0, "only {moved} px in 1 s");
}

#[test]
fn rusher_contact_hurts_the_player_once_per_cooldown() {
    let mut app = app_in_run(1);
    let EnemyBehaviour::Rusher(contact) =
        Enemies::bundled().unwrap().get(RUSHER).unwrap().behaviour
    else {
        panic!("the rusher is a Rusher");
    };
    let max_hp = tuning(&app).player.max_hp;
    spawn_enemy(&mut app, RUSHER, Vec2::new(20.0, 0.0));

    // Reach the player, hurt them once…
    let mut frame = 0;
    while player_health(&mut app).current() == max_hp {
        step(&mut app, 1);
        frame += 1;
        assert!(frame < frames(2.0), "never hurt the player");
    }
    assert_eq!(player_health(&mut app).current(), max_hp - contact.damage);

    // …and not again until both its contact cooldown and the player's
    // post-hit invulnerability are over, however close it stays.
    let mut hurt_again_after = 0;
    while player_health(&mut app).current() == max_hp - contact.damage {
        step(&mut app, 1);
        let rusher = only_enemy(&mut app).position;
        set_player_position(&mut app, rusher);
        hurt_again_after += 1;
        assert!(hurt_again_after < frames(3.0), "never hurt again");
    }
    let waited = hurt_again_after as f32 * FRAME.as_secs_f32();
    assert!(
        waited >= contact.cooldown - FRAME.as_secs_f32(),
        "hurt again after {waited} s"
    );
}
