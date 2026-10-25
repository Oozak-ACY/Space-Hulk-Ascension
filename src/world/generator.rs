//! Floor generation: a pure function from a seed and the Floor tuning to a
//! layout of single-screen Rooms on an Isaac-style grid (ADR 0001).
//!
//! The Floor grows outward from the Start Room. A new Room may touch only the
//! one Room it grows from, so layouts branch instead of clumping and every
//! door joins exactly two Rooms. If growth stalls before the target Room
//! count, generation retries from the same random stream, so a seed always
//! yields the same Floor.

use std::collections::VecDeque;

use crate::data::FloorTuning;

/// Generation attempts before giving up on unreachable tuning. Bundled tuning
/// succeeds within a handful; this only guards against impossible values.
const MAX_ATTEMPTS: u32 = 10_000;

/// A cell of the Floor grid, `(x, y)` with `y` growing southward.
pub type GridPos = (i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const ALL: [Direction; 4] = [Self::North, Self::East, Self::South, Self::West];

    /// The grid step toward the neighbouring cell.
    pub fn offset(self) -> GridPos {
        match self {
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomKind {
    Start,
    Combat,
    Encounter,
    Boss,
}

/// Index of a Room in its [`Floor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RoomId(pub usize);

/// One Room's place on the Floor: its grid cell, kind and doors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomCell {
    pos: GridPos,
    kind: RoomKind,
    doors: [Option<RoomId>; 4],
}

impl RoomCell {
    pub fn pos(&self) -> GridPos {
        self.pos
    }

    pub fn kind(&self) -> RoomKind {
        self.kind
    }

    /// The Room through the door in `dir`, if there is one.
    pub fn neighbour(&self, dir: Direction) -> Option<RoomId> {
        self.doors[dir.index()]
    }

    pub fn neighbours(&self) -> impl Iterator<Item = RoomId> + '_ {
        self.doors.iter().flatten().copied()
    }
}

/// A generated Floor layout. The Start Room is always [`Floor::start`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Floor {
    rooms: Vec<RoomCell>,
}

impl Floor {
    pub fn rooms(&self) -> &[RoomCell] {
        &self.rooms
    }

    pub fn ids(&self) -> impl Iterator<Item = RoomId> + use<> {
        (0..self.rooms.len()).map(RoomId)
    }

    pub fn start(&self) -> RoomId {
        RoomId(0)
    }

    pub fn room(&self, id: RoomId) -> &RoomCell {
        &self.rooms[id.0]
    }

    pub fn room_at(&self, pos: GridPos) -> Option<RoomId> {
        self.rooms
            .iter()
            .position(|room| room.pos == pos)
            .map(RoomId)
    }
}

/// Generates the Floor for `seed`. The same seed and tuning always give the
/// same Floor.
///
/// # Panics
///
/// If the tuning cannot hold a Start, Boss and Encounter Room, or its target
/// Room count is unreachable on its grid.
pub fn generate_floor(seed: u64, tuning: &FloorTuning) -> Floor {
    assert!(
        3 <= tuning.min_rooms && tuning.min_rooms <= tuning.max_rooms,
        "Floor tuning needs 3 <= min_rooms <= max_rooms, got {}..={}",
        tuning.min_rooms,
        tuning.max_rooms,
    );
    let mut rng = fastrand::Rng::with_seed(seed);
    // Rolled once: re-rolling on retry would favour the easier, smaller targets.
    let target = tuning.min_rooms + below(&mut rng, tuning.max_rooms - tuning.min_rooms + 1);
    for _ in 0..MAX_ATTEMPTS {
        if let Some(cells) = grow(&mut rng, tuning, target) {
            return assign_kinds(&mut rng, cells);
        }
    }
    panic!("Floor tuning {tuning:?} never reached its Room count in {MAX_ATTEMPTS} attempts");
}

/// Grows `target` cells breadth-first from the grid centre, or `None` if
/// growth stalls first.
fn grow(rng: &mut fastrand::Rng, tuning: &FloorTuning, target: usize) -> Option<Vec<GridPos>> {
    let in_grid = |(x, y): GridPos| {
        (0..tuning.grid_width).contains(&x) && (0..tuning.grid_height).contains(&y)
    };
    let start = (tuning.grid_width / 2, tuning.grid_height / 2);
    let mut cells = vec![start];
    let mut frontier = VecDeque::from([start]);

    while let Some(from) = frontier.pop_front() {
        let mut dirs = Direction::ALL;
        // Fisher–Yates on `below`, not `Rng::shuffle`, which is pointer-width dependent.
        for i in (1..dirs.len()).rev() {
            dirs.swap(i, below(rng, i + 1));
        }
        for dir in dirs {
            if cells.len() == target {
                return Some(cells);
            }
            let cell = step(from, dir);
            let touching = Direction::ALL
                .iter()
                .filter(|&&d| cells.contains(&step(cell, d)))
                .count();
            if !in_grid(cell) || cells.contains(&cell) || touching > 1 {
                continue;
            }
            if rng.f32() < tuning.expansion_chance {
                cells.push(cell);
                frontier.push_back(cell);
            }
        }
    }
    (cells.len() == target).then_some(cells)
}

/// Links doors between grid neighbours and picks the Boss (farthest dead end)
/// and the Encounter (another dead end when one exists).
fn assign_kinds(rng: &mut fastrand::Rng, cells: Vec<GridPos>) -> Floor {
    let rooms = cells
        .iter()
        .enumerate()
        .map(|(i, &pos)| RoomCell {
            pos,
            kind: if i == 0 {
                RoomKind::Start
            } else {
                RoomKind::Combat
            },
            doors: [None; 4],
        })
        .collect();
    let mut floor = Floor { rooms };
    for (i, &pos) in cells.iter().enumerate() {
        floor.rooms[i].doors = Direction::ALL.map(|d| floor.room_at(step(pos, d)));
    }

    let distance = distances_from_start(&floor);
    let dead_ends: Vec<RoomId> = floor
        .ids()
        .filter(|&id| id != floor.start() && floor.room(id).neighbours().count() == 1)
        .collect();
    // Growth is a tree of at least three Rooms, so it has a dead end besides the Start.
    // `max_by_key` keeps the last maximum; reversing makes ties go to the lowest RoomId.
    let boss = *dead_ends
        .iter()
        .rev()
        .max_by_key(|id| distance[id.0])
        .expect("a Floor of three or more Rooms has a dead end");
    floor.rooms[boss.0].kind = RoomKind::Boss;

    let spare_dead_ends: Vec<RoomId> = dead_ends.into_iter().filter(|&id| id != boss).collect();
    let candidates = if spare_dead_ends.is_empty() {
        floor
            .ids()
            .filter(|&id| floor.room(id).kind == RoomKind::Combat)
            .collect()
    } else {
        spare_dead_ends
    };
    let encounter = candidates[below(rng, candidates.len())];
    floor.rooms[encounter.0].kind = RoomKind::Encounter;
    floor
}

fn distances_from_start(floor: &Floor) -> Vec<usize> {
    let mut distance = vec![usize::MAX; floor.rooms.len()];
    distance[floor.start().0] = 0;
    let mut queue = VecDeque::from([floor.start()]);
    while let Some(id) = queue.pop_front() {
        for next in floor.room(id).neighbours() {
            if distance[next.0] == usize::MAX {
                distance[next.0] = distance[id.0] + 1;
                queue.push_back(next);
            }
        }
    }
    distance
}

/// A uniform index in `0..n`. Draws a fixed-width `u32` because fastrand's
/// `usize` draws differ between 64-bit native and 32-bit wasm builds, which
/// would give the same seed a different Floor in the browser.
fn below(rng: &mut fastrand::Rng, n: usize) -> usize {
    rng.u32(..n as u32) as usize
}

fn step((x, y): GridPos, dir: Direction) -> GridPos {
    let (dx, dy) = dir.offset();
    (x + dx, y + dy)
}
