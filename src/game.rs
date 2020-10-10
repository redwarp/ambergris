use std::collections::VecDeque;

use crate::components::*;
use crate::map::Map;
use legion::Entity;
use legion::IntoQuery;
use legion::Resources;
use legion::World;

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
        let body = <&Body>::query()
            .get(&self.world, self.player_entity)
            .unwrap();

        let position = (body.x + dx, body.y + dy);
        let map = self.resources.get::<Map>().unwrap();

        let mut enemies = <(Entity, &Body, &Monster, &CombatStats)>::query();

        let mut attack_action = None;
        for (entity, body, _, _) in enemies.iter(&self.world) {
            let body: &Body = body; // That seems to help.
            if body.position() == position {
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

    pub fn log<T: Into<String>>(&self, text: T) {
        if let Some(mut journal) = self.resources.get_mut::<Journal>() {
            journal.log(text);
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum RunState {
    Init,
    WaitForPlayerInput,
    PlayerTurn,
    AiTurn,
    Exit,
    Death,
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
