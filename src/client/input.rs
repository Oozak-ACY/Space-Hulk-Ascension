//! Keyboard and mouse → intents and requests. Key codes are physical
//! positions, so WASD on QWERTY is ZQSD on AZERTY.

use super::camera::{ScreenCamera, WorldCamera, cursor_to_world};
use crate::{
    kinematics::Position,
    pause::PauseRequest,
    player::{AimIntent, DodgeIntent, MoveIntent, Player},
    schedule::GameSystemSet,
};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowFocused},
};

const UP: [KeyCode; 2] = [KeyCode::KeyW, KeyCode::ArrowUp];
const DOWN: [KeyCode; 2] = [KeyCode::KeyS, KeyCode::ArrowDown];
const LEFT: [KeyCode; 2] = [KeyCode::KeyA, KeyCode::ArrowLeft];
const RIGHT: [KeyCode; 2] = [KeyCode::KeyD, KeyCode::ArrowRight];
const PAUSE: [KeyCode; 2] = [KeyCode::KeyP, KeyCode::Escape];
const DODGE_ROLL_KEY: KeyCode = KeyCode::Space;
const DODGE_ROLL_BUTTON: MouseButton = MouseButton::Right;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            write_move_intent,
            write_aim_intent,
            write_dodge_intent,
            request_pause,
        )
            .in_set(GameSystemSet::InputAndIntent),
    );
}

fn write_move_intent(keys: Res<ButtonInput<KeyCode>>, mut intents: Query<&mut MoveIntent>) {
    let axis = |negative, positive| {
        keys.any_pressed(positive) as i8 as f32 - keys.any_pressed(negative) as i8 as f32
    };
    let direction = Vec2::new(axis(LEFT, RIGHT), axis(DOWN, UP)).normalize_or_zero();
    for mut intent in &mut intents {
        intent.0 = direction;
    }
}

/// Aims at the cursor; keeps the last aim while the cursor is off the canvas.
fn write_aim_intent(
    window: Single<&Window, With<PrimaryWindow>>,
    screen_camera: Single<(&Camera, &GlobalTransform), With<ScreenCamera>>,
    world_camera: Single<&Transform, With<WorldCamera>>,
    mut players: Query<(&Position, &mut AimIntent), With<Player>>,
) {
    let Some(target) = window
        .cursor_position()
        .and_then(|cursor| cursor_to_world(cursor, *screen_camera, &world_camera))
    else {
        return;
    };
    for (position, mut aim) in &mut players {
        aim.direction = target - position.0;
    }
}

/// Only sets the intent: gameplay clears it once read.
fn write_dodge_intent(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut intents: Query<&mut DodgeIntent>,
) {
    if keys.just_pressed(DODGE_ROLL_KEY) || buttons.just_pressed(DODGE_ROLL_BUTTON) {
        for mut intent in &mut intents {
            intent.0 = true;
        }
    }
}

fn request_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: MessageReader<WindowFocused>,
    mut pause: MessageWriter<PauseRequest>,
) {
    if keys.any_just_pressed(PAUSE) {
        pause.write(PauseRequest::Toggle);
    }
    if focus.read().any(|change| !change.focused) {
        pause.write(PauseRequest::Pause);
    }
}
