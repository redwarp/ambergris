use bevy::prelude::*;
use torchbearer::{fov::VisionMap, path::PathMap};

use crate::graphics::{Graphics, TILE_SIZE};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>().add_startup_system(create_map);
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

#[derive(Resource)]
pub struct Map {
    pub size: Size,
    pub spawn_point: Position,
    cells: Vec<MapCell>,
    _tiles_id: Vec<Entity>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            size: Size::default(),
            spawn_point: Position::default(),
            cells: Vec::new(),
            _tiles_id: Vec::new(),
        }
    }
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

struct MapCell {
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

pub fn create_map(mut commands: Commands, graphics: Res<Graphics>, mut map: ResMut<Map>) {
    let mut spawn_point = None;
    // Debug code, will trash that later.
    let mut height = 0;
    let mut width = 0;

    let (cells, _tiles_id): (Vec<MapCell>, Vec<Entity>) = std::fs::read_to_string("assets/map.txt")
        .unwrap()
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            height += 1;
            width = line.len();

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
                    (
                        char.into(),
                        spawn_tile(&mut commands, &graphics, x as i32, y as i32, char),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unzip();

    println!("Map size: {}x{}", width, height);

    *map = Map {
        size: Size {
            width: width as u32,
            height: height as u32,
        },
        spawn_point: spawn_point.unwrap_or_else(|| Position { x: 0, y: 0 }),
        cells,
        _tiles_id,
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
