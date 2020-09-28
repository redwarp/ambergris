use crate::engine::Engine;
use crate::resources::PlayerInfo;
use crate::{components::*, game::State};
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
    resources.insert(map);
    let mut state = State {
        world,
        resources,
        run_state: RunState::PlayerTurn,
        player_entity,
    };
    state.resources.insert(PlayerInfo { position: (0, 0) });

    let mut renderer = Engine::new();

    renderer.run(&mut state);
}
