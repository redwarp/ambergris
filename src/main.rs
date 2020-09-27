use crate::component::*;
use crate::engine::Engine;
use legion::World;

use rand::rngs::StdRng;
use rand::SeedableRng;

mod component;
mod engine;
mod game;
mod map;

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
    let mut state = crate::game::State { world, map };

    let mut renderer = Engine::new();

    renderer.run(&mut state);
}
