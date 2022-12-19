use actions::ActionsPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use graphics::GraphicsPlugin;
use map::MapPlugin;
use player::PlayerPlugin;
use stages::StagesPlugin;

mod actions;
mod graphics;
mod map;
mod monsters;
mod player;
mod spawner;
mod stages;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(StagesPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(GraphicsPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(ActionsPlugin)
        .run();
}
