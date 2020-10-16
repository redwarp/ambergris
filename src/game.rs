use std::collections::VecDeque;

use crate::components::*;
use crate::map::Map;
use legion::Entity;
use legion::IntoQuery;
use legion::Resources;
use legion::World;
use legion::*;

pub struct State {
    pub world: World,
    pub resources: Resources,
    pub player_entity: Entity,
}

pub enum Ai {
    Basic,
}

impl State {
    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let coordinates = <&Coordinates>::query()
            .get(&self.world, self.player_entity)
            .unwrap();

        let position = (coordinates.x + dx, coordinates.y + dy);
        let map = self.resources.get::<Map>().unwrap();

        let mut enemies = <(Entity, &Coordinates, &Monster, &CombatStats)>::query();

        let mut attack_action = None;
        for (entity, body, _, _) in enemies.iter(&self.world) {
            let coordinates: &Coordinates = body; // That seems to help.
            if coordinates.position() == position {
                // We can attack a monster!
                attack_action = Some(AttackAction {
                    attacker_entity: self.player_entity,
                    target_entity: entity.clone(),
                });
                break;
            }
        }

        match attack_action {
            Some(attack_action) => {
                self.world.push((attack_action,));
            }
            None => {
                if !map.is_blocked(position) {
                    self.world.push((MoveAction {
                        dx,
                        dy,
                        entity: self.player_entity,
                    },));
                };
            }
        }
    }

    pub fn grab_item(&mut self) -> bool {
        let position = <&Coordinates>::query()
            .get(&self.world, self.player_entity)
            .map(|coordinates| coordinates.position())
            .unwrap();

        let mut pickup_item_action = None;
        let mut grabbed_item = false;

        <(&Coordinates, Entity)>::query()
            .filter(component::<Item>())
            .filter(!component::<InInventory>())
            .for_each(
                &self.world,
                |(coordinates, entity): (&Coordinates, &Entity)| {
                    if coordinates.position() == position {
                        // We can grab!
                        pickup_item_action = Some((PickupItemAction {
                            collected_by: self.player_entity,
                            item: *entity,
                        },));
                        grabbed_item = true;
                    }
                },
            );

        if let Some(pickup_item_action) = pickup_item_action {
            self.world.push(pickup_item_action);
        }

        grabbed_item
    }

    pub fn use_item(&mut self, item: Entity) -> RunState {
        if let Ok(ranged) = <&Ranged>::query().get(&self.world, item) {
            RunState::ShowTargeting {
                item,
                range: ranged.range,
                burst: ranged.burst,
            }
        } else {
            let use_item_action = UseItemAction { entity: item };

            if let Some(mut player_entry) = self.world.entry(self.player_entity) {
                player_entry.add_component(use_item_action);
            }
            RunState::PlayerTurn
        }
    }

    pub fn log<T: Into<String>>(&self, text: T) {
        if let Some(mut journal) = self.resources.get_mut::<Journal>() {
            journal.log(text);
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    Init,
    WaitForPlayerInput,
    PlayerTurn,
    AiTurn,
    Exit,
    Death,
    ShowInventory,
    ShowTargeting {
        item: Entity,
        range: i32,
        burst: i32,
    },
}

pub struct Targetting {
    pub item: Entity,
    pub range: i32,
    pub burst: i32,
}

pub struct Journal {
    /// The maximum amount of entries to keep in memory.
    size: usize,
    entries: VecDeque<String>,
}

impl Journal {
    pub fn new() -> Self {
        Journal {
            size: 10,
            entries: VecDeque::with_capacity(12),
        }
    }

    pub fn log<S: Into<String>>(&mut self, entry: S) {
        self.entries.push_front(entry.into());
        while self.entries.len() > self.size {
            self.entries.pop_back();
        }
    }

    pub fn get_entries(&self) -> &VecDeque<String> {
        &self.entries
    }
}
