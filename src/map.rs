use bevy::prelude::*;

use crate::graphics::{Graphics, TILE_SIZE};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::default())
            .add_startup_system(create_map);
    }
}

#[derive(Component)]
struct Tile;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource)]
pub struct Map {
    pub spawn_point: Position,
    tiles: Vec<Entity>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            spawn_point: Position::default(),
            tiles: Vec::new(),
        }
    }
}

pub fn create_map(mut commands: Commands, graphics: Res<Graphics>, mut map: ResMut<Map>) {
    let mut spawn_point = None;
    // Debug code, will trash that later.
    let tiles = std::fs::read_to_string("assets/map.txt")
        .unwrap()
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .map(|(x, char)| {
                    if char == 'x' {
                        spawn_point = Some(Position {
                            x: x as i32,
                            y: y as i32,
                        });
                        println!("Spawn point: {:?}", spawn_point);
                    }
                    spawn_tile(&mut commands, &graphics, x as i32, y as i32, char)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    *map = Map {
        spawn_point: spawn_point.unwrap_or_else(|| Position { x: 0, y: 0 }),
        tiles,
    };
}

fn spawn_tile(commands: &mut Commands, graphics: &Graphics, x: i32, y: i32, tile: char) -> Entity {
    let tile = match tile {
        '#' => 16,
        _ => 48,
    };

    commands
        .spawn((
            SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(tile),
                texture_atlas: graphics.tiles_atlas.clone(),
                transform: Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.0),
                ..Default::default()
            },
            Tile,
            Position { x, y },
        ))
        .id()
}
