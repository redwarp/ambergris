use bevy::prelude::*;
use torchbearer::path::PathMap;

use crate::{
    graphics::TILE_SIZE,
    map::{MapInfo, Position, Solid},
};

pub struct MoveAction {
    pub entity: Entity,
    pub target_position: Position,
}

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<MoveAction>()
            .add_system(handle_move_actions);
    }
}

pub fn handle_move_actions(
    mut move_actions: EventReader<MoveAction>,
    mut query: Query<(&mut Position, &mut Transform, Option<&Solid>)>,
    mut map_info: ResMut<MapInfo>,
) {
    for move_action in move_actions.iter() {
        let (mut position, mut transform, solid) = query.get_mut(move_action.entity).unwrap();

        if map_info.is_walkable((move_action.target_position.x, move_action.target_position.y)) {
            if solid.is_some() {
                map_info.set_blocked(&position, false);
                map_info.set_blocked(&move_action.target_position, true);
            }

            *position = move_action.target_position;

            transform.translation.x = position.x as f32 * TILE_SIZE;
            transform.translation.y = -position.y as f32 * TILE_SIZE;
        }
    }
}
