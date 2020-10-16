use crate::colors::Color;
use crate::game::Ai;
use crate::map::Position;
use legion::Entity;

pub struct Body {
    pub name: String,
    pub blocking: bool,
    pub char: char,
    pub color: Color,
}

#[derive(Debug, PartialEq)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

impl Coordinates {
    pub fn new(x: i32, y: i32) -> Self {
        Coordinates { x, y }
    }

    pub fn distance_to(&self, position: Position) -> f32 {
        ((self.x - position.0).pow(2) as f32 + (self.y - position.1).pow(2) as f32).sqrt()
    }

    pub fn position(&self) -> Position {
        (self.x, self.y)
    }

    pub fn set_position(&mut self, position: Position) {
        self.x = position.0;
        self.y = position.1;
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

impl CombatStats {
    pub fn heal(&mut self, healing_amount: i32) {
        self.hp = (self.hp + healing_amount).max(0).min(self.max_hp);
    }
}

pub struct Item {}

pub struct ProvidesHealing {
    pub heal_amount: i32,
}

pub struct Consumable {}

pub struct Ranged {
    pub range: i32,
    pub burst: i32,
}

pub struct InflictsDamage {
    pub damage: i32,
}

pub struct InInventory {
    pub owner: Entity,
}

pub struct PickupItemAction {
    pub collected_by: Entity,
    pub item: Entity,
}

pub struct UseItemIntent {
    pub item_entity: Entity,
}

pub struct DropItemIntent {
    pub item_entity: Entity,
}
