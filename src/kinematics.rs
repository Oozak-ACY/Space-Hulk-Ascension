//! Where things are on the Room's flat top-down plane.
//!
//! Gameplay positions are in pixels with +Y pointing up the screen, the
//! current Room's interior centred on the origin. A position is the feet:
//! the collision circle's centre and the sprite's anchor.

use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Position(pub Vec2);

/// Pixels per second.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vec2);
