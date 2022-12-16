use bevy::prelude::*;
use graphics::GraphicsPlugin;
use hello::HelloPlugin;
use map::MapPlugin;
use player::PlayerPlugin;

mod graphics;
mod hello;
mod map;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GraphicsPlugin)
        .add_plugin(HelloPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
