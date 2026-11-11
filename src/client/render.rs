//! Placeholder drawing following the three-quarter view rules (spec v3 §7):
//! sprites anchored at the feet, draw order sorted by Y, the top wall
//! showing its front face and the other walls only their top edge.
//! Weapons and bolts sit at chest height, bolts over a ground shadow.

use crate::{
    combat::{HitFlash, HurtInvulnerability, Projectile},
    data::Tuning,
    enemies::Enemy,
    kinematics::{Position, Velocity},
    player::{AimIntent, Player},
    schedule::GameSystemSet,
    world::room::Room,
};
use bevy::{prelude::*, sprite::Anchor};

const FLOOR: Color = Color::srgb(0.106, 0.110, 0.129);
const FLOOR_GRID: Color = Color::srgb(0.129, 0.137, 0.161);
const WALL_EDGE: Color = Color::srgb(0.227, 0.239, 0.271);
const WALL_FACE: Color = Color::srgb(0.302, 0.322, 0.357);
const MORTAL: Color = Color::srgb(0.624, 0.827, 0.561);
const RUSHER: Color = Color::srgb(0.753, 0.224, 0.169);
const FLASH: Color = Color::WHITE;
const WEAPON: Color = Color::srgb(0.420, 0.447, 0.494);
const LAS_BOLT: Color = Color::srgb(1.0, 0.478, 0.361);
const SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);

const FLOOR_GRID_STEP: f32 = 16.0;
/// Side and bottom walls show only this much of their top edge.
const WALL_TOP_EDGE: f32 = 3.0;
const MORTAL_SIZE: Vec2 = Vec2::new(10.0, 16.0);
const RUSHER_SIZE: Vec2 = Vec2::new(12.0, 14.0);
/// Times per second the player blinks while invulnerable after a hit.
const HURT_BLINK_RATE: f32 = 20.0;
/// Height above the feet where weapons are held and bolts fly.
const CHEST_HEIGHT: f32 = 9.0;
const WEAPON_SIZE: Vec2 = Vec2::new(8.0, 2.0);
const LAS_BOLT_SIZE: Vec2 = Vec2::new(6.0, 2.0);
const SHADOW_SIZE: Vec2 = Vec2::new(4.0, 1.0);
/// In front of or behind its holder.
const WEAPON_DEPTH: f32 = 0.0001;

const Z_FLOOR: f32 = 0.0;
const Z_FLOOR_GRID: f32 = 0.1;
const Z_WALLS: f32 = 1.0;
/// Actors are drawn between these depths, lower on screen in front.
const Z_ACTORS: f32 = 10.0;
const Y_SORT_SCALE: f32 = 0.001;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            draw_new_rooms,
            draw_new_players,
            draw_new_enemies,
            draw_new_projectiles,
            sync_positions,
            point_weapons,
            flash_when_hit,
            blink_while_invulnerable,
        )
            .chain()
            .in_set(GameSystemSet::RenderSync),
    );
}

fn draw_new_rooms(
    rooms: Query<(Entity, &Room), Added<Room>>,
    tuning: Res<Tuning>,
    mut commands: Commands,
) {
    let wall = tuning.room.wall_thickness;
    for (entity, room) in &rooms {
        let interior = room.interior;
        let (min, max, size) = (interior.min, interior.max, interior.size());
        let rect = |color: Color, centre: Vec2, size: Vec2, z: f32| {
            (
                Sprite::from_color(color, size),
                Transform::from_translation(centre.extend(z)),
            )
        };

        let mut parts = vec![
            rect(FLOOR, interior.center(), size, Z_FLOOR),
            // Top wall: its tall front face, capped by its top edge.
            rect(
                WALL_FACE,
                Vec2::new(interior.center().x, max.y + wall / 2.0),
                Vec2::new(size.x + 2.0 * wall, wall),
                Z_WALLS,
            ),
            rect(
                WALL_EDGE,
                Vec2::new(interior.center().x, max.y + wall - 1.0),
                Vec2::new(size.x + 2.0 * wall, 2.0),
                Z_WALLS + 0.1,
            ),
            // Side and bottom walls: only their thin top edge, against the floor.
            rect(
                WALL_EDGE,
                Vec2::new(min.x - WALL_TOP_EDGE / 2.0, interior.center().y),
                Vec2::new(WALL_TOP_EDGE, size.y),
                Z_WALLS,
            ),
            rect(
                WALL_EDGE,
                Vec2::new(max.x + WALL_TOP_EDGE / 2.0, interior.center().y),
                Vec2::new(WALL_TOP_EDGE, size.y),
                Z_WALLS,
            ),
            rect(
                WALL_EDGE,
                Vec2::new(interior.center().x, min.y - WALL_TOP_EDGE / 2.0),
                Vec2::new(size.x + 2.0 * WALL_TOP_EDGE, WALL_TOP_EDGE),
                Z_WALLS,
            ),
        ];
        let mut x = min.x + FLOOR_GRID_STEP;
        while x < max.x {
            parts.push(rect(
                FLOOR_GRID,
                Vec2::new(x, interior.center().y),
                Vec2::new(1.0, size.y),
                Z_FLOOR_GRID,
            ));
            x += FLOOR_GRID_STEP;
        }
        let mut y = min.y + FLOOR_GRID_STEP;
        while y < max.y {
            parts.push(rect(
                FLOOR_GRID,
                Vec2::new(interior.center().x, y),
                Vec2::new(size.x, 1.0),
                Z_FLOOR_GRID,
            ));
            y += FLOOR_GRID_STEP;
        }

        commands
            .entity(entity)
            .insert((Transform::default(), Visibility::default()))
            .with_children(|room| {
                for part in parts {
                    room.spawn(part);
                }
            });
    }
}

/// The colour a sprite returns to when it stops flashing.
#[derive(Component)]
struct BaseColor(Color);

/// Points at the aim, pivoting at the chest.
#[derive(Component)]
struct WeaponSprite;

fn draw_new_players(players: Query<Entity, Added<Player>>, mut commands: Commands) {
    for entity in &players {
        commands
            .entity(entity)
            .insert((
                Sprite::from_color(MORTAL, MORTAL_SIZE),
                BaseColor(MORTAL),
                Anchor::BOTTOM_CENTER,
                Transform::default(),
            ))
            .with_child((
                WeaponSprite,
                Sprite::from_color(WEAPON, WEAPON_SIZE),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(0.0, CHEST_HEIGHT, WEAPON_DEPTH),
            ));
    }
}

fn draw_new_enemies(enemies: Query<Entity, Added<Enemy>>, mut commands: Commands) {
    for entity in &enemies {
        commands.entity(entity).insert((
            Sprite::from_color(RUSHER, RUSHER_SIZE),
            BaseColor(RUSHER),
            Anchor::BOTTOM_CENTER,
            Transform::default(),
        ));
    }
}

fn flash_when_hit(mut actors: Query<(&HitFlash, &BaseColor, &mut Sprite)>) {
    for (flash, base, mut sprite) in &mut actors {
        sprite.color = if flash.is_active() { FLASH } else { base.0 };
    }
}

fn blink_while_invulnerable(mut players: Query<(&HurtInvulnerability, &mut Visibility)>) {
    for (invulnerability, mut visibility) in &mut players {
        let hidden = invulnerability.is_active()
            && ((invulnerability.0 * HURT_BLINK_RATE) as u32).is_multiple_of(2);
        *visibility = if hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

/// A shadow on the ground, the bolt itself at chest height.
fn draw_new_projectiles(
    projectiles: Query<(Entity, &Velocity), Added<Projectile>>,
    mut commands: Commands,
) {
    for (entity, velocity) in &projectiles {
        commands
            .entity(entity)
            .insert((
                Sprite::from_color(SHADOW, SHADOW_SIZE),
                Transform::default(),
            ))
            .with_child((
                Sprite::from_color(LAS_BOLT, LAS_BOLT_SIZE),
                Transform::from_xyz(0.0, CHEST_HEIGHT, 0.0)
                    .with_rotation(Quat::from_rotation_z(velocity.0.to_angle())),
            ));
    }
}

/// Held in front of the body, except when aiming up the screen.
fn point_weapons(
    players: Query<&AimIntent, With<Player>>,
    mut weapons: Query<(&ChildOf, &mut Transform), With<WeaponSprite>>,
) {
    for (holder, mut transform) in &mut weapons {
        let Ok(aim) = players.get(holder.parent()) else {
            continue;
        };
        if aim.direction == Vec2::ZERO {
            continue;
        }
        transform.rotation = Quat::from_rotation_z(aim.direction.to_angle());
        transform.translation.z = if aim.direction.y > 0.0 {
            -WEAPON_DEPTH
        } else {
            WEAPON_DEPTH
        };
    }
}

/// Feet on whole pixels; lower on screen is drawn in front.
fn sync_positions(mut actors: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in &mut actors {
        let feet = position.0.round();
        transform.translation = feet.extend(Z_ACTORS - feet.y * Y_SORT_SCALE);
    }
}
