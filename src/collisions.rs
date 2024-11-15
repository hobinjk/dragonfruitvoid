use bevy::prelude::*;

use std::collections::HashSet;
use std::ops::{Add, Mul, Sub};

use crate::aoes::*;
use crate::game::*;
use crate::mobs::*;
use crate::orbs::*;
use crate::phase::{EffectForcedMarch, Velocity};
use crate::waves::*;

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

fn bullet_damage(bullet: &Bullet) -> f32 {
    let dist_traveled = bullet.age * BULLET_SPEED;
    // Reward being close to the target with more damage
    let mut damage_tier = 1.5 - (dist_traveled * PX_TO_GAME / 1200.);
    if damage_tier < 1. {
        damage_tier = 1.;
    }
    bullet.base_damage * damage_tier
}

pub fn collisions_bullets_orbs_system(
    players: Query<&Transform, With<Player>>,
    mut bullets: Query<(&Transform, &Bullet, &mut HasHit), (With<Bullet>, Without<MobOrb>)>,
    mut orbs: Query<
        (Entity, &Transform, &mut Velocity, Option<&mut Hp>),
        (With<MobOrb>, Without<Bullet>),
    >,
) {
    for (transform_bullet, bullet, mut has_hit) in &mut bullets {
        let bullet_pos = transform_bullet.translation;
        for (entity_orb, transform_orb, mut velocity_orb, hp) in &mut orbs {
            if has_hit.0.contains(&entity_orb) {
                continue;
            }

            let orb_pos = transform_orb.translation;
            if collide(bullet_pos, BULLET_SIZE / 2., orb_pos, ORB_RADIUS) {
                has_hit.0.insert(entity_orb);

                let player = players.get(bullet.firer);
                if player.is_err() {
                    continue;
                }
                let transform_player = player.unwrap();
                let push_str = 4.;
                let orb_max_vel = 60.;
                let mut diff = orb_pos.sub(transform_player.translation);
                diff.z = 0.;
                if diff.length_squared() > 1.
                    && velocity_orb.0.length_squared() < orb_max_vel * orb_max_vel * 4.
                {
                    velocity_orb.0 = velocity_orb
                        .0
                        .add(diff.clamp_length(push_str, push_str))
                        .clamp_length(0., orb_max_vel);
                }

                if let Some(mut hp) = hp {
                    hp.0 -= bullet_damage(&bullet);
                }
            }
        }
    }
}

pub fn collisions_bullets_enemies_system(
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut bullets: Query<(&Bullet, &Transform, &mut HasHit), (With<Bullet>, Without<Enemy>)>,
    mut enemies: Query<
        (Entity, &Transform, &Visibility, &CollisionRadius, &mut Hp),
        (With<Enemy>, Without<Bullet>, Without<MobOrb>),
    >,
) {
    for (bullet, transform_bullet, mut has_hit) in &mut bullets {
        let bullet_pos = transform_bullet.translation;
        for (entity_enemy, transform_enemy, visibility, radius_enemy, mut hp) in &mut enemies {
            if has_hit.0.contains(&entity_enemy) || visibility == Visibility::Hidden {
                continue;
            }

            let enemy_pos = transform_enemy.translation;
            if !collide(bullet_pos, BULLET_SIZE / 2., enemy_pos, radius_enemy.0) {
                continue;
            }

            has_hit.0.insert(entity_enemy);
            hp.0 -= bullet_damage(&bullet);
            if hp.0 > 0. {
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_enemy,
                });
            }
        }
    }
}

pub fn collisions_crabs_orbs_system(
    mut players: Query<&mut Player>,
    crabs: Query<&Transform, With<MobCrab>>,
    orbs: Query<&Transform, With<MobOrb>>,
) {
    for transform_orb in &orbs {
        let orb_pos = transform_orb.translation;
        for transform_crab in &crabs {
            let crab_pos = transform_crab.translation;
            if collide(orb_pos, ORB_RADIUS, crab_pos, CRAB_SIZE / 2.) {
                for mut player in &mut players {
                    player.damage(999., "crab hit orb");
                }
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
            let enemy_pos = transform_enemy.translation;
            if collide(orb_pos, ORB_RADIUS, enemy_pos, collision_radius.0) {
                velocity_orb.0 = velocity_orb.0.mul(-10.);
            }
        }
    }
}

pub fn collisions_orb_targets_system(
    mut game: ResMut<Game>,
    game_state: Res<State<GameState>>,
    mut res_next_game_state: ResMut<NextState<GameState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
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
            // let cur_state = state.current().clone();
            res_next_game_state.set(next_game_state(*game_state.get()))
            // state.set(next_game_state(cur_state)).unwrap();
        } else {
            next_menu_state.set(MenuState::Success);
            // state.push(GameState::Success).unwrap();
        }
    }
}

pub fn collisions_players_edge_system(mut players: Query<(&mut Player, &Transform)>) {
    for (mut player, transform_player) in &mut players {
        if !collide(transform_player.translation, 0., Vec3::ZERO, MAP_RADIUS) {
            player.damage(999., "player fell off the edge");
            info!("player fell off the edge: {}", transform_player.translation);
        }
    }
}

pub fn collisions_players_echo_system(
    time: Res<Time>,
    mut players: Query<(&mut Player, &Transform)>,
    mut echos: Query<(&mut MobEcho, &Transform, &CollisionRadius), Without<Player>>,
) {
    for (mut echo, transform_echo, radius) in &mut echos {
        for (mut player, transform_player) in &mut players {
            let player_pos = transform_player.translation;

            if !collide(
                player_pos,
                PLAYER_RADIUS,
                transform_echo.translation,
                radius.0,
            ) {
                continue;
            }

            if !player.invuln.finished() {
                continue;
            }

            echo.gottem = true;

            player.damage(ECHO_DAMAGE * time.delta_seconds(), "echo hug");
            player.damage_taken += ECHO_DAMAGE * time.delta_seconds();
        }
    }
}

pub fn collisions_players_soups_system(
    time: Res<Time>,
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    soups: Query<(&Soup, &Transform, &CollisionRadius)>,
) {
    for (entity_player, transform_player, mut player) in &mut players {
        let player_pos = transform_player.translation;
        for (soup, transform_soup, radius) in &soups {
            if !collide(player_pos, 0., transform_soup.translation, radius.0) {
                continue;
            }
            let damage = soup.damage * time.delta_seconds();
            player.damage(damage, "soup");
            player.damage_taken += damage;
            if soup.damage > 0.1 {
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_player,
                });
            }
        }
    }
}

pub fn collisions_players_waves_system(
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut players: Query<(Entity, &Transform, &mut Player), Without<EffectForcedMarch>>,
    waves: Query<(&Wave, &Visibility, &Transform)>,
) {
    if players.is_empty() {
        return;
    }

    for (entity_player, transform_player, mut player) in &mut players {
        let player_pos = transform_player.translation;

        for (_, visibility, transform) in &waves {
            if visibility == Visibility::Hidden {
                continue;
            }

            let r_outer = transform.scale.x * WAVE_MAX_RADIUS;
            let r_inner = r_outer - 20.;

            // Safe because we're in the "eye" of the wave
            if collide(player_pos, 0., transform.translation, r_inner) {
                continue;
            }
            if collide(player_pos, 0., transform.translation, r_outer) {
                if player.invuln.finished() && player.jump.finished() {
                    player.damage(WAVE_DAMAGE, "wave");
                    player.damage_taken += WAVE_DAMAGE;
                    damage_flash_events.send(DamageFlashEvent {
                        entity: entity_player,
                    });
                    // Brief invuln from being knocked (not actually knocked because Reasons)
                    player.invuln = Timer::from_seconds(1., TimerMode::Once);
                }
            }
        }
    }
}

pub fn collisions_orbs_edge_system(
    mut players: Query<&mut Player>,
    orbs: Query<(&MobOrb, &Transform)>,
) {
    for (_, transform_orb) in &orbs {
        if !collide(
            transform_orb.translation,
            0.,
            Vec3::ZERO,
            MAP_RADIUS - ORB_RADIUS,
        ) {
            for mut player in &mut players {
                player.damage(999., "orb hit the edge");
            }
            info!("orb hit the edge: {}", transform_orb.translation);
        }
    }
}

pub fn collisions_players_enemy_bullets_system(
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut commands: Commands,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    bullets: Query<(
        Entity,
        &EnemyBullet,
        &Transform,
        &Velocity,
        &CollisionRadius,
    )>,
) {
    for (entity_bullet, bullet, transform_bullet, velocity, radius) in &bullets {
        for (entity_player, transform_player, mut player) in &mut players {
            let player_pos = transform_player.translation;

            if !collide(
                player_pos,
                PLAYER_RADIUS,
                transform_bullet.translation,
                radius.0,
            ) {
                continue;
            }

            if player.invuln.finished() {
                player.damage(bullet.damage, "bullet");
                player.damage_taken += bullet.damage;
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_player,
                });

                if bullet.knockback.abs() > 0.1 {
                    let target =
                        player_pos.add(velocity.0.clamp_length(bullet.knockback, bullet.knockback));
                    let speed = bullet.knockback / 0.2;
                    commands
                        .entity(entity_player)
                        .insert(EffectForcedMarch { target, speed });
                }
                // Brief invuln from being damaged
                player.invuln = Timer::from_seconds(0.1, TimerMode::Once);
            }

            commands.entity(entity_bullet).despawn_recursive();
        }
    }
}
