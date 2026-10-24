//! A Room: one fixed screen of play.

use bevy::prelude::*;

/// The Room the player is in.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Room {
    /// The walkable floor inside the walls, in gameplay coordinates.
    pub interior: Rect,
}
