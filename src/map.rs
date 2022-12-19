use std::str::FromStr;

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use torchbearer::{fov::VisionMap, path::PathMap};

use crate::{
    graphics::{Graphics, TILE_SIZE},
    spawner::spawn_creature,
    stages::UpdateStages,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapInfo>()
            .add_startup_system(create_map)
            .add_system_to_stage(UpdateStages::UpdateMap, update_blocked_tiles)
            .register_inspectable::<Position>();
    }
}

#[derive(Component)]
struct Tile;

#[derive(Component, Default, Debug, Clone, Copy, Inspectable)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Solid;

#[derive(Default, Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct SpawnPoint {
    pub spawn_type: char,
    pub position: Position,
}

#[derive(Debug, Default)]
pub struct Map {
    pub size: Size,
    pub spawn_positions: Vec<SpawnPoint>,
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

    fn in_bounds(&self, (x, y): (i32, i32)) -> bool {
        x >= 0 && y >= 0 && x < self.size.width as i32 && y < self.size.height as i32
    }
}

impl FromStr for Map {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut height = 0;
        let mut width = 0;
        let mut spawn_positions = vec![];

        let cells: Vec<MapCell> = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                height += 1;
                width = line.len();

                line.chars()
                    .enumerate()
                    .map(|(x, char)| {
                        if char != '#' && char != ' ' {
                            spawn_positions.push(SpawnPoint {
                                spawn_type: char,
                                position: Position {
                                    x: x as i32,
                                    y: y as i32,
                                },
                            });
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
            spawn_positions,
            cells,
        })
    }
}

#[derive(Resource, Default)]
pub struct MapInfo {
    pub map: Map,
    pub blocked: Vec<bool>,
    _tiles_id: Vec<Entity>,
}

impl MapInfo {
    pub fn index_from_position(&self, position: &Position) -> usize {
        (position.y * self.map.size.width as i32 + position.x) as usize
    }

    pub fn in_bounds(&self, position: &Position) -> bool {
        self.map.in_bounds((position.x, position.y))
    }

    pub fn set_blocked(&mut self, position: &Position, blocked: bool) {
        if self.in_bounds(position) {
            let index = self.index_from_position(position);
            self.blocked[index] = blocked;
        }
    }

    pub fn reset_blocked(&mut self) {
        self.blocked.clear();
        self.blocked
            .extend(self.map.cells.iter().map(|cell| !cell.walkable));
    }
}

impl VisionMap for MapInfo {
    fn dimensions(&self) -> (i32, i32) {
        (self.map.size.width as i32, self.map.size.height as i32)
    }

    fn is_transparent(&self, (x, y): torchbearer::Point) -> bool {
        if self.map.in_bounds((x, y)) {
            self.map.cells[y as usize * self.map.size.width as usize + x as usize].transparent
        } else {
            false
        }
    }
}

impl PathMap for MapInfo {
    fn dimensions(&self) -> (i32, i32) {
        VisionMap::dimensions(self)
    }

    fn is_walkable(&self, (x, y): torchbearer::Point) -> bool {
        if self.map.in_bounds((x, y)) {
            !self.blocked[y as usize * self.map.size.width as usize + x as usize]
        } else {
            false
        }
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
    let blocked = map.cells.iter().map(|c| !c.walkable).collect();

    for SpawnPoint {
        spawn_type,
        position,
    } in map.spawn_positions.iter()
    {
        spawn_creature(
            &mut commands,
            &graphics,
            *spawn_type,
            position.x,
            position.y,
        );
    }

    *map_info = MapInfo {
        map,
        blocked,
        _tiles_id,
    };
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

fn update_blocked_tiles(query: Query<&Position, With<Solid>>, mut map_info: ResMut<MapInfo>) {
    map_info.reset_blocked();
    for position in query.iter() {
        map_info.set_blocked(position, true);
    }
}
