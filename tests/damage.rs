//! Shots hitting enemies, damage and its mitigation, and post-hit invulnerability.

mod common;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use common::*;
use space_hulk_ascension::{
    combat::{
        DamageTaken, DamageType, EnemyDamaged, EnemyKilled, HitFlash, HurtInvulnerability,
        Resistances,
    },
    enemies::Knockback,
    headless::FRAME,
};
use std::time::Duration;

fn frames(seconds: f32) -> usize {
    (seconds / FRAME.as_secs_f32()).ceil() as usize
}

/// Fires one bolt along +X.
fn fire_once(app: &mut App) {
    set_aim_intent(app, Vec2::X);
    set_fire_intent(app, true);
    step(app, 1);
    set_fire_intent(app, false);
}

fn set_player_resistance(app: &mut App, damage_type: DamageType, resistance: f32) {
    let player = player(app);
    app.world_mut()
        .get_mut::<Resistances>(player)
        .unwrap()
        .0
        .insert(damage_type, resistance);
}

#[test]
fn a_bolt_damages_flashes_and_knocks_back_the_enemy_it_hits() {
    let mut app = app_in_run(1);
    let laspistol = weapon_data(&app, "laspistol");
    let juice = tuning(&app).juice;
    let rusher = spawn_enemy(&mut app, RUSHER, Vec2::new(80.0, 0.0));
    let mut damaged = MessageLog::<EnemyDamaged>::open(&app);
    let mut taken = MessageLog::<DamageTaken>::open(&app);

    fire_once(&mut app);
    let mut waited = 0;
    while projectiles(&mut app).len() == 1 {
        step(&mut app, 1);
        waited += 1;
        assert!(waited < frames(1.0), "the bolt never hit");
    }

    assert_eq!(only_enemy(&mut app).hp, 30.0 - laspistol.damage);
    let damage = damaged.read(&app);
    assert_eq!(damage.len(), 1);
    assert_eq!(damage[0].enemy, rusher);
    assert_eq!(damage[0].amount, laspistol.damage);
    assert!(
        taken.read(&app).is_empty(),
        "hitting an enemy is not the player taking damage"
    );

    assert!(app.world().get::<HitFlash>(rusher).unwrap().is_active());
    // Knocked back along the shot, by the tuned knockback (minus a frame of decay at most).
    let knockback = app.world().get::<Knockback>(rusher).unwrap().0;
    assert!(knockback.x > 0.0, "pushed away from the shooter");
    assert!(
        knockback.length() <= juice.knockback + 1e-3 && knockback.length() >= juice.knockback * 0.8,
        "knockback {knockback}"
    );
}

/// On a clamped long frame a bolt moves further than an enemy is wide, so
/// only a swept test can catch it wherever the enemy stands on its path.
#[test]
fn bolts_never_pass_through_an_enemy_on_long_frames() {
    let app = app_in_run(1);
    let t = tuning(&app);
    let laspistol = weapon_data(&app, "laspistol");
    let per_frame = laspistol.speed * t.max_frame_delta;
    assert!(per_frame > 2.0 * (6.0 + laspistol.projectile_radius));

    for x in 40..80 {
        let mut app = app_in_run(1);
        spawn_enemy(&mut app, RUSHER, Vec2::new(x as f32, 0.0));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
        fire_once(&mut app);
        step(&mut app, 8);
        assert_eq!(
            only_enemy(&mut app).hp,
            30.0 - laspistol.damage,
            "a rusher at x = {x} was not hit"
        );
    }
}

#[test]
fn killing_an_enemy_removes_it_and_announces_the_kill_with_the_weapon() {
    let mut app = app_in_run(1);
    let rusher = spawn_enemy(&mut app, RUSHER, Vec2::new(150.0, 0.0));
    let mut kills = MessageLog::<EnemyKilled>::open(&app);

    set_aim_intent(&mut app, Vec2::X);
    set_fire_intent(&mut app, true);
    let mut waited = 0;
    while !enemies(&mut app).is_empty() {
        step(&mut app, 1);
        waited += 1;
        assert!(waited < frames(3.0), "the rusher never died");
    }

    let kills = kills.read(&app);
    assert_eq!(kills.len(), 1);
    assert_eq!(kills[0].enemy, rusher);
    assert_eq!(kills[0].id, RUSHER);
    assert_eq!(kills[0].weapon.as_deref(), Some("laspistol"));
}

#[test]
fn player_damage_is_announced_after_mitigation() {
    let mut app = app_in_run(1);
    let max_hp = tuning(&app).player.max_hp;
    set_player_resistance(&mut app, DamageType::Physical, 0.5);
    let player = player(&mut app);
    let mut taken = MessageLog::<DamageTaken>::open(&app);

    request_damage(&mut app, player, 10.0, DamageType::Physical);
    step(&mut app, 1);

    assert_eq!(player_health(&mut app).current(), max_hp - 5.0);
    let damage = taken.read(&app);
    assert_eq!(damage.len(), 1);
    assert_eq!(damage[0].target, player);
    assert_eq!(damage[0].amount, 5.0);
    assert_eq!(damage[0].damage_type, DamageType::Physical);
}

#[test]
fn resistance_only_mitigates_its_own_damage_type() {
    let mut app = app_in_run(1);
    let max_hp = tuning(&app).player.max_hp;
    set_player_resistance(&mut app, DamageType::Physical, 0.5);
    let player = player(&mut app);

    request_damage(&mut app, player, 10.0, DamageType::Energy);
    step(&mut app, 1);
    assert_eq!(player_health(&mut app).current(), max_hp - 10.0);
}

#[test]
fn resistance_is_capped_at_the_tuned_maximum() {
    let mut app = app_in_run(1);
    let t = tuning(&app);
    assert_eq!(t.combat.max_resistance, 0.75);
    set_player_resistance(&mut app, DamageType::Energy, 1.5);
    let player = player(&mut app);

    request_damage(&mut app, player, 20.0, DamageType::Energy);
    step(&mut app, 1);
    // 75% of 20 is ignored, never all of it.
    assert_eq!(player_health(&mut app).current(), t.player.max_hp - 5.0);
}

#[test]
fn the_player_is_invulnerable_for_a_while_after_a_hit() {
    let mut app = app_in_run(1);
    let t = tuning(&app);
    let player = player(&mut app);
    let hurt = |app: &mut App| {
        request_damage(app, player, 10.0, DamageType::Physical);
        step(app, 1);
    };

    hurt(&mut app);
    assert_eq!(player_health(&mut app).current(), t.player.max_hp - 10.0);
    assert!(
        app.world()
            .get::<HurtInvulnerability>(player)
            .unwrap()
            .is_active()
    );

    hurt(&mut app);
    step(&mut app, frames(t.player.hurt_invulnerability) - 3);
    hurt(&mut app);
    assert_eq!(
        player_health(&mut app).current(),
        t.player.max_hp - 10.0,
        "hurt during the post-hit invulnerability"
    );

    // Freeze frames hold gameplay time still, so allow for the hurt freeze.
    step(&mut app, frames(t.juice.freeze_on_hurt) + 3);
    hurt(&mut app);
    assert_eq!(player_health(&mut app).current(), t.player.max_hp - 20.0);
}

#[test]
fn dodge_roll_invincibility_ignores_damage() {
    let mut app = app_in_run(1);
    let max_hp = tuning(&app).player.max_hp;
    let player = player(&mut app);
    set_aim_intent(&mut app, Vec2::X);
    request_dodge_roll(&mut app);
    step(&mut app, 1);

    request_damage(&mut app, player, 10.0, DamageType::Physical);
    step(&mut app, 1);
    assert_eq!(player_health(&mut app).current(), max_hp);
}

#[test]
fn a_new_run_restores_full_hp() {
    let mut app = app_in_run(1);
    let max_hp = tuning(&app).player.max_hp;
    let player = player(&mut app);
    request_damage(&mut app, player, 30.0, DamageType::Physical);
    step(&mut app, 1);
    assert!(player_health(&mut app).current() < max_hp);

    start_run(&mut app, Some(2));
    assert_eq!(player_health(&mut app).current(), max_hp);
}
