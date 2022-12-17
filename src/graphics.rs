use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::{
        AssetServer, Assets, Camera, Camera2d, Camera2dBundle, ClearColor, Color, Commands,
        Component, Handle, Image, Plugin, Res, ResMut, Resource, StartupStage, Vec2,
    },
    render::view::RenderLayers,
    sprite::TextureAtlas,
};

pub const TILE_SIZE: f32 = 32.;

#[derive(Component)]
pub struct MapCamera;

#[derive(Component)]
pub struct UiCamera;

#[derive(Resource)]
pub struct Graphics {
    pub characters_atlas: Handle<TextureAtlas>,
    pub tiles_atlas: Handle<TextureAtlas>,
}

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
            .add_startup_system_to_stage(StartupStage::PreStartup, load_sprites)
            .add_startup_system(setup_cameras);
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

fn setup_cameras(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(MapCamera);

    // Second camera for UI elements. To add stuff for the UI camera, add the component
    // RenderLayers::layer(1) to the entity.
    // It will then be rendered without moving with the map camera.
    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                priority: 1,
                ..Default::default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..Default::default()
        })
        .insert(RenderLayers::layer(1))
        .insert(UiCamera);
}
