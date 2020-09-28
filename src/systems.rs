use crate::components::*;
use crate::map::Map;
use crate::resources::PlayerInfo;
use legion::component;
use legion::system;
use legion::world::SubWorld;
use legion::*;

#[system]
pub fn new_turn() {
    println!("You took a turn!");
}

#[system(for_each)]
#[filter(!component::<Player>())]
#[read_component(Player)]
pub fn monster_move(
    world: &mut SubWorld,
    body: &Body,
    monster: &Monster,
    #[resource] player_info: &PlayerInfo,
    #[resource] map: &Map,
) {
    if body.distance_to(player_info.position) < 5.0 {
        println!("The {} sees you.", body.name);
    }
}
