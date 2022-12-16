use bevy::{
    prelude::{
        AssetServer, Assets, Commands, Handle, Image, Plugin, Res, ResMut, Resource, StartupStage,
        Vec2,
    },
    sprite::TextureAtlas,
};

pub const TILE_SIZE: f32 = 32.;

#[derive(Resource)]
pub struct Graphics {
    pub characters_atlas: Handle<TextureAtlas>,
    pub tiles_atlas: Handle<TextureAtlas>,
}

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_sprites);
    }
}

fn load_sprites(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let characters = assets.load::<Image, _>("sprites/chars.png");
    let characters_atlas = texture_atlases.add(TextureAtlas::from_grid(
        characters,
        Vec2::splat(TILE_SIZE),
        4,
        2,
        None,
        None,
    ));

    let tiles_atlas = TextureAtlas::from_grid(
        assets.load::<Image, _>("sprites/tiles.png"),
        Vec2::splat(TILE_SIZE),
        8,
        7,
        None,
        None,
    );

    let tiles_atlas = texture_atlases.add(tiles_atlas);

    commands.insert_resource(Graphics {
        characters_atlas,
        tiles_atlas,
    });
}
