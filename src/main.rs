use bevy::prelude::*;
use space_hulk_ascension::{GameplayPlugin, client::ClientPlugin};

fn main() {
    App::new().add_plugins((ClientPlugin, GameplayPlugin)).run();
}
