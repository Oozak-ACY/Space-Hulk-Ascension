use std::collections::{HashSet, VecDeque};

use space_hulk_ascension::data::{FloorTuning, Tuning};
use space_hulk_ascension::world::generator::{Direction, Floor, RoomId, RoomKind, generate_floor};

const SEEDS: std::ops::Range<u64> = 0..500;

fn bundled() -> FloorTuning {
    Tuning::bundled().expect("tuning.ron must parse").floor
}

/// Path distance from the Start Room to every Room, by walking doors.
fn distances_from_start(floor: &Floor) -> Vec<Option<usize>> {
    let mut dist = vec![None; floor.rooms().len()];
    let mut queue = VecDeque::from([floor.start()]);
    dist[floor.start().0] = Some(0);
    while let Some(id) = queue.pop_front() {
        let d = dist[id.0].unwrap();
        for n in floor.room(id).neighbours() {
            if dist[n.0].is_none() {
                dist[n.0] = Some(d + 1);
                queue.push_back(n);
            }
        }
    }
    dist
}

fn rooms_of_kind(floor: &Floor, kind: RoomKind) -> Vec<RoomId> {
    floor
        .ids()
        .filter(|&id| floor.room(id).kind() == kind)
        .collect()
}

fn is_dead_end(floor: &Floor, id: RoomId) -> bool {
    floor.room(id).neighbours().count() == 1
}

#[test]
fn room_count_is_within_the_tuned_range() {
    let tuning = bundled();
    for seed in SEEDS {
        let n = generate_floor(seed, &tuning).rooms().len();
        assert!(
            (tuning.min_rooms..=tuning.max_rooms).contains(&n),
            "seed {seed}: {n} Rooms"
        );
    }
}

#[test]
fn every_room_is_reachable_from_the_start() {
    for seed in SEEDS {
        let floor = generate_floor(seed, &bundled());
        let dist = distances_from_start(&floor);
        assert!(
            dist.iter().all(Option::is_some),
            "seed {seed}: unreachable Room"
        );
    }
}

#[test]
fn exactly_one_start_boss_and_encounter_room() {
    for seed in SEEDS {
        let floor = generate_floor(seed, &bundled());
        assert_eq!(
            rooms_of_kind(&floor, RoomKind::Start),
            vec![floor.start()],
            "seed {seed}"
        );
        assert_eq!(
            rooms_of_kind(&floor, RoomKind::Boss).len(),
            1,
            "seed {seed}"
        );
        assert_eq!(
            rooms_of_kind(&floor, RoomKind::Encounter).len(),
            1,
            "seed {seed}"
        );
    }
}

#[test]
fn boss_room_is_the_dead_end_farthest_from_the_start() {
    for seed in SEEDS {
        let floor = generate_floor(seed, &bundled());
        let dist = distances_from_start(&floor);
        let boss = rooms_of_kind(&floor, RoomKind::Boss)[0];
        assert!(
            is_dead_end(&floor, boss),
            "seed {seed}: Boss is not a dead end"
        );
        let farthest = floor
            .ids()
            .filter(|&id| id != floor.start() && is_dead_end(&floor, id))
            .map(|id| dist[id.0].unwrap())
            .max()
            .unwrap();
        assert_eq!(dist[boss.0].unwrap(), farthest, "seed {seed}");
    }
}

#[test]
fn encounter_room_is_another_dead_end_when_one_exists() {
    for seed in SEEDS {
        let floor = generate_floor(seed, &bundled());
        let encounter = rooms_of_kind(&floor, RoomKind::Encounter)[0];
        let boss = rooms_of_kind(&floor, RoomKind::Boss)[0];
        assert_ne!(encounter, floor.start(), "seed {seed}");
        assert_ne!(encounter, boss, "seed {seed}");
        let spare_dead_end = floor
            .ids()
            .any(|id| id != floor.start() && id != boss && is_dead_end(&floor, id));
        if spare_dead_end {
            assert!(
                is_dead_end(&floor, encounter),
                "seed {seed}: Encounter is not a dead end"
            );
        }
    }
}

#[test]
fn placement_avoids_clumps() {
    // Each new Room touches exactly one existing Room, so grid adjacencies
    // form a tree: one fewer adjacency than Rooms, and never a 2x2 block.
    for seed in SEEDS {
        let floor = generate_floor(seed, &bundled());
        let cells: HashSet<_> = floor.ids().map(|id| floor.room(id).pos()).collect();
        let adjacencies = cells
            .iter()
            .filter(|p| cells.contains(&(p.0 + 1, p.1)))
            .count()
            + cells
                .iter()
                .filter(|p| cells.contains(&(p.0, p.1 + 1)))
                .count();
        assert_eq!(adjacencies, cells.len() - 1, "seed {seed}");
    }
}

#[test]
fn doors_match_grid_neighbours_both_ways() {
    for seed in SEEDS {
        let floor = generate_floor(seed, &bundled());
        for id in floor.ids() {
            let (x, y) = floor.room(id).pos();
            for dir in Direction::ALL {
                let (dx, dy) = dir.offset();
                let expected = floor.room_at((x + dx, y + dy));
                assert_eq!(floor.room(id).neighbour(dir), expected, "seed {seed}");
                if let Some(n) = expected {
                    assert_eq!(
                        floor.room(n).neighbour(dir.opposite()),
                        Some(id),
                        "seed {seed}"
                    );
                }
            }
        }
    }
}

#[test]
fn same_seed_always_yields_an_identical_floor() {
    for seed in SEEDS {
        assert_eq!(
            generate_floor(seed, &bundled()),
            generate_floor(seed, &bundled())
        );
    }
}

#[test]
fn different_seeds_yield_varied_layouts() {
    let layouts: HashSet<Vec<(i32, i32)>> = SEEDS
        .map(|seed| {
            let floor = generate_floor(seed, &bundled());
            floor.ids().map(|id| floor.room(id).pos()).collect()
        })
        .collect();
    assert!(
        layouts.len() > SEEDS.count() * 9 / 10,
        "only {} distinct layouts",
        layouts.len()
    );
}

#[test]
fn room_counts_and_grid_size_come_from_tuning() {
    let tuning = FloorTuning {
        min_rooms: 5,
        max_rooms: 5,
        grid_width: 5,
        grid_height: 4,
        ..bundled()
    };
    for seed in SEEDS {
        let floor = generate_floor(seed, &tuning);
        assert_eq!(floor.rooms().len(), 5, "seed {seed}");
        for id in floor.ids() {
            let (x, y) = floor.room(id).pos();
            assert!(
                (0..5).contains(&x) && (0..4).contains(&y),
                "seed {seed}: ({x}, {y})"
            );
        }
    }
}

#[test]
fn generation_retries_until_it_reaches_a_demanding_target() {
    // Large targets fail often on a single attempt; retries must still land
    // exactly on target, deterministically.
    let tuning = FloorTuning {
        min_rooms: 18,
        max_rooms: 18,
        ..bundled()
    };
    for seed in 0..100 {
        let floor = generate_floor(seed, &tuning);
        assert_eq!(floor.rooms().len(), 18, "seed {seed}");
        assert_eq!(floor, generate_floor(seed, &tuning));
    }
}

#[test]
fn a_seed_yields_the_same_floor_on_every_platform() {
    // Pinned so any drift in the random stream (e.g. pointer-width dependent
    // draws, which would differ between native and wasm) fails here.
    use RoomKind::*;
    let floor = generate_floor(42, &bundled());
    let layout: Vec<_> = floor
        .ids()
        .map(|id| (floor.room(id).pos(), floor.room(id).kind()))
        .collect();
    assert_eq!(
        layout,
        [
            ((4, 4), Start),
            ((3, 4), Combat),
            ((4, 3), Combat),
            ((5, 3), Combat),
            ((6, 3), Combat),
            ((7, 3), Combat),
            ((6, 2), Combat),
            ((6, 1), Combat),
            ((6, 0), Boss),
            ((5, 1), Encounter),
        ]
    );
}

#[test]
fn encounter_falls_back_to_a_combat_room_without_a_spare_dead_end() {
    // Three Rooms in a line from the Start leave the Boss as the only dead end.
    let tuning = FloorTuning {
        min_rooms: 3,
        max_rooms: 3,
        ..bundled()
    };
    let mut fallbacks = 0;
    for seed in SEEDS {
        let floor = generate_floor(seed, &tuning);
        let boss = rooms_of_kind(&floor, RoomKind::Boss)[0];
        let encounter = rooms_of_kind(&floor, RoomKind::Encounter)[0];
        assert!(
            encounter != floor.start() && encounter != boss,
            "seed {seed}"
        );
        if !is_dead_end(&floor, encounter) {
            fallbacks += 1;
        }
    }
    assert!(fallbacks > 0, "no seed exercised the fallback");
}
