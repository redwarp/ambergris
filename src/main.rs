use crate::components::*;
use crate::engine::Engine;
use crate::resources::PlayerInfo;
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
    let _player = world.push((
        Player,
        Body {
            x: 10,
            y: 10,
            blocking: true,
        },
        Renderable {
            char: '@',
            color: tcod::colors::YELLOW,
        },
    ));
    let map = crate::map::make_map(&mut world, &mut rng);
    let mut state = crate::game::State {
        world,
        map,
        resources: Resources::default(),
    };
    state.resources.insert(PlayerInfo {
        entity: _player,
        position: (0, 0),
    });

    let mut renderer = Engine::new();

    renderer.run(&mut state);
}
