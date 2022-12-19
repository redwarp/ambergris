use bevy::prelude::*;

use crate::{
    actions::{self, MoveAction},
    graphics::MapCamera,
    map::Position,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(camera_follow.after(actions::handle_move_actions))
            .add_system(player_movement.before(actions::handle_move_actions));
    }
}

#[derive(Component)]
pub struct Player;

#[allow(clippy::type_complexity)]
fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera2d>, With<MapCamera>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn player_movement(
    player_query: Query<(Entity, &Position), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    mut move_actions: EventWriter<MoveAction>,
) {
    let move_dx = |player_query: &Query<(Entity, &Position), With<Player>>,
                   move_actions: &mut EventWriter<MoveAction>,
                   dx: i32,
                   dy: i32| {
        let (entity, position) = player_query.single();
        let mut target_position = *position;
        target_position.x += dx;
        target_position.y += dy;

        move_actions.send(MoveAction {
            entity,
            target_position,
        });
    };

    if keyboard.any_just_pressed([KeyCode::Up, KeyCode::W]) {
        move_dx(&player_query, &mut move_actions, 0, -1);
    } else if keyboard.any_just_pressed([KeyCode::Down, KeyCode::S]) {
        move_dx(&player_query, &mut move_actions, 0, 1);
    } else if keyboard.any_just_pressed([KeyCode::Left, KeyCode::A]) {
        move_dx(&player_query, &mut move_actions, -1, 0);
    } else if keyboard.any_just_pressed([KeyCode::Right, KeyCode::D]) {
        move_dx(&player_query, &mut move_actions, 1, 0);
    }
}
