use bevy::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::ops::{Add, Mul, Sub};

use crate::boss_phase::{Puddle, PuddleSpawn};
use crate::game::Player;
use crate::greens::StackGreen;
use crate::mobs::Enemy;
use crate::orbs::ORB_RADIUS;
use crate::{
    collide, Bullet, CollisionRadius, Game, HasHit, MobOrb, OrbTarget, PhaseEntity, Soup,
    StackGreenIndicator, Velocity, BULLET_SIZE, BULLET_SPEED, GAME_TO_PX, LAYER_BULLET, MAP_RADIUS,
    PLAYER_RADIUS,
};

pub enum AiRole {
    Virt1,
    Virt2,
    Herald1,
    Herald2,
    Ham1,
    Ham2,
    Dps1,
    Dps2,
    Dps3,
    Dps4,
}

#[derive(Component)]
pub struct AiPlayer {
    pub role: AiRole,
}

enum Action {
    Move(Vec3),
    Shoot(Vec3),
    Rest,
}

struct Thought {
    utility: f32,
    action: Action,
}

impl Thought {
    const REST: Thought = Thought {
        utility: 0.,
        action: Action::Rest,
    };
}

fn think_dont_fall_off_edge(player_pos: &Vec3) -> Thought {
    let safe_map_radius = MAP_RADIUS - PLAYER_RADIUS * 1.1;
    if player_pos.length_squared() < safe_map_radius * safe_map_radius {
        return Thought {
            utility: 0.,
            action: Action::Rest,
        };
    }

    Thought {
        utility: 1.,
        action: Action::Move(Vec3::new(0., 0., 0.)),
    }
}

fn is_safe_for_orb(player_pos: Vec3, orb_pos: Vec3, enemy_pos: Vec3) -> bool {
    let shoot_dir = enemy_pos.sub(player_pos).truncate();
    let orb_dir = orb_pos.sub(player_pos).truncate();
    let angle_orb = (ORB_RADIUS * 3.).atan2(orb_dir.length());
    let mut angle_shoot = orb_dir.angle_between(shoot_dir);

    if angle_shoot < 0. {
        angle_shoot += 2. * PI;
    }

    if angle_shoot < angle_orb {
        return false;
    }
    if 2. * PI - angle_shoot < angle_orb {
        return false;
    }
    true
}

fn think_shoot_crab(
    player_pos: Vec3,
    orb_pos: Vec3,
    enemies: &Query<(&Enemy, &Transform), Without<Player>>,
) -> Thought {
    let mut closest_enemy: Option<(f32, Vec3)> = None;
    for (_enemy, transform_enemy) in enemies {
        let enemy_pos: Vec3 = transform_enemy.translation;
        if !is_safe_for_orb(player_pos, orb_pos, enemy_pos) {
            continue;
        }
        let shoot_dir = enemy_pos.sub(orb_pos).truncate();
        let dist_sq = shoot_dir.length_squared();
        closest_enemy = match closest_enemy {
            None => Some((dist_sq, enemy_pos)),
            Some((closest_dist_sq, closest_pos)) => {
                if closest_dist_sq < dist_sq {
                    Some((closest_dist_sq, closest_pos))
                } else {
                    Some((dist_sq, enemy_pos))
                }
            }
        }
    }

    match closest_enemy {
        None => Thought::REST,
        Some((_, closest_pos)) => Thought {
            utility: 0.4,
            action: Action::Shoot(closest_pos.sub(player_pos)),
        },
    }
}

fn think_shoot_enemy(
    player_pos: Vec3,
    enemies: &Query<(&Enemy, &Transform), Without<Player>>,
) -> Thought {
    let mut closest_enemy: Option<(f32, Vec3)> = None;
    for (_enemy, transform_enemy) in enemies {
        let enemy_pos: Vec3 = transform_enemy.translation;
        let dist_sq = enemy_pos.sub(player_pos).length_squared();
        closest_enemy = match closest_enemy {
            None => Some((dist_sq, enemy_pos)),
            Some((closest_dist_sq, closest_pos)) => {
                if closest_dist_sq < dist_sq {
                    Some((closest_dist_sq, closest_pos))
                } else {
                    Some((dist_sq, enemy_pos))
                }
            }
        }
    }

    match closest_enemy {
        None => Thought::REST,
        Some((_, closest_pos)) => Thought {
            utility: 0.1,
            action: Action::Shoot(closest_pos.sub(player_pos)),
        },
    }
}

// Align to push in correct dir then shoot
fn think_push_orb(
    player_pos: Vec3,
    orb_pos: Vec3,
    orb_vel: &Velocity,
    orb_target_pos: Vec3,
    orb_dest_pos: Vec3,
    is_active: bool,
) -> Thought {
    let cur_vel = orb_vel.0.truncate();
    // The velocity change that happens if we push the orb from here
    let cur_push_vel = orb_pos.sub(player_pos).truncate().normalize();
    // The velocity we want the orb to have
    let des_orb_vel = orb_target_pos
        .sub(orb_pos)
        .truncate()
        .normalize()
        .mul(300. * GAME_TO_PX);
    // The push we want to apply to the orb
    let mut des_push_vel = des_orb_vel.sub(cur_vel);

    // The greater the difference the more we need to push the orb
    let push_utility = des_push_vel.length() / (400. * GAME_TO_PX);

    des_push_vel = des_push_vel.normalize();

    // cos(angle between vels) means that 1 is good, 0 is bad
    let push_goodness = des_push_vel.dot(cur_push_vel);
    if push_goodness > 0.99 && is_active {
        // roughly +-8 degrees
        return Thought {
            utility: push_utility,
            action: Action::Shoot(des_push_vel.extend(0.)),
        };
    }

    if is_active {
        let good_push_pos = orb_pos.sub(des_push_vel.extend(0.).mul(ORB_RADIUS * 1.3));

        return Thought {
            utility: 0.4,
            action: Action::Move(good_push_pos),
        };
    }

    let good_prep_pos = orb_dest_pos.sub(des_push_vel.extend(0.).mul(ORB_RADIUS * 1.3));

    return Thought {
        utility: 0.3,
        action: Action::Move(good_prep_pos),
    };
}

pub fn player_ai_boss_phase_system(
    time: Res<Time>,
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player, &AiPlayer, &mut Transform)>,
    enemies: Query<(&Enemy, &Transform), Without<Player>>,
    greens: Query<(&StackGreen, &Children)>,
    indicators: Query<(&StackGreenIndicator, &Transform), Without<Player>>,
    puddle_spawns: Query<&PuddleSpawn>,
    puddles: Query<&Puddle>,
    soups: Query<(&Soup, &Transform, &CollisionRadius), Without<Player>>,
) {
    for (entity_player, mut player, ai_player, mut transform) in &mut players {
        let player_pos = transform.translation;

        let thoughts: Vec<Thought> = vec![
            think_dont_fall_off_edge(&player_pos),
            think_shoot_enemy(player_pos, &enemies),
            think_avoid_soups(player_pos, &soups),
            think_do_greens(&ai_player.role, &greens, &indicators),
            think_do_puddles(player_pos, &ai_player.role, &puddle_spawns, &puddles),
            think_go_home(player_pos),
        ];

        let best_thought = thoughts
            .iter()
            .reduce(|a, b| if a.utility > b.utility { a } else { b })
            .unwrap_or(&Thought::REST);

        act_on_thought(
            best_thought,
            &time,
            &mut commands,
            &mut player,
            entity_player,
            &mut transform,
        );
    }
}

fn think_do_greens(
    role: &AiRole,
    greens: &Query<(&StackGreen, &Children)>,
    indicators: &Query<(&StackGreenIndicator, &Transform), Without<Player>>,
) -> Thought {
    let no_green: usize = 9001;
    let green_team: usize = match role {
        AiRole::Dps1 | AiRole::Dps2 | AiRole::Dps3 | AiRole::Dps4 => no_green,
        AiRole::Herald1 | AiRole::Herald2 => 0,
        AiRole::Virt1 | AiRole::Virt2 => 1,
        AiRole::Ham1 | AiRole::Ham2 => 2,
    };

    if green_team == no_green {
        return Thought::REST;
    }

    for (green, children) in greens {
        if green.visibility_start.remaining_secs() > 3. {
            continue;
        }

        if green.detonation.finished() {
            continue;
        }

        let mut green_pos = None;
        for &child in children.iter() {
            if let Ok((indicator, transform_indicator)) = indicators.get(child) {
                if indicator.0 == green_team {
                    green_pos = Some(transform_indicator.translation);
                }
            }
        }

        if let Some(green_pos) = green_pos {
            return Thought {
                utility: 0.95,
                action: Action::Move(green_pos),
            };
        }
    }

    Thought::REST
}

fn think_do_puddles(
    player_pos: Vec3,
    role: &AiRole,
    puddle_spawns: &Query<&PuddleSpawn>,
    puddles: &Query<&Puddle>,
) -> Thought {
    match role {
        AiRole::Dps1
        | AiRole::Dps2
        | AiRole::Dps3
        | AiRole::Dps4
        | AiRole::Herald1
        | AiRole::Herald2
        | AiRole::Virt1
        | AiRole::Virt2 => return Thought::REST,
        AiRole::Ham1 | AiRole::Ham2 => {}
    };

    for puddle in puddles {
        if puddle.drop.percent() < 4. / 6. {
            let r = MAP_RADIUS - PLAYER_RADIUS;
            let theta = player_pos.x.atan2(player_pos.y);
            let target_pos = Vec3::new(r * theta.sin(), r * theta.cos(), 0.);

            return Thought {
                utility: 0.6,
                action: Action::Move(target_pos),
            };
        }
    }

    for puddle_spawn in puddle_spawns {
        if puddle_spawn.visibility_start.remaining_secs() > 4.
            || puddle_spawn.visibility_start.finished()
        {
            continue;
        }

        // Rotate current position towards the back and close to center
        let mut r = player_pos.length();
        let mut theta = player_pos.x.atan2(player_pos.y);
        if theta < 0. {
            theta -= 0.1;
        } else {
            theta += 0.1;
        }
        r -= PLAYER_RADIUS * 2.;
        let target_pos = Vec3::new(r * theta.sin(), r * theta.cos(), 0.);

        return Thought {
            utility: 0.5,
            action: Action::Move(target_pos),
        };
    }

    Thought::REST
}

fn think_do_aoes() -> Thought {
    Thought::REST
}

fn act_on_thought(
    thought: &Thought,
    time: &Res<Time>,
    commands: &mut Commands,
    player: &mut Player,
    entity_player: Entity,
    player_transform: &mut Transform,
) {
    let speed = 250.0 * GAME_TO_PX * time.delta_seconds();

    match thought.action {
        Action::Rest => {}
        Action::Move(target_pos) => {
            let movement = target_pos
                .sub(player_transform.translation)
                .truncate()
                .clamp_length(0., speed)
                .extend(0.);
            player_transform.translation = player_transform.translation.add(movement);
        }
        Action::Shoot(dir) => {
            let player_pos = player_transform.translation;
            if player.shoot_cooldown.finished() {
                let mut vel = dir.clone();
                vel.z = 0.;
                vel = vel.clamp_length(BULLET_SPEED, BULLET_SPEED);

                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.89, 0.39, 0.95),
                            custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_xyz(player_pos.x, player_pos.y, LAYER_BULLET),
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
    }
}

fn get_push_team(role: &AiRole) -> i32 {
    match role {
        AiRole::Virt1 | AiRole::Herald1 => 0,
        AiRole::Virt2 | AiRole::Herald2 => 1,
        AiRole::Ham1 | AiRole::Ham2 | AiRole::Dps1 | AiRole::Dps2 | AiRole::Dps3 | AiRole::Dps4 => {
            2
        }
    }
}

const HOME: Vec3 = Vec3::new(
    (MAP_RADIUS - PLAYER_RADIUS * 2.) * 0.1,
    (MAP_RADIUS - PLAYER_RADIUS * 2.) * 0.995,
    0.,
);

fn think_go_home(player_pos: Vec3) -> Thought {
    if collide(player_pos, PLAYER_RADIUS * 3., HOME, 0.) {
        return Thought::REST;
    }

    Thought {
        utility: 0.2,
        action: Action::Move(HOME),
    }
}

fn think_avoid_soups(
    player_pos: Vec3,
    soups: &Query<(&Soup, &Transform, &CollisionRadius), Without<Player>>,
) -> Thought {
    for (soup, transform_soup, radius) in soups {
        if soup.damage < 0.1 {
            continue;
        }
        let soup_pos = transform_soup.translation;
        if !collide(player_pos, 0., soup_pos, radius.0 + PLAYER_RADIUS * 1.3) {
            continue;
        }

        let diff = soup_pos.sub(player_pos);
        return Thought {
            utility: 0.95,
            action: Action::Move(player_pos.add(diff.mul(-1.))),
        };
    }

    Thought::REST
}

pub fn player_ai_purification_phase_system(
    time: Res<Time>,
    game: Res<Game>,
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player, &AiPlayer, &mut Transform)>,
    enemies: Query<(&Enemy, &Transform), Without<Player>>,
    orb: Query<(&MobOrb, &Transform, &Velocity), Without<Player>>,
    orb_targets: Query<(&OrbTarget, &Transform), Without<Player>>,
    soups: Query<(&Soup, &Transform, &CollisionRadius), Without<Player>>,
) {
    let (_, orb_transform, orb_velocity) = orb.single();
    let orb_pos = orb_transform.translation;

    for (entity_player, mut player, ai_player, mut transform) in &mut players {
        let player_pos = transform.translation;
        let player_orb_team = get_push_team(&ai_player.role);

        let mut orb_target_pos = None;
        let mut orb_dest_pos = None;
        let mut orb_index = 9999;
        for (orb_target, orb_target_transform) in &orb_targets {
            if orb_target.0 < game.orb_target {
                continue;
            }
            if orb_target.0 == game.orb_target {
                orb_dest_pos = Some(orb_target_transform.translation);
            }
            if orb_target.0 % 2 != player_orb_team {
                continue;
            }
            if orb_index < orb_target.0 {
                continue;
            }
            orb_target_pos = Some(orb_target_transform.translation);
            orb_index = orb_target.0;
        }

        let is_active = orb_index == game.orb_target;

        let mut thoughts: Vec<Thought> = vec![
            think_dont_fall_off_edge(&player_pos),
            think_shoot_crab(player_pos, orb_pos, &enemies),
            think_avoid_soups(player_pos, &soups),
        ];

        if let (Some(orb_target_pos), Some(orb_dest_pos)) = (orb_target_pos, orb_dest_pos) {
            thoughts.push(think_push_orb(
                player_pos,
                orb_pos,
                orb_velocity,
                orb_target_pos,
                orb_dest_pos,
                is_active,
            ));
        }

        let best_thought = thoughts
            .iter()
            .reduce(|a, b| if a.utility > b.utility { a } else { b })
            .unwrap_or(&Thought::REST);

        act_on_thought(
            best_thought,
            &time,
            &mut commands,
            &mut player,
            entity_player,
            &mut transform,
        );
    }
}
