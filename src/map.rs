use std::str::FromStr;

use bevy::prelude::*;
use torchbearer::{fov::VisionMap, path::PathMap};

use crate::graphics::{Graphics, TILE_SIZE};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapInfo>()
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

#[derive(Default, Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default)]
pub struct Map {
    pub size: Size,
    pub spawn_point: Position,
    pub cells: Vec<MapCell>,
}

impl Map {
    fn spawn_sprites(&self, commands: &mut Commands, graphics: &Graphics) -> Vec<Entity> {
        self.cells
            .as_slice()
            .chunks_exact(self.size.width as usize)
            .enumerate()
            .fold(
                Vec::<Entity>::with_capacity(self.cells.len()),
                |mut acc, (y, row)| {
                    acc.extend(
                        row.iter().enumerate().map(|(x, cell)| {
                            spawn_tile(commands, graphics, x as i32, y as i32, cell)
                        }),
                    );
                    acc
                },
            )
    }
}

impl FromStr for Map {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spawn_point = None;
        let mut height = 0;
        let mut width = 0;

        let cells: Vec<MapCell> = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                height += 1;
                width = line.len();

                line.chars()
                    .enumerate()
                    .map(|(x, char)| {
                        if char == '@' {
                            spawn_point = Some(Position {
                                x: x as i32,
                                y: y as i32,
                            });
                            println!("Spawn point: {:?}", spawn_point);
                        }
                        char.into()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(Map {
            size: Size {
                width: width as u32,
                height: height as u32,
            },
            spawn_point: spawn_point.unwrap_or(Position { x: 0, y: 0 }),
            cells,
        })
    }
}

#[derive(Resource, Default)]
pub struct MapInfo {
    pub map: Map,
    _tiles_id: Vec<Entity>,
}

impl VisionMap for Map {
    fn dimensions(&self) -> (i32, i32) {
        (self.size.width as i32, self.size.height as i32)
    }

    fn is_transparent(&self, (x, y): torchbearer::Point) -> bool {
        self.cells[y as usize * self.size.width as usize + x as usize].transparent
    }
}

impl PathMap for Map {
    fn dimensions(&self) -> (i32, i32) {
        VisionMap::dimensions(self)
    }

    fn is_walkable(&self, (x, y): torchbearer::Point) -> bool {
        self.cells[y as usize * self.size.width as usize + x as usize].walkable
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapCell {
    walkable: bool,
    transparent: bool,
}

impl From<char> for MapCell {
    fn from(letter: char) -> Self {
        match letter {
            '#' => Self {
                walkable: false,
                transparent: false,
            },
            _ => Self {
                walkable: true,
                transparent: true,
            },
        }
    }
}

pub fn create_map(mut commands: Commands, graphics: Res<Graphics>, mut map_info: ResMut<MapInfo>) {
    let map = std::fs::read_to_string("assets/map.txt")
        .unwrap()
        .parse::<Map>()
        .unwrap();
    let _tiles_id = map.spawn_sprites(&mut commands, &graphics);

    *map_info = MapInfo { map, _tiles_id };
}

fn spawn_tile(
    commands: &mut Commands,
    graphics: &Graphics,
    x: i32,
    y: i32,
    cell: &MapCell,
) -> Entity {
    let tile = match cell.walkable {
        false => 16,
        true => 48,
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
