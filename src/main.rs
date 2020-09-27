use crate::components::*;
use crate::engine::Engine;
use crate::resources::PlayerInfo;
use game::RunState;
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
    let mut state = crate::game::State {
        world,
        map,
        resources: Resources::default(),
        run_state: RunState::Running,
        player_entity,
    };
    state.resources.insert(PlayerInfo { position: (0, 0) });

    let mut renderer = Engine::new();

    renderer.run(&mut state);
}
