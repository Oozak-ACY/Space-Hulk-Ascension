//! Pausing freezes the gameplay clock; long frames are clamped so a hitch
//! never breaks physics.

use crate::{data::Tuning, schedule::GameSystemSet};
use bevy::prelude::*;
use std::time::Duration;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseRequest {
    /// P / Esc.
    Toggle,
    /// Losing window or tab focus.
    Pause,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<PauseRequest>()
        .add_systems(Startup, clamp_frame_delta)
        .add_systems(
            Update,
            apply_pause_requests.in_set(GameSystemSet::InputAndIntent),
        );
}

fn clamp_frame_delta(tuning: Res<Tuning>, mut time: ResMut<Time<Virtual>>) {
    time.set_max_delta(Duration::from_secs_f32(tuning.max_frame_delta));
}

fn apply_pause_requests(
    mut requests: MessageReader<PauseRequest>,
    mut time: ResMut<Time<Virtual>>,
) {
    for request in requests.read() {
        match request {
            PauseRequest::Toggle if time.is_paused() => time.unpause(),
            PauseRequest::Toggle | PauseRequest::Pause => time.pause(),
        }
    }
}
