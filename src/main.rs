use bevy::{
    prelude::*,
    render::color::Color,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, collide_aabb},
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
struct VoidZone(f32);

const VOID_ZONE_GROWTH_DURATION_SECS: f32 = 4.;
const VOID_ZONE_START_RADIUS: f32 = 30.;
const VOID_ZONE_GROWTH_AMOUNT: f32 = 10.;
const VOID_ZONE_CRAB_SPAWN_DURATION_SECS: f32 = 10.;

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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    StartMenu,
    PurificationOne,
    Failure,
    Success,
}

#[derive(Component)]
struct ButtonNextState(GameState);

const CRAB_SIZE: f32 = 30.;
const CRAB_SPEED: f32 = 15.;
const BULLET_SIZE: f32 = 10.;
const ORB_RADIUS: f32 = 50.;
const ORB_TARGET_RADIUS: f32 = 70.;
const ORB_VELOCITY_DECAY: f32 = 0.5;

const LAYER_PLAYER: f32 = 100.;
const LAYER_CURSOR: f32 = LAYER_PLAYER - 5.;
const LAYER_MOB: f32 = 20.;
const LAYER_TARGET: f32 = 10.;
const LAYER_MAP: f32 = 0.;
const LAYER_VOID: f32 = 0.5;
const LAYER_UI: f32 = 1.;
const LAYER_TEXT: f32 = 2.;

const WIDTH: f32 = 1024.;
const HEIGHT: f32 = 1024.;
const PX_TO_GAME: f32 = 2849. / WIDTH;
const GAME_TO_PX: f32 = 1. / PX_TO_GAME;

const MAP_RADIUS: f32 = WIDTH / 2.;


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
    void_zone_growth: Timer,
    void_zone_crab_spawn: Timer,
}
struct MenuData {
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup_menu_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(350.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Purification One",
                TextStyle {
                    font: asset_server.load("trebuchet_ms.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(ButtonNextState(GameState::PurificationOne))
        .id();
    commands.insert_resource(MenuData { button_entity });
}

fn update_menu_system(
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &ButtonNextState),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, next_state) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(next_state.0).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu_system(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

fn setup_success_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(WIDTH), Val::Px(100.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.2).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Phase cleared!",
                TextStyle {
                    font: asset_server.load("trebuchet_ms.ttf"),
                    font_size: 80.,
                    color: Color::rgb(0.3, 1.0, 0.3),
                },
            ));
        });
}

fn setup_failure_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(WIDTH), Val::Px(100.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.2).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "You died :(",
                TextStyle {
                    font: asset_server.load("trebuchet_ms.ttf"),
                    font_size: 80.,
                    color: Color::rgb(0.9, 0.2, 0.2),
                },
            ));
        });
}

fn spawn_crab(commands: &mut Commands, crab_pos: Vec3) {
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

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_purification_one(
    mut commands: Commands, asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {

    let mut game = Game {
        player: Player {
            ..default()
        },
        cursor: None,
        orb_target: 0,
        void_zone_growth: Timer::from_seconds(VOID_ZONE_GROWTH_DURATION_SECS, true),
        void_zone_crab_spawn: Timer::from_seconds(VOID_ZONE_CRAB_SPAWN_DURATION_SECS, true),
    };

    game.orb_target = 0;
    game.void_zone_growth.reset();
    game.void_zone_crab_spawn.reset();

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

    commands.insert_resource(game);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("map.png"),
        transform: Transform::from_xyz(0., 0., LAYER_MAP),
        ..default()
    });

    let crab_positions = vec![
        Vec3::new(-350., 200., LAYER_MOB),
        Vec3::new(-312.5, 237.5, LAYER_MOB),
        Vec3::new(-275., 275., LAYER_MOB),
        Vec3::new(-237.5, 312.5, LAYER_MOB),
        Vec3::new(-200., 350., LAYER_MOB),
    ];

    for crab_pos in crab_positions {
        spawn_crab(&mut commands, crab_pos);
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

    let void_zone_positions = [
        Vec3::new(-WIDTH / 2. + 100., 0., LAYER_VOID),
        Vec3::new(WIDTH / 2. - 100., 0., LAYER_VOID),
        Vec3::new(0., -HEIGHT / 2. + 100., LAYER_VOID),
        Vec3::new(0., HEIGHT / 2. - 100., LAYER_VOID),
    ];

    let void_zone_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(VOID_ZONE_START_RADIUS).into()).into();
    let void_zone_material = ColorMaterial::from(Color::rgba(0.0, 0.0, 0.0, 0.9));

    for pos in void_zone_positions {
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: void_zone_mesh.clone(),
            material: materials.add(void_zone_material.clone()),
            transform: Transform::from_translation(pos),
            ..default()
        }).insert(VoidZone(VOID_ZONE_START_RADIUS));
    }

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 64.,
        color: Color::rgb(0.1, 0.7, 0.1),
    };

    let text_binding_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 28.,
        color: Color::rgb(0.4, 0.2, 0.),
    };
    let binding_y = 18.;

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
        text: Text::from_section("", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(-128., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownPull,
        sprite: Some(sprite_pull),
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("4", text_binding_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(-128., -HEIGHT / 2. + binding_y, LAYER_TEXT),
        ..default()
    });

    let sprite_blink = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("blink.png"),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownBlink,
        sprite: Some(sprite_blink),
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("E", text_binding_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + binding_y, LAYER_TEXT),
        ..default()
    });

    let sprite_portal = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("portal.png"),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownPortal,
        sprite: Some(sprite_portal),
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("R", text_binding_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + binding_y, LAYER_TEXT),
        ..default()
    });
}


fn move_crabs_system(time: Res<Time>,
              mut crabs: Query<&mut Transform, (With<MobCrab>, Without<EffectForcedMarch>)>,
              orb: Query<(&MobOrb, &Transform), Without<MobCrab>>) {
    for mut transform in &mut crabs {
        let (_, orb_transform) = orb.single();
        let vel = orb_transform.translation.sub(transform.translation);
        transform.translation = transform.translation.add(
            vel.mul(CRAB_SPEED / vel.length()).mul(time.delta_seconds()));
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

fn collide(pos_a: Vec3, radius_a: f32, pos_b: Vec3, radius_b: f32) -> bool {
    let mut diff = pos_b.sub(pos_a);
    diff.z = 0.;
    return diff.length_squared() < (radius_a + radius_b) * (radius_a + radius_b);
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
        let bullet_pos = transform_bullet.translation;
        for (entity_crab, transform_crab) in &crabs {
            let crab_pos = transform_crab.translation;
            if let Some(_) = collide_aabb::collide(bullet_pos, Vec2::splat(BULLET_SIZE), crab_pos, Vec2::splat(CRAB_SIZE)) {
                // commands.entity(entity_bullet).despawn_recursive(); allows cleave
                commands.entity(entity_crab).despawn_recursive();
            }
        }

        if bullet_pos.x < -WIDTH / 2. - BULLET_SIZE ||
           bullet_pos.x > WIDTH / 2. - BULLET_SIZE ||
           bullet_pos.y < -HEIGHT / 2. - BULLET_SIZE ||
           bullet_pos.y > HEIGHT / 2. - BULLET_SIZE {
            commands.entity(entity_bullet).despawn_recursive();
        }
    }


    for (entity_bullet, transform_bullet) in &bullets {
        let bullet_pos = transform_bullet.translation;
        for (transform_orb, mut velocity_orb) in &mut orbs {
            let orb_pos = transform_orb.translation;
            if collide(bullet_pos, BULLET_SIZE / 2., orb_pos, ORB_RADIUS) {
                commands.entity(entity_bullet).despawn_recursive();
                let transform_player = players.single();
                let push_str = 4.;
                let orb_max_vel = 60.;
                let mut diff = orb_pos.sub(transform_player.translation);
                diff.z = 0.;
                velocity_orb.0 = velocity_orb.0.add(diff.clamp_length(push_str, push_str)).clamp_length(0., orb_max_vel);
            }
        }
    }

    for (transform_orb, _) in &orbs {
        let orb_pos = transform_orb.translation;
        for (_, transform_crab) in &crabs {
            let crab_pos = transform_crab.translation;
            if collide(orb_pos, ORB_RADIUS, crab_pos, CRAB_SIZE / 2.) {
                game.player.hp = 0;
            }
        }
    }
}

fn collisions_orb_targets_system(
    mut game: ResMut<Game>,
    mut state: ResMut<State<GameState>>,
    mut orbs: Query<(&MobOrb, &Transform, &mut Velocity)>,
    orb_targets: Query<(&OrbTarget, &Transform)>,
    ) {
    let mut any_orb_targetted = false;
    for (_, transform_orb, mut velocity_orb) in &mut orbs {
        let mut orb_pos = transform_orb.translation;
        orb_pos.z = 0.;
        for (orb_target, transform_orb_target) in &orb_targets {
            if game.orb_target != orb_target.0 {
                continue;
            }
            any_orb_targetted = true;
            let mut orb_target_pos = transform_orb_target.translation;
            orb_target_pos.z = 0.;
            if collide(orb_pos, ORB_RADIUS, orb_target_pos, ORB_TARGET_RADIUS) {
                game.orb_target += 1;
                velocity_orb.0 = velocity_orb.0 * ORB_VELOCITY_DECAY;
            }
        }
    }

    if !any_orb_targetted {
        info!("win detected!");
        state.set(GameState::Success).unwrap();
    }
}

fn collisions_edge_system(
    mut game: ResMut<Game>,
    orbs: Query<(&MobOrb, &Transform)>,
    players: Query<&Transform, (With<PlayerTag>, Without<MobOrb>)>,
    ) {
    let transform_player = players.single();
    if !collide(transform_player.translation, 0., Vec3::ZERO, MAP_RADIUS) {
        game.player.hp = 0;
    }

    for (_, transform_orb) in &orbs {
        if !collide(transform_orb.translation, 0., Vec3::ZERO, MAP_RADIUS - ORB_RADIUS) {
            game.player.hp = 0;
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
        let bullet_speed = 200.0;
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
            Color::rgba(0.7, 0.7, 0.7, 0.7)
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

fn void_zone_crab_system(
    time: Res<Time>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    void_zones: Query<(&VoidZone, &Transform)>
    ) {
    game.void_zone_crab_spawn.tick(time.delta());

    if !game.void_zone_crab_spawn.just_finished() {
        return;
    }

    for (_, transform) in &void_zones {
        let mut pos = transform.translation.clone();
        pos.z = LAYER_MOB;

        spawn_crab(&mut commands, pos);
    }
}

fn void_zone_growth_system(time: Res<Time>,
                        mut game: ResMut<Game>,
                    mut void_zones: Query<(&mut VoidZone, &mut Transform)>
                    ) {
    game.void_zone_growth.tick(time.delta());

    let growing = game.void_zone_growth.just_finished();

    for (mut void_zone, mut transform) in &mut void_zones {
        if growing {
            void_zone.0 += VOID_ZONE_GROWTH_AMOUNT;
            let new_scale = void_zone.0 / VOID_ZONE_START_RADIUS;
            transform.scale.x = new_scale;
            transform.scale.y = new_scale;
        }
    }
}

fn player_hp_check_system(game: ResMut<Game>,
                          mut state: ResMut<State<GameState>>,
                          ) {
    if game.player.hp <= 0 {
        state.set(GameState::Failure).unwrap();
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            scale_factor_override: Some(1.),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_state(GameState::StartMenu)

        .add_startup_system(setup)

        .add_system_set(SystemSet::on_enter(GameState::StartMenu).with_system(setup_menu_system))
        .add_system_set(SystemSet::on_update(GameState::StartMenu).with_system(update_menu_system))
        .add_system_set(SystemSet::on_exit(GameState::StartMenu).with_system(cleanup_menu_system))

        .add_system_set(SystemSet::on_enter(GameState::PurificationOne).with_system(setup_purification_one))
        .add_system_set(SystemSet::on_update(GameState::PurificationOne)
            .with_system(handle_mouse_events_system)
            .with_system(handle_spellcasts_system)
            .with_system(velocities_system)
            .with_system(move_player_system)
            .with_system(move_crabs_system)
            .with_system(effect_forced_march_system)
            .with_system(collisions_system)
            .with_system(collisions_orb_targets_system)
            .with_system(collisions_edge_system)
            .with_system(game_orb_target_progression_system)
            .with_system(text_system)
            .with_system(void_zone_growth_system)
            .with_system(void_zone_crab_system)
            .with_system(player_hp_check_system))

        .add_system_set(SystemSet::on_enter(GameState::Success).with_system(setup_success_system))

        .add_system_set(SystemSet::on_enter(GameState::Failure).with_system(setup_failure_system))

        .add_system(bevy::window::close_on_esc)
        .run();
}
