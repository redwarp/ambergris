use actions::ActionsPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use graphics::GraphicsPlugin;
use map::MapPlugin;
use player::PlayerPlugin;

mod actions;
mod graphics;
mod map;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(GraphicsPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(ActionsPlugin)
        .run();
}
