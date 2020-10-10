use crate::components::*;
use crate::game::RunState;
use crate::map::Map;
use crate::resources::SharedInfo;
use crate::{colors::DARK_RED, game::Journal};
use field_of_vision::FovMap;
use legion::component;
use legion::system;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::Entity;
use legion::IntoQuery;
use legion::Schedule;

pub fn game_schedule() -> Schedule {
    Schedule::builder()
        .add_system(monster_action_system())
        .flush()
        .add_system(attack_actions_system())
        .add_system(move_actions_system())
        .add_system(item_collection_system())
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
    body: &Body,
    coordinates: &Coordinates,
    _: &Monster,
    _: &CombatStats,
    entity: &Entity,
    #[resource] shared_info: &SharedInfo,
    #[resource] run_state: &RunState,
    #[resource] fov: &FovMap,
) {
    if *run_state != RunState::AiTurn {
        return;
    }
    let player_position = shared_info.player_position;
    let distance = coordinates.distance_to(player_position);
    if fov.is_in_fov(coordinates.x as isize, coordinates.y as isize) {
        println!("The {} sees you.", body.name);
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
                attacker_entity: entity.clone(),
                target_entity: shared_info.player_entity.clone(),
            };
            cmd.push((attack_action,));
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
pub fn attack_actions(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    move_action: &AttackAction,
    entity: &Entity,
    #[resource] journal: &mut Journal,
) {
    cmd.remove(*entity);

    let attacker = <(&Body, &CombatStats)>::query().get(world, move_action.attacker_entity);
    if attacker.is_err() {
        return;
    };
    let (attacker_body, attacker_stats) = attacker.unwrap();

    let attacker_name = attacker_body.name.clone();
    let attacker_attack = attacker_stats.attack;

    let target = <(&Body, &mut CombatStats)>::query().get_mut(world, move_action.target_entity);
    if target.is_err() {
        return;
    }
    let (target_body, target_stats): (&Body, &mut CombatStats) = target.unwrap();

    let damage = attacker_attack - target_stats.defense;

    if damage > 0 {
        journal.log(format!(
            "The {} attacks the {} for {} damage.",
            attacker_name, target_body.name, damage
        ));
    } else {
        journal.log(format!(
            "The {} is too weak to damage the {}.",
            attacker_name, target_body.name
        ));
    }

    target_stats.hp = (target_stats.hp - damage).max(0);
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
