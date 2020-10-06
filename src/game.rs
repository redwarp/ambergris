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

        println!("Looking for enemies");
        let mut attack_action = None;
        for (entity, body, _, _) in enemies.iter(&self.world) {
            let body: &Body = body; // That seems to help.
            if body.position() == position {
                // We can attack a monster!
                println!("Let's attack the {}", body.name);
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
}

#[derive(PartialEq, Clone, Debug)]
pub enum RunState {
    Init,
    WaitForInput,
    PlayerTurn,
    AiTurn,
    Exit,
    Death,
}
