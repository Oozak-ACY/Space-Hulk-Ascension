//! Keyboard → intents and requests. Key codes are physical positions, so
//! WASD on QWERTY is ZQSD on AZERTY.

use crate::{pause::PauseRequest, player::MoveIntent, schedule::GameSystemSet};
use bevy::{prelude::*, window::WindowFocused};

const UP: [KeyCode; 2] = [KeyCode::KeyW, KeyCode::ArrowUp];
const DOWN: [KeyCode; 2] = [KeyCode::KeyS, KeyCode::ArrowDown];
const LEFT: [KeyCode; 2] = [KeyCode::KeyA, KeyCode::ArrowLeft];
const RIGHT: [KeyCode; 2] = [KeyCode::KeyD, KeyCode::ArrowRight];
const PAUSE: [KeyCode; 2] = [KeyCode::KeyP, KeyCode::Escape];

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (write_move_intent, request_pause).in_set(GameSystemSet::InputAndIntent),
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
