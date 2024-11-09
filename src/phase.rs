use bevy::{
    prelude::*,
    render::color::Color,
    sprite::{Anchor, MaterialMesh2dBundle},
    window::CursorMoved,
};

use std::collections::HashSet;
use std::ops::{Add, Mul, Sub};
use std::time::Duration;

use crate::collisions::*;
use crate::damage_flash::*;
use crate::hints::{scheduled_hint_system, setup_hints};
use crate::mobs::*;
use crate::ui::*;
use crate::{ai::player_ai_system, game::*};
use crate::{ai::AiPlayer, ai::AiRole, aoes::soup_duration_system};

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
pub struct PortalEntry {
    despawn_timer: Timer,
    owner: Entity,
}

#[derive(Component)]
pub struct PortalExit {
    despawn_timer: Timer,
    owner: Entity,
}

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
    mut tra_vels: Query<(Entity, &mut Transform, &Velocity)>,
) {
    for (entity, mut transform, velocity) in &mut tra_vels {
        transform.translation = transform
            .translation
            .add(velocity.0.mul(time.delta_seconds()));

        let pos = transform.translation;

        if pos.x < -WIDTH || pos.x > WIDTH || pos.y < -HEIGHT || pos.y > HEIGHT {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn effect_forced_march_system(
    time: Res<Time>,
    mut commands: Commands,
    mut pulleds: Query<(Entity, &mut Transform, &EffectForcedMarch)>,
) {
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

fn game_player_time_system(
    mut game: ResMut<Game>,
    time: Res<Time>,
    mut players: Query<&mut Player>,
) {
    game.time_elapsed.tick(time.delta());

    for mut player in &mut players {
        player.shoot_cooldown.tick(time.delta());
        player.dodge_cooldown.tick(time.delta());
        player.pull_cooldown.tick(time.delta());
        player.blink_cooldown.tick(time.delta());
        player.portal_cooldown.tick(time.delta());
        player.jump_cooldown.tick(time.delta());
        player.invuln.tick(time.delta());
        player.jump.tick(time.delta());
        player.hp += time.delta_seconds() * PLAYER_REGEN;
        if player.hp > 100. {
            player.hp = 100.;
        }
    }
}

fn game_player_damage_system(mut game: ResMut<Game>, players: Query<&Player>) {
    let mut player_damage_taken = 0.;

    for player in &players {
        player_damage_taken += player.damage_taken;
    }

    game.player_damage_taken = f32::max(game.player_damage_taken, player_damage_taken);
}

fn handle_mouse_events_system(
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut players: Query<(Entity, &Transform, &mut Player), (Without<CursorMark>, Without<AiPlayer>)>,
    mut cursors: Query<&mut Transform, With<CursorMark>>,
) {
    for (entity_player, transform_player, mut player) in &mut players {
        let player_loc = transform_player.translation;
        if player.shoot_cooldown.finished()
            && (mouse_button_input.pressed(MouseButton::Left)
                || keyboard_input.pressed(KeyCode::Key1))
        {
            let cursor = cursors.single();
            let mut vel = cursor.translation.sub(player_loc);
            vel.z = 0.;
            vel = vel.clamp_length(BULLET_SPEED, BULLET_SPEED);

            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.89, 0.39, 0.95),
                        custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(player_loc.x, player_loc.y, LAYER_BULLET),
                    ..default()
                })
                .insert(Velocity(vel))
                .insert(Bullet {
                    age: 0.,
                    firer: entity_player,
                })
                .insert(HasHit(HashSet::new()))
                .insert(PhaseEntity);
            player.shoot_cooldown.reset();
        }
    }

    for event in cursor_moved_events.iter() {
        let mut cursor = cursors.single_mut();
        cursor.translation.x = event.position.x - WIDTH / 2.;
        cursor.translation.y = HEIGHT / 2. - event.position.y;
    }
}

fn handle_spellcasts_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut players: Query<(Entity, &Transform, &mut Player), (Without<CursorMark>, Without<AiPlayer>)>,
    portal_entries: Query<(&Transform, &PortalEntry)>,
    portal_exits: Query<(&Transform, &PortalExit)>,
    cursors: Query<&Transform, With<CursorMark>>,
    crabs: Query<(Entity, &Transform, &MobCrab)>,
) {
    let cursor_loc = cursors.single().translation;

    for (entity_player, transform_player, mut player) in &mut players {
        let player_loc = transform_player.translation;

        if player.jump_cooldown.finished() && keyboard_input.pressed(KeyCode::Space) {
            player.jump = Timer::from_seconds(0.5, TimerMode::Once);
            player.jump_cooldown.reset();
        }

        if player.dodge_cooldown.finished() && keyboard_input.pressed(KeyCode::V) {
            let dodge_range = 300. * GAME_TO_PX;
            let dodge_speed = dodge_range / 0.75;
            let diff = cursor_loc
                .sub(player_loc)
                .clamp_length(dodge_range, dodge_range);
            let target = player_loc.add(diff);

            commands.entity(entity_player).insert(EffectForcedMarch {
                target,
                speed: dodge_speed,
            });

            player.invuln = Timer::from_seconds(0.75, TimerMode::Once);
            player.dodge_cooldown.reset();
        }

        if player.blink_cooldown.finished() && keyboard_input.pressed(KeyCode::E) {
            let blink_range = 1200.0 * GAME_TO_PX;
            let blink_speed = blink_range / 0.1;
            let mut diff = cursor_loc.sub(player_loc);
            diff.z = 0.;
            diff = diff.clamp_length(0., blink_range);
            let target = player_loc.add(diff);

            commands.entity(entity_player).insert(EffectForcedMarch {
                target,
                speed: blink_speed,
            });

            player.invuln = Timer::from_seconds(0.1, TimerMode::Once);
            player.blink_cooldown.reset();
        }

        if player.pull_cooldown.finished() && keyboard_input.pressed(KeyCode::Key4) {
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

            player.pull_cooldown.reset();
        }

        if player.portal_cooldown.finished() && keyboard_input.just_pressed(KeyCode::R) {
            let portal_loc = player_loc;

            if portal_entries.is_empty() {
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0., 1., 1.),
                            custom_size: Some(Vec2::new(PORTAL_RADIUS * 2., PORTAL_RADIUS * 2.)),
                            ..default()
                        },
                        texture: asset_server.load("ring.png"),
                        transform: Transform::from_translation(portal_loc),
                        ..default()
                    })
                    .insert(PortalEntry {
                        despawn_timer: Timer::from_seconds(60., TimerMode::Once),
                        owner: entity_player,
                    })
                    .insert(PhaseEntity);

                player.portal_cooldown = Timer::from_seconds(0.5, TimerMode::Once);
            } else if portal_exits.is_empty() {
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(1., 0.7, 0.),
                            custom_size: Some(Vec2::new(PORTAL_RADIUS * 2., PORTAL_RADIUS * 2.)),
                            ..default()
                        },
                        texture: asset_server.load("ring.png"),
                        transform: Transform::from_translation(portal_loc),
                        ..default()
                    })
                    .insert(PortalExit {
                        despawn_timer: Timer::from_seconds(10., TimerMode::Once),
                        owner: entity_player,
                    })
                    .insert(PhaseEntity);

                player.portal_cooldown = Timer::from_seconds(60., TimerMode::Once);
            }
        }

        if keyboard_input.just_pressed(KeyCode::F)
            && !portal_entries.is_empty()
            && !portal_exits.is_empty()
        {
            for (transform_entry, entry) in &portal_entries {
                if !collide(
                    player_loc,
                    PLAYER_RADIUS,
                    transform_entry.translation,
                    PORTAL_RADIUS,
                ) {
                    continue;
                }

                for (transform_exit, exit) in &portal_exits {
                    if exit.owner != entry.owner {
                        continue;
                    }
                    commands.entity(entity_player).insert(EffectForcedMarch {
                        target: transform_exit.translation,
                        speed: 20000.,
                    });
                    break;
                }
            }

            for (transform_exit, exit) in &portal_exits {
                if !collide(
                    player_loc,
                    PLAYER_RADIUS,
                    transform_exit.translation,
                    PORTAL_RADIUS,
                ) {
                    continue;
                }

                for (transform_entry, entry) in &portal_entries {
                    if exit.owner != entry.owner {
                        continue;
                    }
                    commands.entity(entity_player).insert(EffectForcedMarch {
                        target: transform_entry.translation,
                        speed: 20000.,
                    });
                    break;
                }
            }
        }
    }
}

fn portal_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut players: Query<&mut Player>,
    mut portal_entries: Query<(Entity, &mut PortalEntry)>,
    mut portal_exits: Query<(Entity, &mut PortalExit)>,
) {
    if portal_exits.is_empty() {
        for (entity, mut entry) in &mut portal_entries {
            entry.despawn_timer.tick(time.delta());
            if entry.despawn_timer.finished() {
                if let Ok(mut player) = players.get_mut(entry.owner) {
                    player.portal_cooldown = Timer::from_seconds(60., TimerMode::Once);
                }
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    for (entity, mut exit) in &mut portal_exits {
        exit.despawn_timer.tick(time.delta());
        if exit.despawn_timer.finished() {
            commands.entity(entity).despawn_recursive();

            for (entity, _) in &portal_entries {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn handle_keyboard_system(
    keyboard_input: Res<Input<KeyCode>>,
    menu_state: Res<State<MenuState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match menu_state.get() {
            MenuState::Paused | MenuState::PausedShowHint => {
                next_menu_state.set(MenuState::Unpaused);
            }
            MenuState::StartMenu | MenuState::Failure | MenuState::Success => {}
            MenuState::Unpaused => {
                next_menu_state.set(MenuState::Paused);
            }
        }
    }
}

fn move_rotating_soup_system(
    time: Res<Time>,
    mut soups: Query<(&mut Transform, &mut RotatingSoup)>,
) {
    for (mut transform, mut soup) in &mut soups {
        soup.theta += soup.dtheta * time.delta_seconds();
        transform.translation.x = soup.theta.cos() * soup.radius;
        transform.translation.y = soup.theta.sin() * soup.radius;
    }
}

fn move_player_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut transforms: Query<
        (&mut Transform, &Player),
        (Without<EffectForcedMarch>, Without<AiPlayer>),
    >,
) {
    // Much slower than actual movement
    let speed = 250.0 * GAME_TO_PX * time.delta_seconds();
    for (mut transform, player) in &mut transforms {
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
        movement = movement.clamp_length(0., speed);
        transform.translation = transform.translation.add(movement);
    }
}

fn void_zone_growth_system(
    time: Res<Time>,
    mut void_zone_growths: Query<&mut VoidZoneGrowth>,
    mut void_zones: Query<(&mut CollisionRadius, &mut Transform), With<VoidZone>>,
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

fn player_hp_check_system(players: Query<(Entity, &Player)>, mut commands: Commands) {
    for (entity_player, player) in &players {
        if player.hp <= 0.1 {
            commands.entity(entity_player).despawn_recursive();
        }
    }
}

fn player_count_system(players: Query<&Player>, mut next_menu_state: ResMut<NextState<MenuState>>) {
    if players.is_empty() {
        next_menu_state.set(MenuState::Failure);
    }
}

fn bullet_age_system(
    time: Res<Time>,
    game: Res<Game>,
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in &mut bullets {
        bullet.age += time.delta_seconds();
        if !game.unlimited_range_enabled {
            if bullet.age * BULLET_SPEED > BULLET_RANGE {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn echo_grab_system(
    mut players: Query<&mut Transform, With<Player>>,
    echos: Query<(&MobEcho, &Transform), Without<Player>>,
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
    players: Query<&Transform, With<Player>>,
    mut echos: Query<(&mut MobEcho, &Transform, &mut Velocity, &CollisionRadius)>,
) {
    for transform_player in &players {
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
}

fn enemies_hp_check_system(mut commands: Commands, enemies: Query<(Entity, &Hp), With<Enemy>>) {
    for (entity_enemy, hp) in &enemies {
        if hp.0 <= 0. {
            commands.entity(entity_enemy).despawn_recursive();
        }
    }
}

pub fn add_update_phase_set(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_mouse_events_system,
            handle_spellcasts_system,
            handle_keyboard_system,
            velocities_system,
            move_player_system,
            move_rotating_soup_system,
            effect_forced_march_system,
            player_ai_system,
        )
            .in_set(PhaseSet::UpdatePhase),
    );

    app.add_systems(
        Update,
        (
            collisions_players_edge_system,
            collisions_players_echo_system,
            collisions_bullets_enemies_system,
            collisions_players_soups_system,
            collisions_players_enemy_bullets_system,
        )
            .in_set(PhaseSet::UpdatePhase),
    );

    app.add_systems(
        Update,
        (
            bullet_age_system,
            player_text_system,
            enemies_hp_check_system,
            void_zone_growth_system,
            player_hp_check_system,
            soup_duration_system,
            echo_grab_system,
            echo_retarget_system,
            scheduled_hint_system,
            portal_despawn_system,
            game_player_time_system,
            game_player_damage_system,
            player_count_system,
        )
            .in_set(PhaseSet::UpdatePhase),
    );

    app.add_systems(
        Update,
        (damage_flash_system, tint_untint_system)
            .chain()
            .in_set(PhaseSet::UpdatePhase),
    );
}

pub fn setup_phase(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    state: Res<State<GameState>>,
    mut players: Query<&mut Player>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    setup_hints(&mut commands, &game, state);

    // Reset all cooldowns and invuln timings
    if !game.continuous {
        game.time_elapsed.reset();
        game.player_damage_taken = 0.;
        for mut player in &mut players {
            player.hp = 100.;
            player.dodge_cooldown.tick(Duration::from_secs_f32(1000.));
            player.blink_cooldown.tick(Duration::from_secs_f32(1000.));
            player.portal_cooldown.tick(Duration::from_secs_f32(1000.));
            player.pull_cooldown.tick(Duration::from_secs_f32(1000.));
            player.invuln.tick(Duration::from_secs_f32(1000.));
            player.jump.tick(Duration::from_secs_f32(1000.));
        }
    }

    if players.is_empty() {
        game.time_elapsed.reset();

        for x in 0..10 {
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(PLAYER_RADIUS * 2., PLAYER_RADIUS * 2.)),
                        ..default()
                    },
                    texture: asset_server.load("virt.png"),
                    transform: Transform::from_xyz((x as f32 - 4.5) * 10., 200., LAYER_PLAYER),
                    ..default()
                })
                .insert(Player { ..default() })
                .insert(AiPlayer { role: AiRole::Ham1 });
        }

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(PLAYER_RADIUS * 2., PLAYER_RADIUS * 2.)),
                    ..default()
                },
                texture: asset_server.load("virt.png"),
                transform: Transform::from_xyz(0., 200., LAYER_PLAYER),
                ..default()
            })
            .insert(Player { ..default() });
    }

    if game.echo_enabled {
        commands
            .spawn(SpriteBundle {
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
            .insert(CollisionRadius(ECHO_RADIUS))
            .insert(PhaseEntity);
    }

    commands
        .spawn(VoidZoneGrowth(Timer::from_seconds(
            VOID_ZONE_GROWTH_DURATION_SECS,
            TimerMode::Repeating,
        )))
        .insert(PhaseEntity);

    commands
        .spawn(VoidZoneCrabSpawn(Timer::from_seconds(
            VOID_ZONE_CRAB_SPAWN_DURATION_SECS,
            TimerMode::Repeating,
        )))
        .insert(PhaseEntity);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.9, 0., 0.),
                custom_size: Some(Vec2::new(4., 4.)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., LAYER_CURSOR),
            ..default()
        })
        .insert(CursorMark)
        .insert(PhaseEntity);

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("map.png"),
            transform: Transform::from_xyz(0., 0., LAYER_MAP),
            ..default()
        })
        .insert(PhaseEntity);

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

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("hp", text_style.clone())
                .with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(0., -HEIGHT / 2. + 55., LAYER_TEXT),
            ..default()
        })
        .insert(TextDisplay {
            value: TextValue::Hp,
            sprite: None,
        })
        .insert(PhaseEntity);

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.6, 0.1, 0.1))),
            transform: Transform::from_xyz(0., -HEIGHT / 2. + 55., LAYER_UI),
            ..default()
        })
        .insert(PhaseEntity);

    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "0",
                TextStyle {
                    font: asset_server.load("trebuchet_ms.ttf"),
                    font_size: 80.,
                    color: Color::rgb(0.7, 0.7, 0.1),
                },
            )
            .with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(0., -HEIGHT / 2. + 155., LAYER_TEXT),
            ..default()
        })
        .insert(TextDisplay {
            value: TextValue::CooldownDodge,
            sprite: None,
        })
        .insert(PhaseEntity);

    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "0",
                TextStyle {
                    font: asset_server.load("trebuchet_ms.ttf"),
                    font_size: 80.,
                    color: Color::rgb(0.1, 0.7, 0.7),
                },
            )
            .with_alignment(TextAlignment::Right),
            text_anchor: Anchor::CenterRight,
            transform: Transform::from_xyz(-90., -HEIGHT / 2. + 155., LAYER_TEXT),
            ..default()
        })
        .insert(TextDisplay {
            value: TextValue::StatusJump,
            sprite: None,
        })
        .insert(PhaseEntity);

    let sprite_pull = commands
        .spawn(SpriteBundle {
            texture: asset_server.load("pull.png"),
            transform: Transform::from_xyz(-128., -HEIGHT / 2. + 55., LAYER_UI),
            ..default()
        })
        .insert(PhaseEntity)
        .id();

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("", text_style.clone()).with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(-128., -HEIGHT / 2. + 55., LAYER_TEXT),
            ..default()
        })
        .insert(TextDisplay {
            value: TextValue::CooldownPull,
            sprite: Some(sprite_pull),
        })
        .insert(PhaseEntity);

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("4", text_binding_style.clone())
                .with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(-128., -HEIGHT / 2. + binding_y, LAYER_TEXT),
            ..default()
        })
        .insert(PhaseEntity);

    let sprite_blink = commands
        .spawn(SpriteBundle {
            texture: asset_server.load("blink.png"),
            transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_UI),
            ..default()
        })
        .insert(PhaseEntity)
        .id();

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("", text_style.clone()).with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_TEXT),
            ..default()
        })
        .insert(TextDisplay {
            value: TextValue::CooldownBlink,
            sprite: Some(sprite_blink),
        })
        .insert(PhaseEntity);

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("E", text_binding_style.clone())
                .with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(128., -HEIGHT / 2. + binding_y, LAYER_TEXT),
            ..default()
        })
        .insert(PhaseEntity);

    let sprite_portal = commands
        .spawn(SpriteBundle {
            texture: asset_server.load("portal.png"),
            transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_UI),
            ..default()
        })
        .insert(PhaseEntity)
        .id();

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("", text_style.clone()).with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_TEXT),
            ..default()
        })
        .insert(TextDisplay {
            value: TextValue::CooldownPortal,
            sprite: Some(sprite_portal),
        })
        .insert(PhaseEntity);

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("R", text_binding_style.clone())
                .with_alignment(TextAlignment::Center),
            text_anchor: Anchor::Center,
            transform: Transform::from_xyz(256., -HEIGHT / 2. + binding_y, LAYER_TEXT),
            ..default()
        })
        .insert(PhaseEntity);
}

pub fn cleanup_phase(
    mut commands: Commands,
    game: Res<Game>,
    entities: Query<Entity, With<PhaseEntity>>,
    player_entity: Query<Entity, With<Player>>,
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
