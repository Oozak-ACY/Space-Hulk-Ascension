//! The HUD band and the pause overlay, pinned to the world camera.

use super::camera::{HUD_HEIGHT, VIEW_HEIGHT, VIEW_WIDTH, WorldCamera};
use crate::{
    combat::{Health, Weapon},
    data::{Tuning, Weapons},
    player::{DodgeRoll, Player},
    run::{AppState, Depth},
    schedule::GameSystemSet,
};
use bevy::{prelude::*, sprite::Anchor};

const HUD_BACKGROUND: Color = Color::srgb(0.059, 0.063, 0.075);
const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const TEXT: Color = Color::srgb(0.85, 0.85, 0.85);
const TEXT_SIZE: f32 = 16.0;
const HP_TRACK: Color = Color::srgb(0.157, 0.165, 0.188);
const HP: Color = Color::srgb(0.878, 0.314, 0.314);
const HP_GAUGE_SIZE: Vec2 = Vec2::new(64.0, 4.0);
const ROLL_TRACK: Color = Color::srgb(0.157, 0.165, 0.188);
const ROLL_CHARGING: Color = Color::srgb(0.420, 0.447, 0.494);
const ROLL_READY: Color = Color::srgb(0.624, 0.827, 0.561);
const ROLL_GAUGE_SIZE: Vec2 = Vec2::new(48.0, 4.0);
const HEAT_TRACK: Color = Color::srgb(0.157, 0.165, 0.188);
const HEAT: Color = Color::srgb(1.0, 0.604, 0.235);
const OVERHEAT: Color = Color::srgb(1.0, 0.231, 0.122);
const HEAT_GAUGE_SIZE: Vec2 = Vec2::new(64.0, 4.0);
const HUD_MARGIN: f32 = 8.0;
/// From the HP label's left edge to the gauge's.
const HP_GAUGE_OFFSET: f32 = 24.0;
/// From the HUD's left edge to the ROLL label's.
const ROLL_OFFSET: f32 = 100.0;
/// From the ROLL label's left edge to the gauge's.
const ROLL_GAUGE_OFFSET: f32 = 44.0;
/// From the HUD's left edge to the weapon name's.
const WEAPON_OFFSET: f32 = 212.0;
/// From the weapon name's left edge to the heat gauge's.
const HEAT_GAUGE_OFFSET: f32 = 96.0;
/// Between the heat gauge and the overheat label.
const OVERHEAT_LABEL_GAP: f32 = 8.0;
const Z_HUD: f32 = 900.0;

#[derive(Component)]
struct PauseOverlay;

#[derive(Component)]
struct GameOverOverlay;

#[derive(Component)]
struct GameOverText;

/// Empties as the player loses HP.
#[derive(Component)]
struct HpGaugeFill;

/// Fills as the Dodge Roll cooldown elapses.
#[derive(Component)]
struct RollGaugeFill;

#[derive(Component)]
struct WeaponName;

/// Fills as the weapon heats up; drains through the overheat lockout.
#[derive(Component)]
struct HeatGaugeFill;

/// Shown while the weapon is locked out.
#[derive(Component)]
struct OverheatLabel;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostStartup, spawn_hud).add_systems(
        Update,
        (
            show_pause_overlay,
            show_game_over,
            (fill_hp_gauge, fill_roll_gauge, show_weapon).in_set(GameSystemSet::RenderSync),
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
                        Text2d::new("HP"),
                        TextFont {
                            font_size: FontSize::Px(TEXT_SIZE),
                            ..default()
                        },
                        TextColor(TEXT),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(left, 0.0, 1.0),
                    ));
                    let hp = left + HP_GAUGE_OFFSET;
                    band.spawn((
                        Sprite::from_color(HP_TRACK, HP_GAUGE_SIZE),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(hp, 0.0, 1.0),
                    ));
                    band.spawn((
                        HpGaugeFill,
                        Sprite::from_color(HP, HP_GAUGE_SIZE),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(hp, 0.0, 2.0),
                    ));

                    let roll = left + ROLL_OFFSET;
                    band.spawn((
                        Text2d::new("ROLL"),
                        TextFont {
                            font_size: FontSize::Px(TEXT_SIZE),
                            ..default()
                        },
                        TextColor(TEXT),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(roll, 0.0, 1.0),
                    ));
                    let gauge = roll + ROLL_GAUGE_OFFSET;
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

                    let weapon = left + WEAPON_OFFSET;
                    band.spawn((
                        WeaponName,
                        Text2d::default(),
                        TextFont {
                            font_size: FontSize::Px(TEXT_SIZE),
                            ..default()
                        },
                        TextColor(TEXT),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(weapon, 0.0, 1.0),
                    ));
                    let heat = weapon + HEAT_GAUGE_OFFSET;
                    band.spawn((
                        Sprite::from_color(HEAT_TRACK, HEAT_GAUGE_SIZE),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(heat, 0.0, 1.0),
                    ));
                    band.spawn((
                        HeatGaugeFill,
                        Sprite::from_color(HEAT, HEAT_GAUGE_SIZE),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(heat, 0.0, 2.0).with_scale(Vec3::new(0.0, 1.0, 1.0)),
                    ));
                    band.spawn((
                        OverheatLabel,
                        Text2d::new("OVERHEAT"),
                        TextFont {
                            font_size: FontSize::Px(TEXT_SIZE),
                            ..default()
                        },
                        TextColor(OVERHEAT),
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz(
                            heat + HEAT_GAUGE_SIZE.x + OVERHEAT_LABEL_GAP,
                            0.0,
                            1.0,
                        ),
                        Visibility::Hidden,
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
            camera
                .spawn((
                    GameOverOverlay,
                    Sprite::from_color(OVERLAY, below_hud),
                    Transform::from_xyz(0.0, -HUD_HEIGHT / 2.0, Z_HUD),
                    Visibility::Hidden,
                ))
                .with_child((
                    GameOverText,
                    Text2d::default(),
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

/// Death ends the Run: how deep it got, and how to start the next.
fn show_game_over(
    state: Res<State<AppState>>,
    depth: Option<Res<Depth>>,
    mut overlay: Single<&mut Visibility, With<GameOverOverlay>>,
    mut text: Single<&mut Text2d, With<GameOverText>>,
) {
    let over = *state.get() == AppState::GameOver;
    **overlay = if over {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if let (true, Some(depth)) = (over, depth) {
        let message = format!(
            "YOU DIED\nreached Floor {}\npress Enter for a new Run",
            depth.floor
        );
        if text.0 != message {
            text.0 = message;
        }
    }
}

fn fill_hp_gauge(
    player: Option<Single<&Health, With<Player>>>,
    mut fill: Single<&mut Transform, With<HpGaugeFill>>,
) {
    if let Some(health) = player {
        fill.scale.x = health.fraction();
    }
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

fn show_weapon(
    catalog: Res<Weapons>,
    player: Option<Single<&Weapon, With<Player>>>,
    mut name: Single<&mut Text2d, With<WeaponName>>,
    mut fill: Single<(&mut Transform, &mut Sprite), With<HeatGaugeFill>>,
    mut overheat_label: Single<&mut Visibility, With<OverheatLabel>>,
) {
    let Some(weapon) = player else {
        return;
    };
    let data = catalog.get(weapon.id()).expect("weapon id in weapons.ron");
    let label = data.name.to_uppercase();
    if name.0 != label {
        name.0 = label;
    }
    let (transform, sprite) = &mut *fill;
    transform.scale.x = weapon.heat_fraction(data);
    let overheated = weapon.is_overheated();
    sprite.color = if overheated { OVERHEAT } else { HEAT };
    **overheat_label = if overheated {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}
