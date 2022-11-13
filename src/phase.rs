use bevy::{
    prelude::*,
    render::color::Color,
    sprite::MaterialMesh2dBundle,
    window::CursorMoved,
};

use std::collections::HashSet;
use std::ops::{Add, Mul, Sub};
use std::time::Duration;

use crate::game::*;
use crate::aoes::*;
use crate::hints::{setup_hints, scheduled_hint_system};
use crate::ui::*;
use crate::damage_flash::*;
use crate::collisions::*;
use crate::mobs::*;

pub const VOID_ZONE_GROWTH_DURATION_SECS: f32 = 4.;
pub const VOID_ZONE_START_RADIUS: f32 = 30.;
pub const VOID_ZONE_GROWTH_AMOUNT: f32 = 252. / 14.;
pub const VOID_ZONE_CRAB_SPAWN_DURATION_SECS: f32 = 10.;

pub const PORTAL_RADIUS: f32 = 24.;

#[derive(Component)]
pub struct RotatingSoup {
    pub radius: f32,
    pub theta: f32,
    pub dtheta: f32,
}

#[derive(Component)]
pub struct VoidZone;

#[derive(Component)]
pub struct VoidZoneGrowth(pub Timer);

#[derive(Component)]
pub struct VoidZoneCrabSpawn(pub Timer);

#[derive(Component)]
pub struct PortalEntry(Timer);

#[derive(Component)]
pub struct PortalExit(Timer);

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct EffectForcedMarch {
    pub target: Vec3,
    pub speed: f32,
}

fn velocities_system(
    time: Res<Time>,
    mut commands: Commands,
    mut tra_vels: Query<(Entity, &mut Transform, &Velocity)>
    ) {
    for (entity, mut transform, velocity) in &mut tra_vels {
        transform.translation = transform.translation.add(velocity.0.mul(time.delta_seconds()));

        let pos = transform.translation;

        if pos.x < -WIDTH ||
           pos.x > WIDTH ||
           pos.y < -HEIGHT ||
           pos.y > HEIGHT {
            commands.entity(entity).despawn_recursive();
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

fn handle_mouse_events_system(
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut cursor_moved_events: EventReader<CursorMoved>,
    players: Query<&Transform, (With<PlayerTag>, Without<CursorMark>)>,
    mut cursors: Query<&mut Transform, With<CursorMark>>,
    mut game: ResMut<Game>,
    time: Res<Time>) {

    let player_loc = {
        let transform = players.single();
        transform.translation
    };

    game.time_elapsed.tick(time.delta());
    game.player.shoot_cooldown.tick(time.delta());
    game.player.dodge_cooldown.tick(time.delta());
    game.player.pull_cooldown.tick(time.delta());
    game.player.blink_cooldown.tick(time.delta());
    game.player.portal_cooldown.tick(time.delta());
    game.player.jump_cooldown.tick(time.delta());
    game.player.invuln.tick(time.delta());
    game.player.jump.tick(time.delta());
    game.player.hp += time.delta_seconds() * PLAYER_REGEN;
    if game.player.hp > 100. {
        game.player.hp = 100.;
    }

    if game.player.shoot_cooldown.finished() &&
       (mouse_button_input.pressed(MouseButton::Left) ||
        keyboard_input.pressed(KeyCode::Key1)) {
        let cursor = cursors.single();
        let mut vel = cursor.translation.sub(player_loc);
        vel.z = 0.;
        vel = vel.clamp_length(BULLET_SPEED, BULLET_SPEED);

        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.89, 0.39, 0.95),
                custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(player_loc),
            ..default()
        }).insert(Velocity(vel))
          .insert(Bullet(0.))
          .insert(HasHit(HashSet::new()));
        game.player.shoot_cooldown.reset();
    }

    // {
    //     let cursor = cursors.single();
    //     // info!("{:?}", event);
    //     if mouse_button_input.just_pressed(MouseButton::Left) {
    //         info!("{:?}", cursor.translation);
    //     }
    // }

    for event in cursor_moved_events.iter() {
        let mut cursor = cursors.single_mut();
        cursor.translation.x = event.position.x - WIDTH / 2.;
        cursor.translation.y = event.position.y - HEIGHT / 2.;
    }
}

fn handle_spellcasts_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    players: Query<&Transform, (With<PlayerTag>, Without<CursorMark>)>,
    portal_entries: Query<&Transform, With<PortalEntry>>,
    portal_exits: Query<&Transform, With<PortalExit>>,
    cursors: Query<&Transform, With<CursorMark>>,
    crabs: Query<(Entity, &Transform, &MobCrab)>,
    mut game: ResMut<Game>) {

    let player_loc = {
        let transform = players.single();
        transform.translation
    };

    let cursor_loc = cursors.single().translation;

    if game.player.jump_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::Space) {

        game.player.jump = Timer::from_seconds(0.5, TimerMode::Once);
        game.player.jump_cooldown.reset();
    }

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

        game.player.invuln = Timer::from_seconds(0.75, TimerMode::Once);
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

        game.player.invuln = Timer::from_seconds(0.1, TimerMode::Once);
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

    if game.player.portal_cooldown.finished() &&
        keyboard_input.just_pressed(KeyCode::R) {
        let portal_loc = player_loc;

        if portal_entries.is_empty() {
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0., 1., 1.),
                    custom_size: Some(Vec2::new(PORTAL_RADIUS * 2., PORTAL_RADIUS * 2.)),
                    ..default()
                },
                texture: asset_server.load("ring.png"),
                transform: Transform::from_translation(portal_loc),
                ..default()
            })
            .insert(PortalEntry(Timer::from_seconds(60., TimerMode::Once)));

            game.player.portal_cooldown = Timer::from_seconds(0.5, TimerMode::Once);
        } else if portal_exits.is_empty() {
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1., 0.7, 0.),
                    custom_size: Some(Vec2::new(PORTAL_RADIUS * 2., PORTAL_RADIUS * 2.)),
                    ..default()
                },
                texture: asset_server.load("ring.png"),
                transform: Transform::from_translation(portal_loc),
                ..default()
            })
            .insert(PortalExit(Timer::from_seconds(10., TimerMode::Once)));

            game.player.portal_cooldown = Timer::from_seconds(60., TimerMode::Once);
        }
    }

    if keyboard_input.just_pressed(KeyCode::F) &&
        !portal_entries.is_empty() &&
        !portal_exits.is_empty() {
        let entry = portal_entries.single().translation;
        let exit = portal_exits.single().translation;
        if collide(player_loc, PLAYER_RADIUS, entry, PORTAL_RADIUS) {
            commands.entity(game.player.entity.unwrap()).insert(EffectForcedMarch {
                target: exit,
                speed: 20000.,
            });
        } else if collide(player_loc, PLAYER_RADIUS, exit, PORTAL_RADIUS) {
            commands.entity(game.player.entity.unwrap()).insert(EffectForcedMarch {
                target: entry,
                speed: 20000.,
            });
        }
    }
}

fn portal_despawn_system(
    mut game: ResMut<Game>,
    mut commands: Commands,
    time: Res<Time>,
    mut portal_entries: Query<(Entity, &mut PortalEntry)>,
    mut portal_exits: Query<(Entity, &mut PortalExit)>,
    ) {
    if portal_exits.is_empty() {
        for (entity, mut entry) in &mut portal_entries {
            entry.0.tick(time.delta());
            if entry.0.finished() {
                game.player.portal_cooldown = Timer::from_seconds(60., TimerMode::Once);
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    for (entity, mut exit) in &mut portal_exits {
        exit.0.tick(time.delta());
        if exit.0.finished() {
            commands.entity(entity).despawn_recursive();

            for (entity, _) in &portal_entries {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn handle_keyboard_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    ) {

    if keyboard_input.just_pressed(KeyCode::Escape) {
        match state.current() {
            GameState::Paused => {
                state.pop().unwrap();
            },
            GameState::StartMenu => {
            },
            _ => {
                state.push(GameState::Paused).unwrap();
            }
        }
    }
}

fn move_rotating_soup_system(
    time: Res<Time>,
    mut soups: Query<(&mut Transform, &mut RotatingSoup)>
    ) {
    for (mut transform, mut soup) in &mut soups {
        soup.theta += soup.dtheta * time.delta_seconds();
        transform.translation.x = soup.theta.cos() * soup.radius;
        transform.translation.y = soup.theta.sin() * soup.radius;
    }
}

fn move_player_system(time: Res<Time>, keyboard_input: Res<Input<KeyCode>>,
               mut transforms: Query<&mut Transform, (With<PlayerTag>, Without<EffectForcedMarch>)>) {
    // Much slower than actual movement
    let speed = 250.0 * GAME_TO_PX * time.delta_seconds();
    for mut transform in &mut transforms {
        let mut movement = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            movement.y += speed;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            movement.y -= speed;
        }
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            movement.x -= speed;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            movement.x += speed;
        }
        movement.clamp_length(0., speed);
        transform.translation = transform.translation.add(movement);
    }
}

fn void_zone_growth_system(
    time: Res<Time>,
    mut void_zone_growths: Query<&mut VoidZoneGrowth>,
    mut void_zones: Query<(&mut CollisionRadius, &mut Transform), With<VoidZone>>
    ) {
    let void_zone_growth = &mut void_zone_growths.single_mut().0;

    void_zone_growth.tick(time.delta());

    let growing = void_zone_growth.just_finished();

    for (mut collision_radius, mut transform) in &mut void_zones {
        if growing {
            collision_radius.0 += VOID_ZONE_GROWTH_AMOUNT;
            let new_scale = collision_radius.0 / VOID_ZONE_START_RADIUS;
            transform.scale.x = new_scale;
            transform.scale.y = new_scale;
        }
    }
}

fn player_hp_check_system(game: Res<Game>,
                          mut state: ResMut<State<GameState>>,
                          ) {
    if game.player.hp <= 0.1 {
        state.push(GameState::Failure).unwrap();
    }
}

fn bullet_age_system(
    time: Res<Time>,
    mut bullets: Query<&mut Bullet>,
    ) {
    for mut bullet in &mut bullets {
        bullet.0 += time.delta_seconds();
    }
}

fn echo_grab_system(
    mut players: Query<&mut Transform, With<PlayerTag>>,
    echos: Query<(&MobEcho, &Transform), Without<PlayerTag>>
    ) {
    for (echo, transform) in &echos {
        if !echo.gottem {
            continue;
        }
        let echo_pos = transform.translation;

        for mut transform_player in &mut players {
            transform_player.translation.x = echo_pos.x;
            transform_player.translation.y = echo_pos.y;
        }
    }
}

fn echo_retarget_system(
    time: Res<Time>,
    players: Query<&Transform, With<PlayerTag>>,
    mut echos: Query<(&mut MobEcho, &Transform, &mut Velocity, &CollisionRadius)>
    ) {
    let transform_player = players.single();

    for (mut echo, transform, mut velocity, radius) in &mut echos {
        if echo.retarget.finished() {
            continue;
        }
        if collide(transform.translation, 0., Vec3::ZERO, MAP_RADIUS - radius.0) {
            continue;
        }
        echo.retarget.tick(time.delta());
        if !echo.retarget.finished() {
            velocity.0 = Vec3::ZERO;
            continue;
        }
        echo.retarget.reset();

        let mut vel = transform_player.translation.sub(transform.translation);
        vel.z = 0.;
        vel = vel.clamp_length(ECHO_SPEED, ECHO_SPEED);
        velocity.0 = vel;
    }
}

fn enemies_hp_check_system(
    mut commands: Commands,
    enemies: Query<(Entity, &Hp), With<Enemy>>,
    ) {
    for (entity_enemy, hp) in &enemies {
        if hp.0 <= 0. {
            commands.entity(entity_enemy).despawn_recursive();
        }
    }
}

pub fn build_update_phase(phase: GameState) -> SystemSet {
    SystemSet::on_update(phase)
        .with_system(handle_mouse_events_system)
        .with_system(handle_spellcasts_system)
        .with_system(handle_keyboard_system)
        .with_system(velocities_system)
        .with_system(move_player_system)
        .with_system(move_rotating_soup_system)
        .with_system(effect_forced_march_system)
        .with_system(collisions_players_edge_system)
        .with_system(collisions_players_echo_system)
        .with_system(collisions_bullets_enemies_system)
        .with_system(collisions_players_soups_system)
        .with_system(collisions_players_enemy_bullets_system)
        .with_system(bullet_age_system)
        .with_system(text_system)
        .with_system(enemies_hp_check_system)
        .with_system(damage_flash_system)
        .with_system(tint_untint_system.after(damage_flash_system))
        .with_system(void_zone_growth_system)
        .with_system(player_hp_check_system)
        .with_system(soup_duration_system)
        .with_system(echo_grab_system)
        .with_system(echo_retarget_system)
        .with_system(scheduled_hint_system)
        .with_system(portal_despawn_system)
}

pub fn setup_phase(
    mut commands: Commands, asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    state: Res<State<GameState>>,
    existing_player: Query<&PlayerTag>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {

    setup_hints(&mut commands, &game, state);

    // Reset all cooldowns and invuln timings
    if !game.continuous {
        game.time_elapsed.reset();
        game.player.hp = 100.;
        game.player.dodge_cooldown.tick(Duration::from_secs_f32(1000.));
        game.player.blink_cooldown.tick(Duration::from_secs_f32(1000.));
        game.player.portal_cooldown.tick(Duration::from_secs_f32(1000.));
        game.player.pull_cooldown.tick(Duration::from_secs_f32(1000.));
        game.player.invuln.tick(Duration::from_secs_f32(1000.));
        game.player.jump.tick(Duration::from_secs_f32(1000.));
    }

    if existing_player.is_empty() {
        game.time_elapsed.reset();
        game.player.hp = 100.;
        game.player.entity = Some(
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(PLAYER_RADIUS * 2., PLAYER_RADIUS * 2.)),
                    ..default()
                },
                texture: asset_server.load("virt.png"),
                transform: Transform::from_xyz(0., 200., LAYER_PLAYER),
                ..default()
            }).insert(PlayerTag).id()
        );
    }

    if game.echo_enabled {
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                // color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(ECHO_RADIUS * 2., ECHO_RADIUS * 2.)),
                ..default()
            },
            texture: asset_server.load("echo.png"),
            transform: Transform::from_xyz(-200., -200., LAYER_MOB),
            ..default()
        })
        .insert(MobEcho {
            gottem: false,
            retarget: Timer::from_seconds(3., TimerMode::Once),
        })
        .insert(Velocity(Vec3::new(0., -ECHO_SPEED, 0.)))
        .insert(CollisionRadius(ECHO_RADIUS));
    }

    commands.spawn()
        .insert(VoidZoneGrowth(Timer::from_seconds(VOID_ZONE_GROWTH_DURATION_SECS, TimerMode::Repeating)))
        .insert(VoidZoneCrabSpawn(Timer::from_seconds(VOID_ZONE_CRAB_SPAWN_DURATION_SECS, TimerMode::Repeating)));

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.9, 0., 0.),
            custom_size: Some(Vec2::new(4., 4.)),
            ..default()
        },
        transform: Transform::from_xyz(0., 0., LAYER_CURSOR),
        ..default()
    }).insert(CursorMark);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("map.png"),
        transform: Transform::from_xyz(0., 0., LAYER_MAP),
        ..default()
    });

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

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("0",
            TextStyle {
                font: asset_server.load("trebuchet_ms.ttf"),
                font_size: 80.,
                color: Color::rgb(0.1, 0.7, 0.7),
            })
            .with_alignment(TextAlignment::CENTER_RIGHT),
        transform: Transform::from_xyz(-90., -HEIGHT / 2. + 155., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::StatusJump,
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

pub fn cleanup_phase(
    mut commands: Commands,
    game: Res<Game>,
    entities: Query<Entity, (Without<PlayerTag>, Without<Camera>)>,
    player_entity: Query<Entity, With<PlayerTag>>,
    ) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
    if !game.continuous {
        for entity in &player_entity {
            commands.entity(entity).despawn_recursive();
        }
    }
}
