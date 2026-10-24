//! Space Hulk Ascension: a 2D twin-stick roguelike.
//!
//! [`GameplayPlugin`] holds every game rule and never reads input or draws;
//! [`client::ClientPlugin`] turns keyboard/mouse into intents and draws the
//! world. Tests drive the gameplay plugin alone through [`headless::app`].

pub mod client;
pub mod data;
pub mod headless;
pub mod kinematics;
pub mod pause;
pub mod player;
pub mod run;
pub mod schedule;
pub mod world;

use bevy::prelude::*;

/// All game rules, ordered by [`schedule::GameSystemSet`].
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        let tuning = data::Tuning::bundled().expect("bundled tuning.ron is valid");
        app.insert_resource(tuning).add_plugins((
            schedule::plugin,
            pause::plugin,
            run::plugin,
            player::plugin,
        ));
    }
}
