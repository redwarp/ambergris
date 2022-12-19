use bevy::{
    prelude::{Commands, Transform},
    sprite::{SpriteSheetBundle, TextureAtlasSprite},
};

use crate::{
    graphics::{Graphics, TILE_SIZE},
    map::{Position, Solid},
    monsters::Monster,
    player::Player,
};

pub fn spawn_creature(
    commands: &mut Commands,
    graphics: &Graphics,
    spawn_type: char,
    x: i32,
    y: i32,
) {
    match spawn_type {
        '@' => {
            spawn_player(commands, graphics, x, y);
        }
        'd' => {
            spawn_deer(commands, graphics, x, y);
        }
        _ => {}
    }
}

pub fn spawn_player(commands: &mut Commands, graphics: &Graphics, x: i32, y: i32) {
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: graphics.characters_atlas.clone(),
            transform: Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 10.0),
            ..Default::default()
        },
        Player,
        Position { x, y },
        Solid,
    ));
}

pub fn spawn_deer(commands: &mut Commands, graphics: &Graphics, x: i32, y: i32) {
    commands.spawn((
        Monster,
        Position { x, y },
        Solid,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(2),
            texture_atlas: graphics.characters_atlas.clone(),
            transform: Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 10.0),
            ..Default::default()
        },
    ));
}
