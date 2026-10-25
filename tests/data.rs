//! Every bundled data file parses and holds the spec v3 §9.1 values.

use space_hulk_ascension::data::Tuning;

#[test]
fn bundled_tuning_parses_and_holds_spec_values() {
    let tuning = Tuning::bundled().expect("tuning.ron must parse");

    let player = &tuning.player;
    assert_eq!(player.move_speed, 110.0);
    assert_eq!(player.snappiness, 25.0);
    assert_eq!(player.max_hp, 100.0);
    assert_eq!(player.hurt_invulnerability, 0.6);
    assert_eq!(player.radius, 5.0);
    let roll = &player.dodge_roll;
    assert_eq!(roll.speed, 260.0);
    assert_eq!(roll.duration, 0.25);
    assert_eq!(roll.invincibility, 0.30);
    assert_eq!(roll.cooldown, 0.8);

    assert_eq!(tuning.room.interior_width, 464.0);
    assert_eq!(tuning.room.interior_height, 224.0);
    assert_eq!(tuning.room.wall_thickness, 8.0);

    let floor = &tuning.floor;
    assert_eq!((floor.min_rooms, floor.max_rooms), (8, 12));
    assert_eq!((floor.grid_width, floor.grid_height), (9, 8));
    assert_eq!(floor.expansion_chance, 0.5);
    assert_eq!(floor.encounter_rooms, 1);

    assert_eq!(tuning.max_frame_delta, 0.05);
}
