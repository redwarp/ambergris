use crate::engine::Engine;
use crate::map::Map;
use crate::resources::PlayerInfo;
use crate::{components::*, game::State};
use field_of_vision::FovMap;
use legion::{Resources, World};
use rand::rngs::StdRng;
use rand::SeedableRng;
mod components;
mod engine;
mod game;
mod map;
mod resources;
mod systems;

fn main() {
    println!("Hello, world!");

    let mut rng = StdRng::seed_from_u64(42);
    let mut world = World::default();
    let mut resources = Resources::default();
    let player_entity = world.push((
        Player,
        Body {
            name: "player".into(),
            x: 10,
            y: 10,
            blocking: true,
            char: '@',
            color: tcod::colors::YELLOW,
        },
    ));
    let map = crate::map::make_map(&mut world, &mut rng);
    let fov = make_fov(&map);
    resources.insert(map);
    resources.insert(fov);
    let mut state = State {
        world,
        resources,
        player_entity,
    };
    state.resources.insert(PlayerInfo { position: (0, 0) });

    let mut renderer = Engine::new();

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
