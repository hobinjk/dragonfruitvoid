use bevy::{
    prelude::*,
};
use std::ops::{Add, Mul, Sub};
use std::collections::HashSet;

use crate::game::*;
use crate::aoes::*;
use crate::mobs::*;
use crate::orbs::*;
use crate::waves::*;
use crate::phase::{Velocity, EffectForcedMarch};

use crate::damage_flash::*;

#[derive(Component)]
pub struct CollisionRadius(pub f32);

#[derive(Component)]
pub struct HasHit(pub HashSet<Entity>);

pub fn collide(pos_a: Vec3, radius_a: f32, pos_b: Vec3, radius_b: f32) -> bool {
    let mut diff = pos_b.sub(pos_a);
    diff.z = 0.;
    return diff.length_squared() < (radius_a + radius_b) * (radius_a + radius_b);
}

pub fn collisions_bullets_orbs_system(
    mut commands: Commands,
    players: Query<&Transform, With<PlayerTag>>,
    bullets: Query<(Entity, &Transform), (With<Bullet>, Without<MobOrb>)>,
    mut orbs: Query<(&Transform, &mut Velocity), (With<MobOrb>, Without<Bullet>)>,
    ) {
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
                if velocity_orb.0.length_squared() < orb_max_vel * orb_max_vel * 4. {
                    velocity_orb.0 = velocity_orb.0.add(diff.clamp_length(push_str, push_str)).clamp_length(0., orb_max_vel);
                }
            }
        }
    }
}

pub fn collisions_bullets_enemies_system(
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut bullets: Query<(&Bullet, &Transform, &mut HasHit), (With<Bullet>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &Transform, &Visibility, &CollisionRadius, &mut Hp), (With<Enemy>, Without<Bullet>)>,
    ) {
    for (bullet, transform_bullet, mut has_hit) in &mut bullets {
        let bullet_pos = transform_bullet.translation;
        for (entity_enemy, transform_enemy, visibility, radius_enemy, mut hp) in &mut enemies {
            if has_hit.0.contains(&entity_enemy) || !visibility.is_visible {
                continue;
            }

            let enemy_pos = transform_enemy.translation;
            if !collide(bullet_pos, BULLET_SIZE / 2., enemy_pos, radius_enemy.0) {
                continue;
            }

            has_hit.0.insert(entity_enemy);
            let dist_traveled = bullet.0 * BULLET_SPEED;
            // Reward being close to the target with more damage
            let mut damage_tier = 1.5 - (dist_traveled * PX_TO_GAME / 1200.);
            if damage_tier < 1. {
                damage_tier = 1.;
            }
            hp.0 -= BULLET_DAMAGE * damage_tier;
            if hp.0 > 0. {
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_enemy,
                });
            }
        }
    }
}

pub fn collisions_crabs_orbs_system(
    mut game: ResMut<Game>,
    crabs: Query<&Transform, With<MobCrab>>,
    orbs: Query<&Transform, With<MobOrb>>,
    ) {
    for transform_orb in &orbs {
        let orb_pos = transform_orb.translation;
        for transform_crab in &crabs {
            let crab_pos = transform_crab.translation;
            if collide(orb_pos, ORB_RADIUS, crab_pos, CRAB_SIZE / 2.) {
                game.player.hp = 0.;
                info!("crab hit orb");
            }
        }
    }
}

pub fn collisions_enemies_orbs_system(
    enemies: Query<(&Transform, &CollisionRadius), (With<Enemy>, Without<MobCrab>)>,
    mut orbs: Query<(&Transform, &mut Velocity), With<MobOrb>>,
    ) {
    for (transform_orb, mut velocity_orb) in &mut orbs {
        let orb_pos = transform_orb.translation;
        for (transform_enemy, collision_radius) in &enemies {
            let enemy_pos = transform_enemy .translation;
            if collide(orb_pos, ORB_RADIUS, enemy_pos, collision_radius.0) {
                velocity_orb.0 = velocity_orb.0.mul(-10.);
            }
        }
    }
}


pub fn collisions_orb_targets_system(
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
        if game.continuous {
            let cur_state = state.current().clone();
            state.set(next_game_state(cur_state)).unwrap();
        } else {
            state.push(GameState::Success).unwrap();
        }
    }
}

pub fn collisions_players_edge_system(
    mut game: ResMut<Game>,
    players: Query<&Transform, (With<PlayerTag>, Without<MobOrb>)>,
    ) {
    let transform_player = players.single();
    if !collide(transform_player.translation, 0., Vec3::ZERO, MAP_RADIUS) {
        game.player.hp = 0.;
        info!("player fell off the edge");
    }
}

pub fn collisions_players_echo_system(
    time: Res<Time>,
    mut game: ResMut<Game>,
    players: Query<&Transform, With<PlayerTag>>,
    mut echos: Query<(&mut MobEcho, &Transform, &CollisionRadius), Without<PlayerTag>>,
    ) {

    for (mut echo, transform_echo, radius) in &mut echos {
        let transform_player = players.single();
        let player_pos = transform_player.translation;

        if !collide(player_pos, PLAYER_RADIUS, transform_echo.translation, radius.0) {
            continue;
        }

        if !game.player.invuln.finished() {
            continue;
        }

        echo.gottem = true;

        game.player.hp -= ECHO_DAMAGE * time.delta_seconds();
        game.player.damage_taken += ECHO_DAMAGE * time.delta_seconds();
    }
}

pub fn collisions_players_soups_system(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    players: Query<(Entity, &Transform), With<PlayerTag>>,
    soups: Query<(&Soup, &Transform, &CollisionRadius)>,
    ) {

    let (entity_player, transform_player) = players.single();
    let player_pos = transform_player.translation;
    for (soup, transform_soup, radius) in &soups {
        if !collide(player_pos, 0., transform_soup.translation, radius.0) {
            continue;
        }
        let damage = soup.damage * time.delta_seconds();
        game.player.hp -= damage;
        game.player.damage_taken += damage;
        if soup.damage > 0.1 {
            damage_flash_events.send(DamageFlashEvent {
                entity: entity_player,
            });
        }
    }
}

pub fn collisions_players_waves_system(
    mut game: ResMut<Game>,
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    players: Query<(Entity, &Transform), (With<PlayerTag>, Without<EffectForcedMarch>)>,
    waves: Query<(&Wave, &Visibility, &Transform)>,
    ) {
    if players.is_empty() {
        return;
    }

    let (entity_player, transform_player) = players.single();
    let player_pos = transform_player.translation;

    for (_, visibility, transform) in &waves {
        if !visibility.is_visible {
            continue;
        }

        let r_outer = transform.scale.x * WAVE_MAX_RADIUS;
        let r_inner = r_outer - 20.;

        // Safe because we're in the "eye" of the wave
        if collide(player_pos, 0., transform.translation, r_inner) {
            continue;
        }
        if collide(player_pos, 0., transform.translation, r_outer) {
            if game.player.invuln.finished() && game.player.jump.finished() {
                game.player.hp -= WAVE_DAMAGE;
                game.player.damage_taken += WAVE_DAMAGE;
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_player,
                });
                // Brief invuln from being knocked (not actually knocked because Reasons)
                game.player.invuln = Timer::from_seconds(1., false);
            }
        }
    }
}

pub fn collisions_orbs_edge_system(
    mut game: ResMut<Game>,
    orbs: Query<(&MobOrb, &Transform)>,
    ) {
    for (_, transform_orb) in &orbs {
        if !collide(transform_orb.translation, 0., Vec3::ZERO, MAP_RADIUS - ORB_RADIUS) {
            game.player.hp = 0.;
            info!("orb hit the edge");
        }
    }
}

pub fn collisions_players_enemy_bullets_system(
    mut game: ResMut<Game>,
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut commands: Commands,
    players: Query<(Entity, &Transform), With<PlayerTag>>,
    bullets: Query<(Entity, &EnemyBullet, &Transform, &Velocity, &CollisionRadius)>,
    ) {

    for (entity_bullet, bullet, transform_bullet, velocity, radius) in &bullets {
        let (entity_player, transform_player) = players.single();
        let player_pos = transform_player.translation;

        if !collide(player_pos, PLAYER_RADIUS, transform_bullet.translation, radius.0) {
            continue;
        }

        if game.player.invuln.finished() {
            game.player.hp -= bullet.damage;
            game.player.damage_taken += bullet.damage;
            damage_flash_events.send(DamageFlashEvent {
                entity: entity_player,
            });

            if bullet.knockback.abs() > 0.1 {
                let target = player_pos.add(velocity.0.clamp_length(bullet.knockback, bullet.knockback));
                let speed = bullet.knockback / 0.2;
                commands.entity(entity_player).insert(EffectForcedMarch {
                    target,
                    speed,
                });
            }
            // Brief invuln from being damaged
            game.player.invuln = Timer::from_seconds(0.1, false);
        }

        commands.entity(entity_bullet).despawn_recursive();
    }
}
