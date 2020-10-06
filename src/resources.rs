use legion::Entity;

use crate::map::Position;

pub struct SharedInfo {
    pub player_entity: Entity,
    pub player_position: Position,
    pub alive: bool,
}
