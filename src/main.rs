use bevy::{
    input::mouse::MouseButtonInput,
    prelude::*,
    window::CursorMoved,
};
use std::ops::{Add, Mul, Sub};

#[derive(Component)]
struct Phase;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct MobOrb {
    velocity: Vec3,
}

#[derive(Component)]
struct MobCrab;

#[derive(Component)]
struct Player;

struct ListPhasesTimer(Timer);

fn add_phases(mut commands: Commands) {
    commands.spawn().insert(Phase).insert(Name("First".to_string()));
    commands.spawn().insert(Phase).insert(Name("Second".to_string()));
    commands.spawn().insert(Phase).insert(Name("Third".to_string()));
}

fn list_phases(time: Res<Time>, mut timer: ResMut<ListPhasesTimer>, query: Query<&Name, With<Phase>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in query.iter() {
            println!("a phase {}!", name.0);
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.3, 0.0, 0.3),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        transform: Transform::from_xyz(50.0, 300.0, 0.0),
        ..default()
    }).insert(Player);

    let crab_positions = vec![
        Vec3::new(20.0, 20.0, 0.0),
        Vec3::new(120.0, 20.0, 0.0),
        Vec3::new(220.0, 20.0, 0.0),
        Vec3::new(20.0, 120.0, 0.0),
        Vec3::new(20.0, 220.0, 0.0),
    ];

    for crab_pos in crab_positions {
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            transform: Transform::from_translation(crab_pos),
            ..default()
        }).insert(MobCrab);
    }
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.9, 1.0, 1.0),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        ..default()
    }).insert(MobOrb { velocity: Vec3::new(10.0, 10.0, 0.0) });
}


fn move_orb(time: Res<Time>, mut state: Query<(&MobOrb, &mut Transform)>) {
    for (orb, mut transform) in &mut state {
        transform.translation = transform.translation.add(orb.velocity.mul(time.delta_seconds()))
    }

}

fn move_crabs(time: Res<Time>,
              mut crabs: Query<&mut Transform, With<MobCrab>>,
              orb: Query<(&MobOrb, &Transform), Without<MobCrab>>) {
    for mut transform in &mut crabs {
        let (_, orb_transform) = orb.single();
        let vel = orb_transform.translation.sub(transform.translation);
        transform.translation = transform.translation.add(
            vel.mul(25.0 / vel.length()).mul(time.delta_seconds()));
    }
}

fn move_player(time: Res<Time>, keyboard_input: Res<Input<KeyCode>>,
               mut transforms: Query<&mut Transform, With<Player>>) {
    let speed = 50.0 * time.delta_seconds();
    for mut transform in &mut transforms {
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += speed;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= speed;
        }
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= speed;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += speed;
        }
    }
}

fn print_mouse_events_system(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    for event in mouse_button_input_events.iter() {
        info!("{:?}", event);
    }

    for event in cursor_moved_events.iter() {
        info!("{:?}", event);
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 640.0,
            height: 640.0,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(ListPhasesTimer(Timer::from_seconds(2.0, true)))
        .add_startup_system(add_phases)
        .add_startup_system(setup)
        .add_system(print_mouse_events_system)
        .add_system(list_phases)
        .add_system(move_player)
        .add_system(move_crabs)
        .add_system(move_orb)
        .run();
}
