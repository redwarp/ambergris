use bevy::prelude::*;

use crate::{
    graphics::{Graphics, MapCamera, TILE_SIZE},
    map::{create_map, Map},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(spawn_player.after(create_map))
            .add_system(camera_follow);
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands, graphics: Res<Graphics>, map: Res<Map>) {
    let spawn_position = map.spawn_point;
    println!("Reading spawn point: {spawn_position:?}");

    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: graphics.characters_atlas.clone(),
            transform: Transform::from_xyz(
                spawn_position.x as f32 * TILE_SIZE,
                -(spawn_position.y as f32) * TILE_SIZE,
                10.0,
            ),
            ..Default::default()
        },
        Player,
        spawn_position,
    ));
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera2d>, With<MapCamera>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}
