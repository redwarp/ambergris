use crate::colors::Color;
use crate::game::Ai;
use crate::map::Position;
use legion::Entity;

pub struct Body {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub blocking: bool,
    pub char: char,
    pub color: Color,
}

impl Body {
    pub fn position(&self) -> Position {
        (self.x, self.y)
    }

    pub fn set_position(&mut self, position: Position) {
        self.x = position.0;
        self.y = position.1;
    }

    pub fn distance_to(&self, position: Position) -> f32 {
        ((self.x - position.0).pow(2) as f32 + (self.y - position.1).pow(2) as f32).sqrt()
    }
}

pub struct Player {
    pub speed: u32,
}

pub struct Monster {
    pub ai: Ai,
    pub speed: u32,
    pub tick: i32,
}

pub struct MoveAction {
    pub entity: Entity,
    pub dx: i32,
    pub dy: i32,
}

pub struct AttackAction {
    pub attacker_entity: Entity,
    pub target_entity: Entity,
}

pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub attack: i32,
}

pub struct Item {}

pub struct ProvidesHealing {
    pub heal_amount: i32,
}

pub struct InInventory {
    pub owner: Entity,
}

pub struct PickupItemAction {
    pub collected_by: Entity,
    pub item: Entity,
}
