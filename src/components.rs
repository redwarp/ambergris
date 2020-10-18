use crate::colors::Color;
use crate::game::Ai;
use legion::Entity;

pub struct Body {
    pub name: String,
    pub blocking: bool,
    pub char: char,
    pub color: Color,
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

    pub fn take_damage(&mut self, damage: i32) {
        self.hp = (self.hp - damage).max(0).min(self.max_hp);
    }
}

pub struct Item {}

pub struct ProvidesHealing {
    pub heal_amount: i32,
}

pub struct Consumable {}

pub struct Ranged {
    pub range: i32,
}

pub struct Burst {
    pub radius: i32,
}

pub struct InflictsDamage {
    pub damage: i32,
}
pub struct SuffersDamage {
    pub entity: Entity,
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
    pub target: Option<(i32, i32)>,
}

pub struct DropItemIntent {
    pub item_entity: Entity,
}
