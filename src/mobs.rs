use bevy::prelude::*;

use core::f32::consts::PI;

use std::ops::{Add, Sub};

use crate::collisions::CollisionRadius;
use crate::game::*;
use crate::phase::{Velocity, EffectForcedMarch};
use crate::aoes::{spawn_aoe, AoeDesc, Aoe};

pub const BOSS_RADIUS: f32 = 420. * GAME_TO_PX;
pub const BIGBOY_RADIUS: f32 = 120. * GAME_TO_PX;

pub const NOODLE_RADIUS: f32 = 80. * GAME_TO_PX;
pub const NOODLE_SLAM_RADIUS: f32 = 540. * GAME_TO_PX;

pub const ECHO_RADIUS: f32 = 160. * GAME_TO_PX;
pub const ECHO_SPEED: f32 = 160.;
pub const ECHO_DAMAGE: f32 = 5.;

pub const GOLIATH_MOVE_SPEED: f32 = 20.;
pub const GOLIATH_BULLET_SPEED: f32 = 50.;
pub const GOLIATH_BULLET_DAMAGE: f32 = 20.;
pub const GOLIATH_BULLET_KNOCKBACK: f32 = 120. * GAME_TO_PX;

pub const WYVERN_CHARGE_RANGE: f32 = 1200. * GAME_TO_PX;
pub const WYVERN_BULLET_SPEED: f32 = 200.;
pub const WYVERN_BULLET_DAMAGE: f32 = 10.;

pub const TIMECASTER_BULLET_SPEED: f32 = 200.;
pub const TIMECASTER_BULLET_DAMAGE: f32 = 10.;

#[derive(Component)]
pub struct Hp(pub f32);

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Boss {
    pub max_hp: f32,
}

#[derive(Component)]
pub struct MobOrb;

#[derive(Component)]
pub struct MobEcho {
    pub retarget: Timer,
    pub gottem: bool,
}

#[derive(Component)]
pub struct MobCrab;

#[derive(Component)]
pub struct MobGoliath {
    pub shoot_cooldown: Timer,
}

#[derive(Component)]
pub struct MobWyvern {
    pub shoot_cooldown: Timer,
    pub shockwave_cooldown: Timer,
    pub charge_cooldown: Timer,
}

#[derive(Component)]
pub struct MobNoodle {
    pub visibility_start: Timer,
    pub slam_cooldown: Timer,
    pub aoe_desc: AoeDesc,
}

#[derive(Component)]
pub struct MobTimeCaster {
    pub shoot_cooldown: Timer,
}

#[derive(Component)]
pub struct MobSaltspray {
    pub shoot_cooldown: Timer,
    pub aoe_desc: AoeDesc,
}

pub fn spawn_crab(commands: &mut Commands, asset_server: &Res<AssetServer>, crab_pos: Vec3) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            // color: Color::rgb(0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(CRAB_SIZE, CRAB_SIZE * 80. / 120.)),
            ..default()
        },
        texture: asset_server.load("crab.png"),
        transform: Transform::from_translation(crab_pos),
        ..default()
    })
    .insert(MobCrab)
    .insert(Enemy)
    .insert(CollisionRadius(CRAB_SIZE / 2.))
    .insert(Hp(0.1))
    .insert(PhaseEntity);
}

fn get_closest_pos(
    players: &Query<&Transform, With<Player>>,
    pos: Vec3,
    ) -> Option<Vec3> {
    let mut closest = None;
    let mut closest_dist: f32 = 0.;
    for player in players {
        let diff = player.translation.sub(pos);
        if closest.is_none() || diff.length_squared() < closest_dist {
            closest_dist = diff.length_squared();
            closest = Some(player.translation);
        }
    }
    closest
}

pub fn goliath_system(
    time: Res<Time>,
    mut commands: Commands,
    mut goliaths: Query<(&mut MobGoliath, &Transform, &mut Velocity), Without<EffectForcedMarch>>,
    players: Query<&Transform, With<Player>>,
    ) {
    for (mut goliath, transform, mut velocity) in &mut goliaths {
        let player_pos = get_closest_pos(&players, transform.translation);
        if player_pos.is_none() {
            continue;
        }
        let player_pos = player_pos.unwrap();

        let mut vel = player_pos.sub(transform.translation);
        vel.z = 0.;
        vel = vel.clamp_length(0., GOLIATH_MOVE_SPEED);
        velocity.0 = vel;

        let bullet_radius = BULLET_SIZE * 4.;
        let bullet_speed = GOLIATH_BULLET_SPEED;
        vel = vel.clamp_length(bullet_speed, bullet_speed);

        goliath.shoot_cooldown.tick(time.delta());
        if goliath.shoot_cooldown.finished() {
            goliath.shoot_cooldown.reset();

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.4, 0., 0.4),
                    custom_size: Some(Vec2::new(bullet_radius * 2., bullet_radius * 2.)),
                    ..default()
                },
                transform: Transform::from_translation(transform.translation),
                ..default()
            })
            .insert(Velocity(vel))
            .insert(EnemyBullet {
                damage: GOLIATH_BULLET_DAMAGE,
                knockback: GOLIATH_BULLET_KNOCKBACK,
            })
            .insert(CollisionRadius(bullet_radius))
            .insert(PhaseEntity);
        }
    }
}

pub fn wyvern_system(
    time: Res<Time>,
    mut commands: Commands,
    mut wyverns: Query<(Entity, &mut MobWyvern, &Transform), Without<EffectForcedMarch>>,
    players: Query<&Transform, With<Player>>,
    ) {
    for (entity, mut wyvern, transform) in &mut wyverns {
        let player_pos = get_closest_pos(&players, transform.translation);
        if player_pos.is_none() {
            continue;
        }
        let player_pos = player_pos.unwrap();
        let mut vel = player_pos.sub(transform.translation);
        vel.z = 0.;
        let bullet_speed = WYVERN_BULLET_SPEED;
        vel = vel.clamp_length(bullet_speed, bullet_speed);


        wyvern.shoot_cooldown.tick(time.delta());
        if wyvern.shoot_cooldown.finished() {
            wyvern.shoot_cooldown.reset();

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0., 0.),
                    custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                    ..default()
                },
                transform: Transform::from_translation(transform.translation),
                ..default()
            })
            .insert(Velocity(vel))
            .insert(EnemyBullet {
                damage: WYVERN_BULLET_DAMAGE,
                knockback: 0.,
            })
            .insert(CollisionRadius(BULLET_SIZE / 2.))
            .insert(PhaseEntity);
        }

        wyvern.shockwave_cooldown.tick(time.delta());
        if wyvern.shockwave_cooldown.finished() {
            wyvern.shockwave_cooldown.reset();

            for bullet_i in 0..16 {
                let theta = (bullet_i as f32) / 16. * 2. * PI;
                let vel = Vec3::new(theta.cos() * WYVERN_BULLET_SPEED, theta.sin() * WYVERN_BULLET_SPEED, 0.);
                let bullet_radius = BULLET_SIZE / 3.;

                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.8, 0., 0.4),
                        custom_size: Some(Vec2::new(bullet_radius * 2., bullet_radius * 2.)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation),
                    ..default()
                })
                .insert(Velocity(vel))
                .insert(EnemyBullet {
                    damage: WYVERN_BULLET_DAMAGE,
                    knockback: 80. * GAME_TO_PX,
                })
                .insert(CollisionRadius(bullet_radius))
                .insert(PhaseEntity);
            }

        }

        wyvern.charge_cooldown.tick(time.delta());
        if wyvern.charge_cooldown.finished() {
            wyvern.charge_cooldown.reset();
            let speed = WYVERN_CHARGE_RANGE / 0.75;
            let diff = player_pos.sub(transform.translation).clamp_length(0., WYVERN_CHARGE_RANGE);
            let target = transform.translation.add(diff);

            commands.entity(entity).insert(EffectForcedMarch {
                target,
                speed,
            });
        }
    }
}

pub fn noodle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut noodles: Query<(&mut MobNoodle, &Transform, &mut Visibility)>,
    ) {
    for (mut noodle, transform, mut visibility) in &mut noodles {
        if !noodle.visibility_start.finished() {
            noodle.visibility_start.tick(time.delta());
            continue;
        }

        *visibility = Visibility::Inherited;

        noodle.slam_cooldown.tick(time.delta());
        if noodle.slam_cooldown.finished() {
            noodle.slam_cooldown.reset();
            let pos = transform.translation;

            spawn_aoe(&mut commands, &noodle.aoe_desc, Vec3::new(pos.x, pos.y, LAYER_AOE), Aoe {
                visibility_start: None,
                detonation: Timer::from_seconds(2., TimerMode::Once),
                damage: 20.,
                linger: None,
            }, None);
        }
    }
}

pub fn saltspray_system(
    time: Res<Time>,
    mut commands: Commands,
    mut saltsprays: Query<(&mut MobSaltspray, &Transform)>,
    players: Query<&Transform, With<Player>>,
    ) {
    for (mut mob, transform) in &mut saltsprays {
        mob.shoot_cooldown.tick(time.delta());
        if mob.shoot_cooldown.finished() {
            mob.shoot_cooldown.reset();

            let player_pos = get_closest_pos(&players, transform.translation);
            if player_pos.is_none() {
                continue;
            }
            let player_pos = player_pos.unwrap();

            let mut to_player = player_pos.sub(transform.translation);
            to_player.z = 0.;
            let backbone = to_player.clamp_length(300., 300.);
            let perp = Vec3::new(-backbone.y, backbone.x, 0.);


            for i in 0..32 {
                let magnitude = i as f32 / 32.;
                let amount = ((i as f32) / 5.).sin();

                let mut pos = transform.translation + backbone * magnitude;
                pos = pos.add(perp * amount * magnitude);
                pos.z = LAYER_AOE;

                spawn_aoe(&mut commands, &mob.aoe_desc, pos, Aoe {
                    visibility_start: Some(Timer::from_seconds(magnitude / 2., TimerMode::Once)),
                    detonation: Timer::from_seconds(1., TimerMode::Once),
                    damage: 30.,
                    linger: Some(Timer::from_seconds(1., TimerMode::Once)),
                }, None);
            }
        }
    }
}

pub fn timecaster_system(
    time: Res<Time>,
    mut commands: Commands,
    mut timecasters: Query<(&mut MobTimeCaster, &Transform), Without<EffectForcedMarch>>,
    ) {

    for (mut mob, transform) in &mut timecasters {
        mob.shoot_cooldown.tick(time.delta());
        if mob.shoot_cooldown.finished() {
            mob.shoot_cooldown.reset();
            for step in 0..3 {
                let theta = time.elapsed_seconds() / 2. + (step as f32) * PI * 2. / 3.;
                let vel = Vec3::new(
                    theta.cos() * TIMECASTER_BULLET_SPEED,
                    theta.sin() * TIMECASTER_BULLET_SPEED,
                    0.,
                );

                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(1.0, 0., 0.),
                        custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation),
                    ..default()
                })
                .insert(Velocity(vel))
                .insert(EnemyBullet {
                    damage: TIMECASTER_BULLET_DAMAGE,
                    knockback: 10.,
                })
                .insert(CollisionRadius(BULLET_SIZE / 2.))
                .insert(PhaseEntity);
            }
        }
    }
}
