use crate::game::RunState;
use crate::map::Map;
use crate::resources::SharedInfo;
use crate::{colors::DARK_RED, game::Journal};
use crate::{components::*, game::Ai};
use legion::system;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::Entity;
use legion::IntoQuery;
use legion::Schedule;
use legion::{component, Write};

pub fn game_schedule() -> Schedule {
    Schedule::builder()
        .add_system(monster_action_system())
        .add_system(use_item_system())
        .add_system(drop_item_system())
        .flush()
        .add_system(attack_actions_system())
        .add_system(move_actions_system())
        .add_system(item_collection_system())
        .flush()
        .add_system(damage_system())
        .flush()
        .add_system(cleanup_deads_system())
        .add_system(update_map_and_position_system())
        .add_system(update_game_state_system())
        .build()
}

#[system(for_each)]
#[filter(!component::<Player>())]
#[read_component(Player)]
pub fn monster_action(
    cmd: &mut CommandBuffer,
    coordinates: &Coordinates,
    monster: &Monster,
    _: &CombatStats,
    entity: &Entity,
    #[resource] shared_info: &SharedInfo,
    #[resource] run_state: &RunState,
    #[resource] map: &Map,
) {
    if *run_state != RunState::AiTurn {
        return;
    }

    if monster.ai == Ai::Basic {
        let player_position = shared_info.player_position;
        let distance = coordinates.distance_to(player_position);
        if map.is_in_player_fov(coordinates.x, coordinates.y) {
            if distance >= 2.0 {
                let dx = player_position.0 - coordinates.x;
                let dy = player_position.1 - coordinates.y;

                let dx = (dx as f32 / distance).round() as i32;
                let dy = (dy as f32 / distance).round() as i32;

                cmd.push((MoveAction {
                    entity: *entity,
                    dx,
                    dy,
                },));
            } else {
                // Attack!
                let attack_action = AttackAction {
                    target_entity: shared_info.player_entity.clone(),
                };
                cmd.add_component(*entity, attack_action);
            }
        }
    }
}

#[system]
#[read_component(Player)]
#[read_component(Body)]
#[read_component(Coordinates)]
pub fn update_map_and_position(
    world: &mut SubWorld,
    #[resource] map: &mut Map,
    #[resource] shared_info: &mut SharedInfo,
) {
    for (index, tile) in map.tiles.iter().enumerate() {
        map.blocked[index] = tile.blocking;
    }

    let mut body_query = <(&Body, &Coordinates)>::query();
    for (body, coordinates) in body_query.iter_mut(world) {
        if body.blocking {
            let index = map.index(coordinates.position());
            map.blocked[index] = true;
        }
    }
    let mut player_query = <&Coordinates>::query().filter(component::<Player>());
    let player_coordinates = player_query.iter(world).next().unwrap();
    shared_info.player_position = player_coordinates.position();
}

#[system(for_each)]
#[write_component(Coordinates)]
pub fn move_actions(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    move_action: &MoveAction,
    entity: &Entity,
    #[resource] map: &mut Map,
) {
    let mut query = <&mut Coordinates>::query();

    let coordinates = query.get_mut(world, move_action.entity);
    if let Ok(coordinates) = coordinates {
        let old_position = coordinates.position();
        let new_position = (
            coordinates.x + move_action.dx,
            coordinates.y + move_action.dy,
        );
        if !map.is_blocked(new_position) {
            coordinates.set_position(new_position);
            // Update map of blocked. It can seem useless but if not for that code,
            // the next entity might try to also move on the same tile.
            let old_index = map.index(old_position);
            let new_index = map.index(new_position);
            map.blocked[old_index] = false;
            map.blocked[new_index] = true;
        }
    }

    cmd.remove(*entity);
}

#[system(for_each)]
#[read_component(Body)]
#[write_component(CombatStats)]
#[write_component(SuffersDamage)]
pub fn attack_actions(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    move_action: &AttackAction,
    entity: &Entity,
    #[resource] journal: &mut Journal,
) {
    cmd.remove_component::<AttackAction>(*entity);

    let attacker = <(&Body, &CombatStats)>::query().get(world, *entity);
    if attacker.is_err() {
        return;
    };
    let (attacker_body, attacker_stats) = attacker.unwrap();

    let attacker_name = attacker_body.name.clone();
    let attacker_attack = attacker_stats.attack;

    let target = <(&Body, &CombatStats)>::query().get(world, move_action.target_entity);
    if target.is_err() {
        return;
    }
    let (target_body, target_stats): (&Body, &CombatStats) = target.unwrap();

    let damage = attacker_attack - target_stats.defense;

    if damage > 0 {
        journal.log(format!(
            "The {} attacks the {} for {} damage.",
            attacker_name, target_body.name, damage
        ));
        let suffers_damage = SuffersDamage {
            entity: move_action.target_entity,
            damage,
        };
        cmd.push((suffers_damage,));
    } else {
        journal.log(format!(
            "The {} is too weak to damage the {}.",
            attacker_name, target_body.name
        ));
    }
}

#[system(for_each)]
#[write_component(CombatStats)]
pub fn damage(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    entity: &Entity,
    suffers_damage: &SuffersDamage,
) {
    if let Ok(combat_stats) = <&mut CombatStats>::query().get_mut(world, suffers_damage.entity) {
        combat_stats.take_damage(suffers_damage.damage);
    }

    cmd.remove(*entity);
}

#[system(for_each)]
#[read_component(Body)]
#[read_component(ProvidesHealing)]
#[read_component(Consumable)]
#[read_component(Burst)]
#[read_component(Coordinates)]
#[read_component(InflictsDamage)]
#[write_component(CombatStats)]
pub fn use_item(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    use_item_action: &UseItemIntent,
    entity: &Entity,
    #[resource] journal: &mut Journal,
    #[resource] map: &Map,
) {
    cmd.remove_component::<UseItemIntent>(*entity);

    let mut targets: Vec<Entity> = vec![];
    match use_item_action.target {
        Some((x, y)) => {
            let positions: Vec<(i32, i32)>;
            let radius =
                if let Ok(burst) = <&Burst>::query().get(world, use_item_action.item_entity) {
                    Some(burst.radius)
                } else {
                    None
                };

            match radius {
                Some(radius) => {
                    positions = field_of_vision::field_of_view(map, x, y, radius, false);
                }
                None => {
                    positions = vec![(x, y)];
                }
            }

            for (entity, coordinates) in <(Entity, &Coordinates)>::query().iter(world) {
                if positions.contains(&(coordinates.x, coordinates.y)) {
                    targets.push(*entity);
                }
            }
        }
        None => {
            targets.push(*entity);
        }
    }

    let name = <&Body>::query().get(world, *entity).unwrap().name.clone();

    if let Ok(item_body) = <&Body>::query().get(world, use_item_action.item_entity) {
        journal.log(format!("The {} uses the {}", name, item_body.name));
    }

    for target in targets {
        let name = <&Body>::query().get(world, target).unwrap().name.clone();

        let mut stats_query = <Write<CombatStats>>::query();
        let (mut stats_world, mut healing_world) = world.split_for_query(&stats_query);

        if let (Ok(stats), Ok(healing)) = (
            stats_query.get_mut(&mut stats_world, target),
            <&ProvidesHealing>::query().get(&mut healing_world, use_item_action.item_entity),
        ) {
            journal.log(format!("The {} heal {} hp", name, healing.heal_amount));
            stats.heal(healing.heal_amount);
        }

        if let Ok(damage) =
            <&InflictsDamage>::query().get(&mut healing_world, use_item_action.item_entity)
        {
            journal.log(format!("The {} take {} damage", name, damage.damage));
            cmd.push((SuffersDamage {
                entity: target,
                damage: damage.damage,
            },));
        }
    }

    if let Ok(_consumable) = <&Consumable>::query().get(world, use_item_action.item_entity) {
        cmd.remove(use_item_action.item_entity);
    }
}

#[system(for_each)]
pub fn cleanup_deads(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    body: &mut Body,
    combat_stats: &CombatStats,
    #[resource] journal: &mut Journal,
) {
    if combat_stats.hp == 0 {
        // We found a cadaver!
        journal.log(format!("The {} is dead.", body.name));

        body.char = '%';
        body.color = DARK_RED;
        body.blocking = false;
        body.name = format!("{}'s body", body.name);

        cmd.remove_component::<CombatStats>(*entity);
    }
}

#[system(for_each)]
#[filter(component::<Player>())]
pub fn update_game_state(
    body: &mut Body,
    #[resource] shared_info: &mut SharedInfo,
    #[resource] journal: &mut Journal,
) {
    if body.char == '%' {
        // All is lost.
        journal.log("All is lost!!!");
        shared_info.alive = false;
    }
}

#[system(for_each)]
#[read_component(Body)]
pub fn item_collection(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    action: &PickupItemAction,
    entity: &Entity,
    #[resource] journal: &mut Journal,
) {
    let in_inventory = InInventory {
        owner: action.collected_by,
    };
    cmd.add_component(action.item, in_inventory);
    cmd.remove_component::<Coordinates>(action.item);

    let collector_name = <&Body>::query()
        .get(world, action.collected_by)
        .map(|body| body.name.clone());
    let item_name = <&Body>::query()
        .get(world, action.item)
        .map(|body| body.name.clone());

    if let (Ok(collector_name), Ok(item_name)) = (collector_name, item_name) {
        journal.log(format!(
            "The {} picks up the {}.",
            collector_name, item_name
        ));
    }

    cmd.remove(*entity);
}

#[system(for_each)]
#[read_component(Body)]
pub fn drop_item(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    intent: &DropItemIntent,
    owner_coordinates: &Coordinates,
    owner_entity: &Entity,
    #[resource] journal: &mut Journal,
) {
    cmd.remove_component::<DropItemIntent>(*owner_entity);
    let item_coordinates = Coordinates {
        x: owner_coordinates.x,
        y: owner_coordinates.y,
    };

    cmd.add_component(intent.item_entity, item_coordinates);
    cmd.remove_component::<InInventory>(intent.item_entity);

    let owner_name = <&Body>::query()
        .get(world, *owner_entity)
        .unwrap()
        .name
        .clone();
    if let Ok(item_body) = <&Body>::query().get(world, intent.item_entity) {
        journal.log(format!("The {} dropped the {}", owner_name, item_body.name));
    }
}
