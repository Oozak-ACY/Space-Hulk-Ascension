//! The HUD band and the pause overlay, pinned to the world camera.

use super::camera::{HUD_HEIGHT, VIEW_HEIGHT, VIEW_WIDTH, WorldCamera};
use bevy::prelude::*;

const HUD_BACKGROUND: Color = Color::srgb(0.059, 0.063, 0.075);
const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const TEXT: Color = Color::srgb(0.85, 0.85, 0.85);
const TEXT_SIZE: f32 = 16.0;
const Z_HUD: f32 = 900.0;

#[derive(Component)]
struct PauseOverlay;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostStartup, spawn_hud)
        .add_systems(Update, show_pause_overlay);
}

fn spawn_hud(camera: Single<Entity, With<WorldCamera>>, mut commands: Commands) {
    let view = Vec2::new(VIEW_WIDTH as f32, VIEW_HEIGHT as f32);
    let below_hud = Vec2::new(view.x, view.y - HUD_HEIGHT);
    commands
        .entity(*camera)
        .insert(Visibility::default())
        .with_children(|camera| {
            camera.spawn((
                Sprite::from_color(HUD_BACKGROUND, Vec2::new(view.x, HUD_HEIGHT)),
                Transform::from_xyz(0.0, (view.y - HUD_HEIGHT) / 2.0, Z_HUD),
            ));
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
