use bevy::{
    prelude::*,
    sprite::collide_aabb::collide,
    render::color::Color,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::CursorMoved,
};
use std::time::Duration;
use std::ops::{Add, Mul, Sub};

#[derive(Component)]
struct Name(String);

enum TextValue {
    Hp,
    CooldownDodge,
    CooldownBlink,
    CooldownPortal,
    CooldownPull,
}

#[derive(Component)]
struct TextDisplay {
    value: TextValue,
    sprite: Option<Entity>,
}

#[derive(Component)]
struct MobOrb;

#[derive(Component)]
struct MobCrab;

#[derive(Component)]
struct OrbTarget(i32);

const ORB_TARGET_COLOR_BASE: Color = Color::rgb(0.5, 0.5, 0.5);
const ORB_TARGET_COLOR_ACTIVE: Color = Color::rgb(0.7, 1., 0.7);

#[derive(Component)]
struct CursorMark;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct PlayerTag;

#[derive(Component)]
struct EffectForcedMarch {
    target: Vec3,
    speed: f32,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    StartMenu,
    PurificationOne,
    Failure,
}

const CRAB_SIZE: f32 = 10.;
const BULLET_SIZE: f32 = 5.;
const ORB_RADIUS: f32 = 50.;
const ORB_TARGET_RADIUS: f32 = 70.;

const LAYER_PLAYER: f32 = 100.;
const LAYER_CURSOR: f32 = LAYER_PLAYER - 5.;
const LAYER_MOB: f32 = 20.;
const LAYER_TARGET: f32 = 10.;
const LAYER_MAP: f32 = 0.;
const LAYER_UI: f32 = 1.;
const LAYER_TEXT: f32 = 2.;

const WIDTH: f32 = 1024.;
const HEIGHT: f32 = 1024.;
const PX_TO_GAME: f32 = 2849. / WIDTH;
const GAME_TO_PX: f32 = 1. / PX_TO_GAME;


#[derive(Default)]
struct Player {
    hp: i32,
    shoot_cooldown: Timer,
    dodge_cooldown: Timer,
    blink_cooldown: Timer,
    portal_cooldown: Timer,
    pull_cooldown: Timer,
    entity: Option<Entity>,
}

struct Game {
    player: Player,
    cursor: Option<Entity>,
    orb_target: i32,
}

fn setup(mut commands: Commands, mut game: ResMut<Game>, asset_server: Res<AssetServer>,
         mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, ) {
    commands.spawn_bundle(Camera2dBundle::default());

    game.player.hp = 100;
    game.player.shoot_cooldown = Timer::from_seconds(0.2, false);
    game.player.dodge_cooldown = Timer::from_seconds(10., false);
    game.player.blink_cooldown = Timer::from_seconds(16., false);
    game.player.portal_cooldown = Timer::from_seconds(60., false);
    game.player.pull_cooldown = Timer::from_seconds(20., false);

    game.player.dodge_cooldown.tick(Duration::from_secs_f32(1000.));
    game.player.blink_cooldown.tick(Duration::from_secs_f32(1000.));
    game.player.portal_cooldown.tick(Duration::from_secs_f32(1000.));
    game.player.pull_cooldown.tick(Duration::from_secs_f32(1000.));

    game.player.entity = Some(
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0., 0.3),
                custom_size: Some(Vec2::new(20., 20.)),
                ..default()
            },
            transform: Transform::from_xyz(50., 300., LAYER_PLAYER),
            ..default()
        }).insert(PlayerTag).id()
    );

    game.cursor = Some(
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.9, 0., 0.),
                custom_size: Some(Vec2::new(4., 4.)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., LAYER_CURSOR),
            ..default()
        }).insert(CursorMark).id()
    );

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("map.png"),
        transform: Transform::from_xyz(0., 0., LAYER_MAP),
        ..default()
    });

    let crab_positions = vec![
        Vec3::new(20.0, 20.0, LAYER_MOB),
        Vec3::new(120.0, 20.0, LAYER_MOB),
        Vec3::new(220.0, 20.0, LAYER_MOB),
        Vec3::new(20.0, 120.0, LAYER_MOB),
        Vec3::new(20.0, 220.0, LAYER_MOB),
    ];

    for crab_pos in crab_positions {
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(CRAB_SIZE, CRAB_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(crab_pos),
            ..default()
        }).insert(MobCrab);
    }
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(ORB_RADIUS).into()).into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.9, 1.0, 1.0))),
        transform: Transform::from_xyz(0., 0., LAYER_MOB),
        ..default()
    }).insert(MobOrb).insert(Velocity(Vec3::new(0., 0., 0.)));

    let orb_target_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(ORB_TARGET_RADIUS).into()).into();
    let orb_target_material = ColorMaterial::from(Color::rgb(0.5, 0.5, 0.5));

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: orb_target_mesh.clone(),
        material: materials.add(orb_target_material.clone()),
        transform: Transform::from_xyz(-240., 240., LAYER_TARGET),
        ..default()
    }).insert(OrbTarget(0));

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: orb_target_mesh.clone(),
        material: materials.add(orb_target_material.clone()),
        transform: Transform::from_xyz(-240., -240., LAYER_TARGET),
        ..default()
    }).insert(OrbTarget(1));

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: orb_target_mesh.clone(),
        material: materials.add(orb_target_material.clone()),
        transform: Transform::from_xyz(240., -240., LAYER_TARGET),
        ..default()
    }).insert(OrbTarget(2));

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 64.,
        color: Color::rgb(0.1, 0.7, 0.1),
    };

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("hp", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::Hp,
        sprite: None,
    });

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.6, 0.1, 0.1))),
        transform: Transform::from_xyz(0., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("0", 
            TextStyle {
                font: asset_server.load("trebuchet_ms.ttf"),
                font_size: 80.,
                color: Color::rgb(0.7, 0.7, 0.1),
            })
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., -HEIGHT / 2. + 155., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownDodge,
        sprite: None,
    });

    let sprite_pull = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("pull.png"),
        transform: Transform::from_xyz(-128., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("hp", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(-128., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownPull,
        sprite: Some(sprite_pull),
    });

    let sprite_blink = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("blink.png"),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("hp", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownBlink,
        sprite: Some(sprite_blink),
    });

    let sprite_portal = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("portal.png"),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("hp", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownPortal,
        sprite: Some(sprite_portal),
    });
}


fn move_crabs_system(time: Res<Time>,
              mut crabs: Query<&mut Transform, (With<MobCrab>, Without<EffectForcedMarch>)>,
              orb: Query<(&MobOrb, &Transform), Without<MobCrab>>) {
    for mut transform in &mut crabs {
        let (_, orb_transform) = orb.single();
        let vel = orb_transform.translation.sub(transform.translation);
        transform.translation = transform.translation.add(
            vel.mul(25.0 / vel.length()).mul(time.delta_seconds()));
    }
}

fn move_player_system(time: Res<Time>, keyboard_input: Res<Input<KeyCode>>,
               mut transforms: Query<&mut Transform, (With<PlayerTag>, Without<EffectForcedMarch>)>) {
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

fn effect_forced_march_system(time: Res<Time>, mut commands: Commands, mut pulleds: Query<(Entity, &mut Transform, &EffectForcedMarch)>) {
    for (ent, mut transform, effect) in &mut pulleds {
        let target = effect.target;
        let mut diff = target.sub(transform.translation);
        diff.z = 0.;
        let speed = effect.speed * time.delta_seconds();
        let vel = diff.clamp_length(speed, speed);
        if vel.length_squared() > diff.length_squared() {
            transform.translation = target;
            commands.entity(ent).remove::<EffectForcedMarch>();
        } else {
            transform.translation = transform.translation.add(vel);
        }
    }
}

fn velocities_system(
    time: Res<Time>,
    mut tra_vels: Query<(&mut Transform, &Velocity)>
    ) {
    for (mut transform, velocity) in &mut tra_vels {
        transform.translation = transform.translation.add(velocity.0.mul(time.delta_seconds()));
    }
}

fn collisions_system(
    mut game: ResMut<Game>,
    mut commands: Commands,
    players: Query<&Transform, (With<PlayerTag>, Without<MobOrb>)>,
    bullets: Query<(Entity, &Transform), (With<Bullet>, Without<MobOrb>)>,
    crabs: Query<(Entity, &Transform), With<MobCrab>>,
    mut orbs: Query<(&Transform, &mut Velocity), (With<MobOrb>, Without<Bullet>)>,
    ) {
    for (entity_bullet, transform_bullet) in &bullets {
        let bullet_loc = transform_bullet.translation;
        for (entity_crab, transform_crab) in &crabs {
            let crab_loc = transform_crab.translation;
            if let Some(_) = collide(bullet_loc, Vec2::splat(BULLET_SIZE), crab_loc, Vec2::splat(CRAB_SIZE)) {
                commands.entity(entity_bullet).despawn_recursive();
                commands.entity(entity_crab).despawn_recursive();
            }
        }
    }

    for (entity_bullet, transform_bullet) in &bullets {
        let bullet_loc = transform_bullet.translation;
        for (transform_orb, mut velocity_orb) in &mut orbs {
            let orb_loc = transform_orb.translation;
            if let Some(_) = collide(bullet_loc, Vec2::splat(BULLET_SIZE), orb_loc, Vec2::splat(ORB_RADIUS * 2.)) {
                commands.entity(entity_bullet).despawn_recursive();
                let transform_player = players.single();
                let push_str = 4.0;
                let orb_max_vel = 30.0;
                let mut diff = orb_loc.sub(transform_player.translation);
                diff.z = 0.;
                velocity_orb.0 = velocity_orb.0.add(diff.clamp_length(push_str, push_str)).clamp_length(orb_max_vel, orb_max_vel);
            }
        }
    }

    for (transform_orb, _) in &orbs {
        let orb_loc = transform_orb.translation;
        for (_, transform_crab) in &crabs {
            let crab_loc = transform_crab.translation;
            if let Some(_) = collide(orb_loc, Vec2::splat(ORB_RADIUS * 2.), crab_loc, Vec2::splat(CRAB_SIZE)) {
                game.player.hp = -1;
            }
        }
    }
}

fn collisions_orb_targets_system(
    mut game: ResMut<Game>,
    orbs: Query<(&MobOrb, &Transform)>,
    orb_targets: Query<(&OrbTarget, &Transform)>,
    ) {
    for (_, transform_orb) in &orbs {
        let mut orb_loc = transform_orb.translation;
        orb_loc.z = 0.;
        for (orb_target, transform_orb_target) in &orb_targets {
            if game.orb_target != orb_target.0 {
                continue;
            }
            let mut orb_target_loc = transform_orb_target.translation;
            orb_target_loc.z = 0.;
            if let Some(_) = collide(orb_loc, Vec2::splat(ORB_RADIUS * 2.), orb_target_loc, Vec2::splat(ORB_TARGET_RADIUS * 2.)) {
                game.orb_target += 1;
            }
        }
    }
}

fn game_orb_target_progression_system(
    game: ResMut<Game>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    orb_targets: Query<(&OrbTarget, &mut Handle<ColorMaterial>)>,
    ) {
    for (orb_target, material) in &orb_targets {
        if orb_target.0 == game.orb_target {
            materials.get_mut(material).unwrap().color = ORB_TARGET_COLOR_ACTIVE;
        } else {
            materials.get_mut(material).unwrap().color = ORB_TARGET_COLOR_BASE;
        }
    }
}

fn handle_mouse_events_system(
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut transforms: Query<&mut Transform>,
    mut game: ResMut<Game>,
    time: Res<Time>) {

    let player_loc = {
        let transform = transforms.get_mut(game.player.entity.unwrap()).unwrap();
        transform.translation
    };

    game.player.shoot_cooldown.tick(time.delta());
    game.player.dodge_cooldown.tick(time.delta());
    game.player.pull_cooldown.tick(time.delta());
    game.player.blink_cooldown.tick(time.delta());
    game.player.portal_cooldown.tick(time.delta());

    if game.player.shoot_cooldown.finished() &&
       (mouse_button_input.pressed(MouseButton::Left) || 
        keyboard_input.pressed(KeyCode::Key1)) {
        let cursor = transforms.get_mut(game.cursor.unwrap()).unwrap();
        let mut vel = cursor.translation.sub(player_loc);
        vel.z = 0.;
        let bullet_speed = 100.0;
        vel = vel.clamp_length(bullet_speed, bullet_speed);

        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1., 0.7, 0.),
                custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(player_loc),
            ..default()
        }).insert(Velocity(vel))
          .insert(Bullet);
        game.player.shoot_cooldown.reset();
    }

    for event in cursor_moved_events.iter() {
        let mut cursor = transforms.get_mut(game.cursor.unwrap()).unwrap();
        // info!("{:?}", event);
        // let mut cursor_transform = cursors.single_mut();
        cursor.translation.x = event.position.x - WIDTH / 2.;
        cursor.translation.y = event.position.y - HEIGHT / 2.;
    }
}

fn handle_spellcasts_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    transforms: Query<&Transform>,
    crabs: Query<(Entity, &Transform, &MobCrab)>,
    mut game: ResMut<Game>) {

    let player_loc = {
        let transform = transforms.get(game.player.entity.unwrap()).unwrap();
        transform.translation
    };

    let cursor_loc = transforms.get(game.cursor.unwrap()).unwrap().translation;

    if game.player.dodge_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::V) {
        let dodge_range = 300. * GAME_TO_PX;
        let dodge_speed = dodge_range / 0.75;
        let diff = cursor_loc.sub(player_loc).clamp_length(dodge_range, dodge_range);
        let target = player_loc.add(diff);


        commands.entity(game.player.entity.unwrap()).insert(EffectForcedMarch {
            target,
            speed: dodge_speed,
        });

        game.player.dodge_cooldown.reset();
    }

    if game.player.blink_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::E) {
        let blink_range = 1200.0 * GAME_TO_PX;
        let blink_speed = blink_range / 0.1;
        let mut diff = cursor_loc.sub(player_loc);
        diff.z = 0.;
        diff = diff.clamp_length(0., blink_range);
        let target = player_loc.add(diff);


        commands.entity(game.player.entity.unwrap()).insert(EffectForcedMarch {
            target,
            speed: blink_speed,
        });

        game.player.blink_cooldown.reset();
    }

    if game.player.pull_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::Key4) {
        let pull_loc = cursor_loc;
        let pull_range = 600.0 * GAME_TO_PX;
        let pull_speed = pull_range / 0.3;

        for (entity_crab, transform_crab, _) in &crabs {
            let crab_loc = transform_crab.translation;
            let mut diff = crab_loc.sub(pull_loc);
            diff.z = 0.;
            if diff.length_squared() > pull_range * pull_range {
                continue;
            }

            let target = pull_loc;
            commands.entity(entity_crab).insert(EffectForcedMarch {
                target,
                speed: pull_speed,
            });
        }

        game.player.pull_cooldown.reset();
    }
}

fn set_cooldown_text_display(timer: &Timer, text: &mut Text, text_display: &TextDisplay, sprites: &mut Query<&mut Sprite>) {
    let dur = timer.duration().as_secs_f32();
    let elapsed = timer.elapsed_secs();
    let left = dur - elapsed;

    if left < 5. {
        text.sections[0].value = format!("{left:.1}");
    } else {
        text.sections[0].value = format!("{left:.0}");
    }

    if left < 0.001 {
        text.sections[0].style.color.set_a(0.0);
    } else {
        text.sections[0].style.color.set_a(1.0);
    }

    if let Some(sprite_handle) = text_display.sprite {
        let color = if left < 0.001 {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.1, 0.1, 0.1, 0.7)
        };
        sprites.get_mut(sprite_handle).unwrap().color = color;
    }
}

fn text_system(game: Res<Game>,
               mut text_displays: Query<(&mut Text, &TextDisplay)>,
               mut sprites: Query<&mut Sprite>) {

    for (mut text, text_display) in &mut text_displays {
        match text_display.value {
            TextValue::Hp => {
                let hp = game.player.hp;
                text.sections[0].value = format!("{hp:.0}");
            },
            TextValue::CooldownBlink => {
                set_cooldown_text_display(&game.player.blink_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownDodge => {
                set_cooldown_text_display(&game.player.dodge_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownPortal => {
                set_cooldown_text_display(&game.player.portal_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownPull => {
                set_cooldown_text_display(&game.player.pull_cooldown, &mut text, &text_display, &mut sprites);
            }
        }
    }

    // set_cooldown_text(&game.player.pull_cooldown, &mut set.p2().get_single_mut().unwrap());
    // set_cooldown_text(&game.player.portal_cooldown, &mut set.p3().get_single_mut().unwrap());
    // set_cooldown_text(&game.player.blink_cooldown, &mut set.p4().get_single_mut().unwrap());
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            scale_factor_override: Some(1.),
            ..default()
        })
        .insert_resource(Game {
            player: Player {
                ..default()
            },
            cursor: None,
            orb_target: 0,
        })
        .add_plugins(DefaultPlugins)
        .add_state(GameState::PurificationOne)
        .add_startup_system(setup)
        .add_system(handle_mouse_events_system)
        .add_system(handle_spellcasts_system)
        .add_system(velocities_system)
        .add_system(move_player_system)
        .add_system(move_crabs_system)
        .add_system(effect_forced_march_system)
        .add_system(collisions_system)
        .add_system(collisions_orb_targets_system)
        .add_system(game_orb_target_progression_system)
        .add_system(text_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}
