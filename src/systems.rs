use crate::components::*;
use crate::map::Map;
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

#[system(for_each)]
#[write_component(Body)]
pub fn move_actions(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    move_action: &MoveAction,
    entity: &Entity,
) {
    let mut query = <&mut Body>::query();

    let body = query.get_mut(world, move_action.entity);
    if let Ok(body) = body {
        body.x = body.x + move_action.dx;
        body.y = body.y + move_action.dy;
    }

    cmd.remove(entity.clone())
}
