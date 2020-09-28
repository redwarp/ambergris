use crate::components::*;
use crate::map::Map;
use crate::map::Position;
use crate::resources::PlayerInfo;
use legion::component;
use legion::system;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::Entity;
use legion::IntoQuery;

#[system(for_each)]
#[filter(!component::<Player>())]
#[read_component(Player)]
pub fn monster_move(
    world: &mut SubWorld,
    body: &Body,
    monster: &Monster,
    #[resource] player_info: &PlayerInfo,
    #[resource] map: &Map,
) {
    if body.distance_to(player_info.position) < 5.0 {
        println!("The {} sees you.", body.name);
    }
}

#[system]
#[read_component(Player)]
#[read_component(Body)]
pub fn update_map_and_position(
    world: &mut SubWorld,
    #[resource] map: &mut Map,
    #[resource] player_info: &mut PlayerInfo,
) {
    for (index, tile) in map.tiles.iter().enumerate() {
        map.blocked[index] = tile.blocking;
    }

    let mut body_query = <&Body>::query();
    for body in body_query.iter_mut(world) {
        if body.blocking {
            let index = map.index(body.position());
            map.blocked[index] = true;
        }
    }
    let mut player_query = <(&Player, &Body)>::query();
    let (_, player_body) = player_query.iter(world).next().unwrap();
    player_info.position = player_body.position();
}

#[system(for_each)]
#[write_component(Body)]
pub fn move_actions(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    move_action: &MoveAction,
    #[resource] map: &Map,
    entity: &Entity,
) {
    let mut query = <&mut Body>::query();

    let body = query.get_mut(world, move_action.entity);
    if let Ok(body) = body {
        let new_position = (body.x + move_action.dx, body.y + move_action.dy);
        if !map.is_blocked(new_position) {
            body.set_position(new_position);
        }
    }

    cmd.remove(entity.clone());
}
