use std::collections::VecDeque;

use crate::map::Map;
use crate::{components::*, map::Position};
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

#[derive(PartialEq)]
pub enum Ai {
    Basic,
}

impl State {
    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let position = <&Position>::query()
            .get(&self.world, self.player_entity)
            .unwrap();
        let position = Position {
            x: position.x + dx,
            y: position.y + dy,
        };

        let map = self.resources.get::<Map>().unwrap();

        let mut enemies = <(Entity, &Position, &Monster, &CombatStats)>::query();

        let mut attack_action = None;
        for (entity, enemy_position, _, _) in enemies.iter(&self.world) {
            if *enemy_position == position {
                // We can attack a monster!
                attack_action = Some(AttackAction {
                    target_entity: entity.clone(),
                });
                break;
            }
        }

        match attack_action {
            Some(attack_action) => {
                if let Some(mut entry) = self.world.entry(self.player_entity) {
                    entry.add_component(attack_action);
                }
            }
            None => {
                if !map.is_blocked(position) {
                    self.world.push((MoveAction {
                        dx,
                        dy,
                        entity: self.player_entity,
                    },));
                }
            }
        }
    }

    pub fn grab_item(&mut self) -> bool {
        let position = <&Position>::query()
            .get(&self.world, self.player_entity)
            .unwrap();

        let mut pickup_item_action = None;
        let mut grabbed_item = false;

        <(&Position, Entity)>::query()
            .filter(component::<Item>())
            .filter(!component::<InInventory>())
            .for_each(
                &self.world,
                |(coordinates, entity): (&Position, &Entity)| {
                    if coordinates == position {
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

    pub fn use_item(&mut self, item_entity: Entity) -> RunState {
        // Check if we need burst targeting
        let radius = if let Ok(burst) = <&Burst>::query().get(&self.world, item_entity) {
            burst.radius
        } else {
            0
        };

        if let Ok(ranged) = <&Ranged>::query().get(&self.world, item_entity) {
            RunState::ShowTargeting {
                item: item_entity,
                range: ranged.range,
                burst: radius,
            }
        } else {
            // An item to use on ourselves.
            let use_item_intent = UseItemIntent {
                item_entity,
                target: None,
            };

            if let Some(mut player_entry) = self.world.entry(self.player_entity) {
                player_entry.add_component(use_item_intent);
            }
            RunState::PlayerTurn
        }
    }

    pub fn drop_item(&mut self, item_entity: Entity) -> RunState {
        let drop_item_intent = DropItemIntent { item_entity };

        if let Some(mut entry) = self.world.entry(self.player_entity) {
            entry.add_component(drop_item_intent);
            // entry.remove_component::<InInventory>();
            // entry.add_component(Coordinates { x, y });
        }

        RunState::PlayerTurn
    }

    pub fn use_range_item_with_targeting(
        &mut self,
        previous_state: RunState,
        item_entity: Entity,
        target_position: (i32, i32),
    ) -> RunState {
        // Check if we need burst targeting
        let radius = if let Ok(burst) = <&Burst>::query().get(&self.world, item_entity) {
            burst.radius
        } else {
            0
        };

        if radius == 0 {
            // We need to verify we could actually get a target.
            for coordinates in <&Position>::query().iter(&self.world) {
                if target_position.0 == coordinates.x && target_position.1 == coordinates.y {
                    // We have a match!
                    let use_item_intent = UseItemIntent {
                        item_entity,
                        target: Some(target_position),
                    };

                    if let Some(mut player_entry) = self.world.entry(self.player_entity) {
                        player_entry.add_component(use_item_intent);
                    }
                    return RunState::PlayerTurn;
                }
            }
            // Couldn't find a valid target, returning previous state.
            return previous_state;
        }

        // It's a burst, we can simply push an intent.
        let use_item_intent = UseItemIntent {
            item_entity,
            target: Some(target_position),
        };

        if let Some(mut player_entry) = self.world.entry(self.player_entity) {
            player_entry.add_component(use_item_intent);
        }
        return RunState::PlayerTurn;
    }

    pub fn log<T: Into<String>>(&self, text: T) {
        if let Some(mut journal) = self.resources.get_mut::<Journal>() {
            journal.log(text);
        }
    }

    pub fn interact(&self) -> Interact {
        let player_position = *<&Position>::query()
            .get(&self.world, self.player_entity)
            .unwrap();

        for (_, interactable) in <(&Position, &Interactable)>::query()
            .iter(&self.world)
            .filter(|(&position, _)| player_position == position)
        {
            return match interactable {
                Interactable::DownStairs => {
                    println!("Going downstairs...");
                    Interact::WentDownstairs
                }
            };
        }

        Interact::Canceled
    }

    pub fn next_level(&mut self) {
        let to_delete = self.find_entity_attached_to_map();

        for entity in to_delete {
            self.world.remove(entity);
        }

        let level = if let Some(map) = self.resources.get::<Map>() {
            map.depth + 1
        } else {
            0
        };

        let map = crate::map::make_map(&mut self.world, level);
        self.resources.insert(map);
    }

    pub fn find_entity_attached_to_map(&mut self) -> Vec<Entity> {
        let entities: Vec<Entity> = <Entity>::query()
            .filter(!component::<Player>())
            .iter(&self.world)
            .cloned()
            .collect();

        let mut to_delete = vec![];
        for entity in entities {
            let mut should_delete = true;
            let entry = self.world.entry(entity).unwrap();

            if let Ok(in_inventory) = entry.get_component::<InInventory>() {
                if in_inventory.owner == self.player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }
}

pub enum Interact {
    Canceled,
    WentDownstairs,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    Init,
    WaitForPlayerInput,
    PlayerTurn,
    AiTurn,
    Exit,
    Death,
    NextLevel,
    ShowInventory,
    ShowTargeting {
        item: Entity,
        range: i32,
        burst: i32,
    },
}

pub struct Targeting {
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
