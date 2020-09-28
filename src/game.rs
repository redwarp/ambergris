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

pub type Position = (i32, i32);

impl State {
    pub fn ai_turn() {}

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let body = <&Body>::query()
            .get(&mut self.world, self.player_entity)
            .unwrap();

        let (new_x, new_y) = (body.x + dx, body.y + dy);
        let map = self.resources.get::<Map>().unwrap();

        if !map.is_blocked(new_x, new_y, &self.world) {
            let body = <&mut Body>::query()
                .get_mut(&mut self.world, self.player_entity)
                .unwrap();
            body.x = new_x;
            body.y = new_y;
        }
        drop(map);
        self.resources.insert(PlayerInfo {
            position: (new_x, new_y),
        })
    }
}

pub enum RunState {
    WaitForInput,
    PlayerTurn,
    AiTurn,
    Exit,
}
