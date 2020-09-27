use crate::components::*;
use crate::resources::PlayerInfo;
use legion::component;
use legion::system;

#[system]
pub fn new_turn() {
    println!("You took a turn!");
}

#[system(for_each)]
#[filter(!component::<Player>())]
pub fn monster_move(body: &Body, #[resource] player_info: &PlayerInfo) {
    if body.distance_to(player_info.position) < 5.0 {
        println!("The {} sees you.", body.name);
    }
}
