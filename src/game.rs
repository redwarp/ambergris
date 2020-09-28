use crate::components::*;
use crate::map::Map;
use crate::resources::PlayerInfo;
use legion::Entity;
use legion::IntoQuery;
use legion::Resources;
use legion::World;

pub struct State {
    pub world: World,
    pub resources: Resources,
    pub run_state: RunState,
    pub player_entity: Entity,
}

pub enum Ai {
    Basic,
}

impl State {
    pub fn ai_turn() {}

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let body = <&Body>::query()
            .get(&mut self.world, self.player_entity)
            .unwrap();

        let position = (body.x + dx, body.y + dy);
        let map = self.resources.get::<Map>().unwrap();

        if !map.is_blocked(position) {
            self.world.push((MoveAction {
                dx,
                dy,
                entity: self.player_entity,
            },));
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum RunState {
    Init,
    WaitForInput,
    PlayerTurn,
    AiTurn,
    Exit,
}
