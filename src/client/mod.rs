//! Window, camera, input and drawing. Kept thin and verified by playing:
//! every game rule lives in [`crate::GameplayPlugin`].

mod camera;
mod hud;
mod input;
mod render;

use crate::run::StartRun;
use bevy::prelude::*;

/// Everything a player sees and touches; add alongside [`crate::GameplayPlugin`].
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space Hulk Ascension".into(),
                        resolution: camera::initial_window_resolution(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((camera::plugin, input::plugin, render::plugin, hud::plugin))
        .add_systems(Startup, start_first_run);
    }
}

/// The game opens straight into a new Run.
fn start_first_run(mut start_run: MessageWriter<StartRun>) {
    start_run.write(StartRun { seed: None });
}
