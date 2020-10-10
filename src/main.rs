use crate::game::State;
use crate::map::Map;
use crate::pistonengine::Engine as PistonEngine;
use crate::resources::SharedInfo;

use field_of_vision::FovMap;
use game::Journal;
use legion::{Resources, World};
use rand::rngs::StdRng;
use rand::SeedableRng;
mod colors;
mod components;
mod game;
mod map;
mod pistonengine;
mod renderer;
mod resources;
mod spawner;
mod systems;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut world = World::default();
    let mut resources = Resources::default();
    let player_entity = world.push(spawner::spawn_player(-1, -1));
    let map = crate::map::make_map(&mut world, &mut rng);
    let fov = make_fov(&map);
    let journal = Journal::new();
    resources.insert(map);
    resources.insert(fov);
    resources.insert(journal);
    resources.insert(SharedInfo {
        player_entity: player_entity,
        player_position: (-1, -1),
        alive: true,
    });
    let mut state = State {
        world,
        resources,
        player_entity,
    };
    state.log("Welcome to Ambergris");

    let mut renderer = PistonEngine::new("Ambergris", SCREEN_WIDTH, SCREEN_HEIGHT);
    renderer.run(&mut state);
}

fn make_fov(map: &Map) -> FovMap {
    let mut fov = FovMap::new(map.width as isize, map.height as isize);

    for y in 0..map.height {
        for x in 0..map.width {
            fov.set_transparent(
                x as isize,
                y as isize,
                !map.tiles[x as usize + y as usize * map.width as usize].block_sight,
            );
        }
    }

    fov
}
