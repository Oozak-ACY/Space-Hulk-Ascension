//! Cultist gunners: they hold their range, strafe, telegraph each shot and
//! fire bullets the Dodge Roll can pass through. Enemies never stack.

mod common;

use bevy::prelude::*;
use common::*;
use space_hulk_ascension::{
    combat::DamageTaken,
    data::{Enemies, EnemyBehaviour, GunnerData},
    headless::FRAME,
};

fn frames(seconds: f32) -> usize {
    (seconds / FRAME.as_secs_f32()).round() as usize
}

fn gunner_data() -> GunnerData {
    let EnemyBehaviour::Gunner(gunner) = Enemies::bundled().unwrap().get(GUNNER).unwrap().behaviour
    else {
        panic!("the gunner is a Gunner");
    };
    gunner
}

fn distance_to_player(app: &mut App) -> f32 {
    let player = player_position(app);
    only_enemy(app).position.distance(player)
}

/// Steps until a gunner's first bullet is in the air; returns it.
fn wait_for_bullet(app: &mut App) -> ProjectileState {
    let mut waited = 0;
    while projectiles(app).is_empty() {
        step(app, 1);
        waited += 1;
        assert!(waited < frames(3.0), "the gunner never fired");
    }
    only_projectile(app)
}

#[test]
fn a_far_gunner_approaches_at_its_speed() {
    let mut app = app_in_run(1);
    spawn_enemy(&mut app, GUNNER, Vec2::new(200.0, 0.0));
    let start = only_enemy(&mut app).position;

    step(&mut app, frames(0.5));
    let after = only_enemy(&mut app).position;
    assert!(after.x < start.x, "moved toward the player");
    let moved = start.distance(after);
    assert!(moved <= 50.0 * 0.5 + 1e-3, "{moved} px in 0.5 s");
    assert!(moved > 15.0, "only {moved} px in 0.5 s");
}

#[test]
fn a_close_gunner_backs_off() {
    let mut app = app_in_run(1);
    spawn_enemy(&mut app, GUNNER, Vec2::new(60.0, 0.0));
    let before = distance_to_player(&mut app);

    step(&mut app, frames(0.5));
    assert!(distance_to_player(&mut app) > before + 10.0);
}

#[test]
fn a_gunner_at_its_range_strafes_around_the_player() {
    let mut app = app_in_run(1);
    let gunner = gunner_data();
    spawn_enemy(&mut app, GUNNER, Vec2::new(gunner.range, 0.0));
    let start = only_enemy(&mut app).position;

    step(&mut app, frames(0.5));
    let after = only_enemy(&mut app).position;
    assert!(
        (after.y - start.y).abs() > 10.0,
        "moved sideways only {} px",
        (after.y - start.y).abs()
    );
    let distance = distance_to_player(&mut app);
    assert!(
        (distance - gunner.range).abs() <= gunner.range_tolerance,
        "drifted to {distance} px"
    );
}

#[test]
fn a_strafing_gunner_switches_direction_now_and_then() {
    let mut app = app_in_run(1);
    let range = gunner_data().range;
    let entity = spawn_enemy(&mut app, GUNNER, Vec2::new(range, 0.0));

    let mut sides = std::collections::HashSet::new();
    for _ in 0..frames(10.0) {
        step(&mut app, 1);
        sides.insert(gunner(&app, entity).strafe_side() > 0.0);
    }
    assert_eq!(sides.len(), 2, "always strafed the same way");
}

#[test]
fn a_gunner_telegraphs_then_fires_along_the_aim_locked_at_the_warning() {
    let mut app = app_in_run(1);
    let data = gunner_data();
    let entity = spawn_enemy(&mut app, GUNNER, Vec2::new(data.range, 0.0));

    // The warning starts one interval after it appears, aimed at the player…
    let mut waited = 1;
    let locked = loop {
        if let Some(aim) = gunner(&app, entity).aim_warning() {
            break aim;
        }
        step(&mut app, 1);
        waited += 1;
        assert!(waited < frames(3.0), "never aimed");
    };
    let seconds = waited as f32 * FRAME.as_secs_f32();
    assert!(
        (seconds - data.interval).abs() <= 2.0 * FRAME.as_secs_f32(),
        "aimed after {seconds} s"
    );
    let towards = (player_position(&mut app) - only_enemy(&mut app).position).normalize();
    assert!(
        locked.distance(towards) < 1e-3,
        "aimed {locked}, not {towards}"
    );

    // …and the player moving away does not move it.
    set_player_position(&mut app, Vec2::new(-150.0, 80.0));
    let mut warned = 0;
    while projectiles(&mut app).is_empty() {
        step(&mut app, 1);
        warned += 1;
        assert!(warned < frames(1.0), "never fired");
    }
    let warning = warned as f32 * FRAME.as_secs_f32();
    assert!(
        (warning - data.aim_warning).abs() <= 2.0 * FRAME.as_secs_f32(),
        "fired {warning} s after the warning"
    );
    let bullet = only_projectile(&mut app);
    assert!(
        bullet.velocity.normalize().distance(locked) < 1e-3,
        "fired toward {}, not the locked {locked}",
        bullet.velocity.normalize()
    );
    assert!((bullet.velocity.length() - data.bullet_speed).abs() < 1e-3);
    assert_eq!(bullet.projectile.damage, data.bullet_damage);
    assert!(gunner(&app, entity).aim_warning().is_none());

    // The next warning comes one interval after the first.
    let mut next = 0;
    while gunner(&app, entity).aim_warning().is_none() {
        step(&mut app, 1);
        next += 1;
        assert!(next < frames(3.0), "never aimed again");
    }
    let period = (warned + next) as f32 * FRAME.as_secs_f32();
    assert!(
        (period - data.interval).abs() <= 2.0 * FRAME.as_secs_f32(),
        "warnings {period} s apart"
    );
}

#[test]
fn an_enemy_bullet_hurts_the_player_and_disappears() {
    let mut app = app_in_run(1);
    let data = gunner_data();
    let max_hp = tuning(&app).player.max_hp;
    let entity = spawn_enemy(&mut app, GUNNER, Vec2::new(data.range, 0.0));
    let bullet = wait_for_bullet(&mut app);
    despawn(&mut app, entity);
    set_player_position(
        &mut app,
        bullet.position + bullet.velocity.normalize() * 30.0,
    );
    let mut taken = MessageLog::<DamageTaken>::open(&app);

    let mut waited = 0;
    while !projectiles(&mut app).is_empty() {
        step(&mut app, 1);
        waited += 1;
        assert!(waited < frames(1.0), "the bullet never hit");
    }
    assert_eq!(
        player_health(&mut app).current(),
        max_hp - data.bullet_damage
    );
    let damage = taken.read(&app);
    assert_eq!(damage.len(), 1);
    assert_eq!(damage[0].amount, data.bullet_damage);
}

#[test]
fn enemy_bullets_pass_through_a_dodge_rolling_player() {
    let mut app = app_in_run(1);
    let data = gunner_data();
    let max_hp = tuning(&app).player.max_hp;
    let entity = spawn_enemy(&mut app, GUNNER, Vec2::new(data.range, 0.0));
    let bullet = wait_for_bullet(&mut app);
    despawn(&mut app, entity);
    let along = bullet.velocity.normalize();
    set_player_position(&mut app, bullet.position + along * 40.0);

    // Roll straight into the bullet.
    set_aim_intent(&mut app, -along);
    request_dodge_roll(&mut app);
    step(&mut app, frames(0.4));

    assert_eq!(player_health(&mut app).current(), max_hp, "the bullet hurt");
    let bullet = only_projectile(&mut app);
    let player = player_position(&mut app);
    assert!(
        (bullet.position - player).dot(along) > 0.0,
        "the bullet is not past the player"
    );
}

#[test]
fn enemy_bullets_fly_through_other_enemies() {
    let mut app = app_in_run(1);
    let data = gunner_data();
    let entity = spawn_enemy(&mut app, GUNNER, Vec2::new(data.range, 0.0));
    let bullet = wait_for_bullet(&mut app);
    despawn(&mut app, entity);
    let along = bullet.velocity.normalize();
    set_player_position(&mut app, bullet.position + along * 100.0);
    let rusher = spawn_enemy(&mut app, RUSHER, bullet.position + along * 20.0);

    step(&mut app, frames(0.3));
    let hp = enemies(&mut app)
        .into_iter()
        .find(|enemy| enemy.entity == rusher)
        .unwrap()
        .hp;
    assert_eq!(hp, 30.0, "a gunner's bullet hurt another enemy");
}

#[test]
fn enemies_push_apart_instead_of_stacking() {
    let mut app = app_in_run(1);
    let catalog = Enemies::bundled().unwrap();
    let radius = |id: &str| catalog.get(id).unwrap().radius;
    set_player_position(&mut app, Vec2::new(-200.0, 0.0));
    // Two on the very same spot, a third overlapping them.
    for (id, at) in [
        (RUSHER, Vec2::new(100.0, 0.0)),
        (RUSHER, Vec2::new(100.0, 0.0)),
        (GUNNER, Vec2::new(102.0, 1.0)),
    ] {
        app.world_mut()
            .write_message(space_hulk_ascension::enemies::SpawnEnemy {
                id: id.to_owned(),
                position: at,
            });
    }

    for frame in 0..frames(1.5) {
        step(&mut app, 1);
        if frame < 3 {
            continue;
        }
        let all = enemies(&mut app);
        assert_eq!(all.len(), 3);
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                let apart = a.position.distance(b.position);
                let touching = radius(&a.id) + radius(&b.id);
                assert!(
                    apart >= touching - 0.5,
                    "frame {frame}: {} and {} overlap ({apart} px apart)",
                    a.id,
                    b.id
                );
            }
        }
    }
}
