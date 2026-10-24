//! The headless game app: gameplay with no window, no rendering and a
//! manually stepped clock. Each `app.update()` advances exactly [`FRAME`].

use crate::GameplayPlugin;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use std::time::Duration;

/// The duration of one stepped frame.
pub const FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// Builds the app and runs its first (zero-length) frame, so the next
/// `update()` advances the clock by [`FRAME`]. No Run is started.
pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameplayPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME));
    app.update();
    app
}
