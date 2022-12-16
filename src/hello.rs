use bevy::prelude::*;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Named(String);

#[derive(Resource)]
struct GreetTimer(Timer);

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_startup_system(add_people)
            .add_system(greet_people);
    }
}

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Named("Alice".to_string())));
    commands.spawn((Person, Named("Bob".to_string())));
}

fn greet_people(
    time: Res<Time>,
    mut timer: ResMut<GreetTimer>,
    query: Query<&Named, With<Person>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for named in query.iter() {
            println!("Hello {}!", named.0);
        }
    }
}
