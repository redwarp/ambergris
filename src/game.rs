use crate::components::*;
use crate::map::Map;
use legion::query;
use legion::IntoQuery;
use legion::Resources;
use legion::World;

pub struct State {
    pub world: World,
    pub map: Map,
    pub resources: Resources,
}

pub enum Ai {
    Basic,
}

pub type Position = (i32, i32);

impl State {
    pub fn ai_turn() {}

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let mut query = <(&Player, &mut Body)>::query();
        let (_, body) = query.iter_mut(&mut self.world).next().unwrap();

        let (new_x, new_y) = (body.x + dx, body.y + dy);

        if !self.map.is_blocked(new_x, new_y, &self.world) {
            let (_, body) = query.iter_mut(&mut self.world).next().unwrap();
            body.x = new_x;
            body.y = new_y;
        }
    }
}
