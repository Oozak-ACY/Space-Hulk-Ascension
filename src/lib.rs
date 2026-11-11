//! Space Hulk Ascension: a 2D twin-stick roguelike.
//!
//! [`GameplayPlugin`] holds every game rule and never reads input or draws;
//! [`client::ClientPlugin`] turns keyboard/mouse into intents and draws the
//! world. Tests drive the gameplay plugin alone through [`headless::app`].

pub mod client;
pub mod combat;
pub mod data;
pub mod enemies;
pub mod headless;
pub mod juice;
pub mod kinematics;
pub mod pause;
pub mod player;
pub mod run;
pub mod schedule;
pub mod world;

use bevy::{prelude::*, state::app::StatesPlugin};

/// All game rules, ordered by [`schedule::GameSystemSet`].
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        let tuning = data::Tuning::bundled().expect("bundled tuning.ron is valid");
        let weapons = data::Weapons::bundled().expect("bundled weapons.ron is valid");
        let enemies = data::Enemies::bundled().expect("bundled enemies.ron is valid");
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }
        app.insert_resource(tuning)
            .insert_resource(weapons)
            .insert_resource(enemies)
            .add_plugins((
                schedule::plugin,
                pause::plugin,
                run::plugin,
                player::plugin,
                combat::plugin,
                enemies::plugin,
                juice::plugin,
            ));
    }
}
