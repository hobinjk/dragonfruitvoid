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
    collide, Aoe, Boss, Bullet, CollisionRadius, Game, GameState, HasHit, MobOrb, OrbTarget,
    PhaseEntity, Soup, StackGreenIndicator, Velocity, VoidZone, Wave, BULLET_SIZE, BULLET_SPEED,
    GAME_TO_PX, HEIGHT, JUMP_DURATION_S, LAYER_BULLET, MAP_RADIUS, PLAYER_RADIUS, WAVE_MAX_RADIUS,
};

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
enum Action {
    Move(Vec3),
    Shoot(Vec3),
    Jump,
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

fn think_jump_wave(
    player: (&Player, &Transform),
    waves: &Query<(&Wave, &Visibility, &Transform), Without<Player>>,
) -> Thought {
    let (player, transform_player) = player;
    let player_pos = transform_player.translation;

    for (_, visibility, transform) in waves {
        if visibility == Visibility::Hidden {
            continue;
        }

        let r_outer = transform.scale.x * WAVE_MAX_RADIUS + PLAYER_RADIUS * 2.;
        let r_inner = r_outer - 20.;

        // Safe because we're in the "eye" of the wave
        if collide(player_pos, 0., transform.translation, r_inner) {
            continue;
        }
        if collide(player_pos, 0., transform.translation, r_outer) {
            if player.jump_cooldown.finished() {
                return Thought {
                    utility: 1.0,
                    action: Action::Jump,
                };
            }
        }
    }

    Thought::REST
}

fn is_safe_for_orb(player_pos: Vec3, orb_pos: Vec3, enemy_pos: Vec3) -> bool {
    let shoot_dir = enemy_pos.sub(player_pos).truncate();
    let orb_dir = orb_pos.sub(player_pos).truncate();

    // Inside the orb, hitting no matter what
    if orb_dir.length_squared() < (ORB_RADIUS * 1.1) * (ORB_RADIUS * 1.1) {
        return false;
    }

    let angle_orb = (ORB_RADIUS * 3.5)
        .atan2(orb_dir.length())
        .clamp(0., PI / 2.);
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
    let mut unsafe_enemy_pos = None;
    for (_enemy, transform_enemy) in enemies {
        let enemy_pos: Vec3 = transform_enemy.translation;
        if !is_safe_for_orb(player_pos, orb_pos, enemy_pos) {
            unsafe_enemy_pos = Some(enemy_pos);
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

    let fallback_thought = match unsafe_enemy_pos {
        None => Thought::REST,
        Some(unsafe_enemy_pos) => Thought {
            utility: 0.1,
            action: Action::Move(unsafe_enemy_pos),
        },
    };

    match closest_enemy {
        None => fallback_thought,
        Some((_, closest_pos)) => Thought {
            utility: 0.4,
            action: Action::Shoot(closest_pos.sub(player_pos)),
        },
    }
}

fn think_shoot_enemy(
    player_pos: Vec3,
    enemies: &Query<(&Enemy, &Transform, &Visibility, Option<&Boss>), Without<Player>>,
) -> Thought {
    let mut closest_enemy: Option<(f32, Vec3)> = None;
    for (_enemy, transform_enemy, visibility, opt_boss) in enemies {
        if visibility == Visibility::Hidden {
            continue;
        }

        let enemy_pos: Vec3 = transform_enemy.translation;
        let mut dist_sq = enemy_pos.sub(player_pos).length_squared();
        // Prioritize boss after every other enemy
        if opt_boss.is_some() {
            dist_sq = (MAP_RADIUS * 2.) * (MAP_RADIUS * 2.);
        }
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
    game_state: Res<State<GameState>>,
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player, &AiPlayer, &mut Transform)>,
    enemies: Query<(&Enemy, &Transform, &Visibility, Option<&Boss>), Without<Player>>,
    greens: Query<(&StackGreen, &Children)>,
    indicators: Query<(&StackGreenIndicator, &Transform), Without<Player>>,
    puddle_spawns: Query<&PuddleSpawn>,
    puddles: Query<&Puddle>,
    soups: Query<(&Soup, &Transform, &CollisionRadius), Without<Player>>,
    aoes: Query<(&Aoe, &Visibility, &Transform, &CollisionRadius), Without<Player>>,
    void_zones: Query<(&CollisionRadius, &Transform), (With<VoidZone>, Without<Player>)>,
    waves: Query<(&Wave, &Visibility, &Transform), Without<Player>>,
) {
    let center_void_zone = void_zones.single();
    let (center_void_zone_radius, _) = center_void_zone;

    for (entity_player, mut player, ai_player, mut transform) in &mut players {
        let player_pos = transform.translation;

        let thoughts: Vec<Thought> = vec![
            think_dont_fall_off_edge(&player_pos),
            think_shoot_enemy(player_pos, &enemies),
            think_avoid_soups(player_pos, &soups),
            think_do_greens(game_state.get(), &ai_player.role, &greens, &indicators),
            think_do_puddles(
                player_pos,
                &ai_player.role,
                &puddle_spawns,
                &puddles,
                &void_zones,
            ),
            think_go_home(game_state.get(), &ai_player.role, player_pos),
            think_avoid_aoes(player_pos, &aoes),
            think_jump_wave((&player, &transform), &waves),
        ];

        act_on_thoughts(
            &thoughts,
            &time,
            &mut commands,
            &mut player,
            &ai_player.role,
            entity_player,
            &mut transform,
            Some(center_void_zone_radius.0),
        );
    }
}

fn think_do_greens(
    game_state: &GameState,
    role: &AiRole,
    greens: &Query<(&StackGreen, &Children)>,
    indicators: &Query<(&StackGreenIndicator, &Transform), Without<Player>>,
) -> Thought {
    let no_green: usize = 9001;
    let green_team: usize = match game_state {
        GameState::SooWonOne | GameState::SooWonTwo => match role {
            AiRole::Dps1 | AiRole::Dps2 => no_green,
            AiRole::Dps3 | AiRole::Dps4 => 2,
            AiRole::Virt1 | AiRole::Virt2 => 0,
            AiRole::Herald1 | AiRole::Herald2 => 1,
            AiRole::Ham1 | AiRole::Ham2 => no_green,
        },
        // This would be more accurate but is hard to pull off for the hams
        // GameState::SooWonTwo => match role {
        //     AiRole::Dps1 | AiRole::Dps2 | AiRole::Dps3 | AiRole::Dps4 => no_green,
        //     AiRole::Virt1 | AiRole::Virt2 => 1,
        //     AiRole::Herald1 | AiRole::Herald2 => 2,
        //     AiRole::Ham1 | AiRole::Ham2 => 0,
        // },
        _ => match role {
            AiRole::Dps1 | AiRole::Dps2 | AiRole::Dps3 | AiRole::Dps4 => no_green,
            AiRole::Virt1 | AiRole::Virt2 => 0,
            AiRole::Herald1 | AiRole::Herald2 => 1,
            AiRole::Ham1 | AiRole::Ham2 => 2,
        },
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
    void_zones: &Query<(&CollisionRadius, &Transform), (With<VoidZone>, Without<Player>)>,
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
                utility: 0.97, // even more important than greens
                action: Action::Move(target_pos),
            };
        }
    }

    let center_void_zone = void_zones.single();
    let (center_void_zone_radius, _) = center_void_zone;

    for puddle_spawn in puddle_spawns {
        if puddle_spawn.visibility_start.remaining_secs() > 6.
            || puddle_spawn.visibility_start.finished()
        {
            continue;
        }

        // Be as close to the center as possible while rotating to the back
        let r = center_void_zone_radius.0 + PLAYER_RADIUS * 1.4;
        let mut theta = player_pos.x.atan2(player_pos.y);
        if theta < 0. {
            theta -= 0.2;
        } else {
            theta += 0.2;
        }
        theta = theta.clamp(-PI, PI);
        let target_pos = Vec3::new(r * theta.sin(), r * theta.cos(), 0.);

        let mut utility = 0.4;
        if player_pos.length_squared() < (r * 1.5) * (r * 1.5) {
            utility = 0.2;
        }
        return Thought {
            utility,
            action: Action::Move(target_pos),
        };
    }

    Thought::REST
}

fn think_avoid_aoes(
    player_pos: Vec3,
    aoes: &Query<(&Aoe, &Visibility, &Transform, &CollisionRadius), Without<Player>>,
) -> Thought {
    let mut avg_overlapping_aoe_pos = Vec3::ZERO;
    let mut n_overlapping = 0.;

    for (aoe, visibility, transform, radius) in aoes {
        let big_and_about_to_happen = radius.0 > 300.
            && if let Some(vis_start) = &aoe.visibility_start {
                vis_start.remaining_secs() < 3.
            } else {
                false
            };
        if visibility == Visibility::Hidden && !big_and_about_to_happen {
            continue;
        }

        let mut aoe_pos = transform.translation;
        // Zhaitan map covering aoe is big and centered
        if radius.0 > MAP_RADIUS - PLAYER_RADIUS && aoe_pos.truncate().length_squared() < 1. {
            continue;
        }

        if !collide(aoe_pos, radius.0, player_pos, PLAYER_RADIUS) {
            continue;
        }

        // Special-case primordus chomps
        let diff = aoe_pos.sub(player_pos);
        let target_pos = player_pos.add(diff.mul(-1.));
        if target_pos.length_squared() > MAP_RADIUS * MAP_RADIUS {
            aoe_pos.y = HEIGHT / 2.;
        }

        let scale_factor = if radius.0 > 300. { 5. } else { 1. };
        avg_overlapping_aoe_pos = avg_overlapping_aoe_pos.add(aoe_pos.mul(scale_factor));
        n_overlapping += scale_factor;
    }

    if n_overlapping < 0.01 {
        return Thought::REST;
    }

    let diff = avg_overlapping_aoe_pos
        .mul(1. / (n_overlapping as f32))
        .sub(player_pos);

    let utility = if (n_overlapping - 5.).abs() < 0.1 {
        0.2
    } else {
        0.7
    };
    Thought {
        utility,
        action: Action::Move(player_pos.add(diff.mul(-1.))),
    }
}

fn act_on_thoughts(
    thoughts: &Vec<Thought>,
    time: &Res<Time>,
    mut commands: &mut Commands,
    mut player: &mut Player,
    role: &AiRole,
    entity_player: Entity,
    mut player_transform: &mut Transform,
    center_void_zone_radius: Option<f32>,
) {
    let best_not_shoot_thought = thoughts
        .iter()
        .filter(|a| match a.action {
            Action::Shoot(_) => false,
            _ => true,
        })
        .reduce(|a, b| if a.utility > b.utility { a } else { b })
        .unwrap_or(&Thought::REST);

    act_on_thought(
        best_not_shoot_thought,
        &time,
        &mut commands,
        &mut player,
        &role,
        entity_player,
        &mut player_transform,
        center_void_zone_radius,
    );

    let best_shoot_thought = thoughts
        .iter()
        .filter(|a| match a.action {
            Action::Shoot(_) => true,
            _ => false,
        })
        .reduce(|a, b| if a.utility > b.utility { a } else { b })
        .unwrap_or(&Thought::REST);

    act_on_thought(
        best_shoot_thought,
        &time,
        &mut commands,
        &mut player,
        &role,
        entity_player,
        &mut player_transform,
        center_void_zone_radius,
    );
}

fn act_on_thought(
    thought: &Thought,
    time: &Res<Time>,
    commands: &mut Commands,
    player: &mut Player,
    role: &AiRole,
    entity_player: Entity,
    player_transform: &mut Transform,
    center_void_zone_radius: Option<f32>,
) {
    let speed = 250.0 * GAME_TO_PX * time.delta_seconds();
    let safe_margin = match role {
        AiRole::Ham1 | AiRole::Ham2 => 1.1,
        _ => 3.,
    };

    match thought.action {
        Action::Rest => {}
        Action::Move(target_pos) => {
            let safe_map_radius = MAP_RADIUS - PLAYER_RADIUS * 1.2;
            let safe_inner_radius = if let Some(center_void_zone_radius) = center_void_zone_radius {
                center_void_zone_radius + PLAYER_RADIUS * safe_margin
            } else {
                0.
            };

            let player_pos = player_transform.translation.truncate();

            let movement = target_pos
                .sub(player_transform.translation)
                .truncate()
                .clamp_length(0., speed);
            let unsafe_translation = player_pos.add(movement);
            let safe_translation =
                unsafe_translation.clamp_length(safe_inner_radius, safe_map_radius);

            let mut safe_movement = safe_translation.sub(player_pos);

            if safe_movement.length_squared() < movement.length_squared() {
                safe_movement = safe_movement.clamp_length_min(movement.length());
            }
            player_transform.translation =
                player_transform.translation.add(safe_movement.extend(0.));
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
        Action::Jump => {
            if player.jump_cooldown.finished() {
                player.jump = Timer::from_seconds(JUMP_DURATION_S, TimerMode::Once);
                player.jump_cooldown.reset();
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
    (MAP_RADIUS - PLAYER_RADIUS * 1.3) * 0.1,
    (MAP_RADIUS - PLAYER_RADIUS * 1.3) * 0.995,
    0.,
);
const HOME_PRIMORDUS: Vec3 = Vec3::new(
    (MAP_RADIUS - PLAYER_RADIUS * 1.3) * -0.24,
    (MAP_RADIUS - PLAYER_RADIUS * 1.3) * 0.97,
    0.,
);

fn home_for_role(game_state: &GameState, role: &AiRole) -> Vec3 {
    let home = match game_state {
        GameState::Primordus => match role {
            AiRole::Ham1 | AiRole::Ham2 => HOME,
            _ => HOME_PRIMORDUS,
        },
        _ => HOME,
    };
    let offset = (*role as i32) as f32;
    home.add(Vec3::new(
        (offset % 3.) * PLAYER_RADIUS,
        -((offset / 3.) % 4.) * PLAYER_RADIUS,
        0.,
    ))
}

fn think_go_home(game_state: &GameState, role: &AiRole, player_pos: Vec3) -> Thought {
    let home = home_for_role(game_state, role);
    if collide(player_pos, PLAYER_RADIUS * 2., home, 0.) {
        return Thought::REST;
    }

    Thought {
        utility: 0.05,
        action: Action::Move(home),
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
        if !collide(player_pos, 0., soup_pos, radius.0 + PLAYER_RADIUS / 4.) {
            continue;
        }

        let diff = soup_pos.sub(player_pos);
        let utility = if soup.damage < 19. { 0.3 } else { 0.98 };
        return Thought {
            utility,
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

        act_on_thoughts(
            &thoughts,
            &time,
            &mut commands,
            &mut player,
            &ai_player.role,
            entity_player,
            &mut transform,
            None,
        );
    }
}
