//! The HUD band and the pause overlay, pinned to the world camera.

use super::camera::{HUD_HEIGHT, VIEW_HEIGHT, VIEW_WIDTH, WorldCamera};
use crate::{
    data::Tuning,
    player::{DodgeRoll, Player},
    schedule::GameSystemSet,
};
use bevy::{prelude::*, sprite::Anchor};

const HUD_BACKGROUND: Color = Color::srgb(0.059, 0.063, 0.075);
const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const TEXT: Color = Color::srgb(0.85, 0.85, 0.85);
const TEXT_SIZE: f32 = 16.0;
const ROLL_TRACK: Color = Color::srgb(0.157, 0.165, 0.188);
const ROLL_CHARGING: Color = Color::srgb(0.420, 0.447, 0.494);
const ROLL_READY: Color = Color::srgb(0.624, 0.827, 0.561);
const ROLL_GAUGE_SIZE: Vec2 = Vec2::new(48.0, 4.0);
const HUD_MARGIN: f32 = 8.0;
/// From the label's left edge to the gauge's.
const ROLL_GAUGE_OFFSET: f32 = 44.0;
const Z_HUD: f32 = 900.0;

#[derive(Component)]
struct PauseOverlay;

/// Fills as the Dodge Roll cooldown elapses.
#[derive(Component)]
struct RollGaugeFill;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostStartup, spawn_hud).add_systems(
        Update,
        (
            show_pause_overlay,
            fill_roll_gauge.in_set(GameSystemSet::RenderSync),
        ),
    );
}

fn spawn_hud(camera: Single<Entity, With<WorldCamera>>, mut commands: Commands) {
    let view = Vec2::new(VIEW_WIDTH as f32, VIEW_HEIGHT as f32);
    let below_hud = Vec2::new(view.x, view.y - HUD_HEIGHT);
    commands
        .entity(*camera)
        .insert(Visibility::default())
        .with_children(|camera| {
            camera
                .spawn((
                    Sprite::from_color(HUD_BACKGROUND, Vec2::new(view.x, HUD_HEIGHT)),
                    Transform::from_xyz(0.0, (view.y - HUD_HEIGHT) / 2.0, Z_HUD),
                ))
                .with_children(|band| {
                    let left = -view.x / 2.0 + HUD_MARGIN;
                    band.spawn((
                        Text2d::new("ROLL"),
                        TextFont {
                            font_size: FontSize::Px(TEXT_SIZE),
                            ..default()
                        },
                        TextColor(TEXT),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(left, 0.0, 1.0),
                    ));
                    let gauge = left + ROLL_GAUGE_OFFSET;
                    band.spawn((
                        Sprite::from_color(ROLL_TRACK, ROLL_GAUGE_SIZE),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(gauge, 0.0, 1.0),
                    ));
                    band.spawn((
                        RollGaugeFill,
                        Sprite::from_color(ROLL_READY, ROLL_GAUGE_SIZE),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(gauge, 0.0, 2.0),
                    ));
                });
            camera
                .spawn((
                    PauseOverlay,
                    Sprite::from_color(OVERLAY, below_hud),
                    Transform::from_xyz(0.0, -HUD_HEIGHT / 2.0, Z_HUD),
                    Visibility::Hidden,
                ))
                .with_child((
                    Text2d::new("PAUSED\npress P or Esc"),
                    TextFont {
                        font_size: FontSize::Px(TEXT_SIZE),
                        ..default()
                    },
                    TextColor(TEXT),
                    TextLayout::justify(Justify::Center),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                ));
        });
}

fn show_pause_overlay(
    time: Res<Time<Virtual>>,
    mut overlay: Single<&mut Visibility, With<PauseOverlay>>,
) {
    **overlay = if time.is_paused() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn fill_roll_gauge(
    tuning: Res<Tuning>,
    player: Option<Single<&DodgeRoll, With<Player>>>,
    mut fill: Single<(&mut Transform, &mut Sprite), With<RollGaugeFill>>,
) {
    let Some(roll) = player else {
        return;
    };
    let tuning = &tuning.player.dodge_roll;
    let (transform, sprite) = &mut *fill;
    transform.scale.x = roll.cooldown_progress(tuning);
    sprite.color = if roll.is_ready(tuning) {
        ROLL_READY
    } else {
        ROLL_CHARGING
    };
}
