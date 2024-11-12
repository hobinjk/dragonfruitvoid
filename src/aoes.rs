use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::ops::Add;

use crate::collisions::{collide, CollisionRadius};
use crate::damage_flash::DamageFlashEvent;
use crate::game::{PhaseEntity, Player, GAME_RADIUS, GAME_TO_PX, LAYER_AOE};

pub const AOE_BASE_COLOR: Color = Color::rgba(0.9, 0.9, 0., 0.4);
pub const AOE_DETONATION_COLOR: Color = Color::rgba(0.7, 0., 0., 0.7);

pub const SPEW_DAMAGE: f32 = 40.;
pub const SPEW_RADIUS: f32 = 220. * GAME_TO_PX;
const SPEW_SPACING: f32 = 30. * GAME_TO_PX;
const SPEW_DYDX: f32 = -0.3;

#[derive(Component)]
pub struct Aoe {
    pub visibility_start: Option<Timer>,
    pub detonation: Timer,
    pub linger: Option<Timer>,
    pub damage: f32,
}

#[derive(Component)]
pub struct AoeFollow {
    pub target: Entity,
}

#[derive(Component)]
pub struct AoeIndicator;

#[derive(Clone)]
pub struct AoeDesc {
    pub mesh: Mesh2dHandle,
    pub radius: f32,
    pub material_base: Handle<ColorMaterial>,
    pub material_detonation: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct Soup {
    pub damage: f32,
    pub duration: Option<Timer>,
}

pub fn aoes_system(
    time: Res<Time>,
    mut aoes: Query<(&mut Aoe, &mut Visibility, &Children)>,
    mut indicators: Query<(&AoeIndicator, &mut Transform)>,
) {
    for (mut aoe, mut visibility, children) in &mut aoes {
        let mut visible = Visibility::Hidden;
        match &mut aoe.visibility_start {
            Some(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    visible = Visibility::Inherited;
                }
            }
            None => {
                visible = Visibility::Inherited;
            }
        }
        *visibility = visible;

        if visible == Visibility::Hidden {
            continue;
        }

        aoe.detonation.tick(time.delta());

        let det_scale = aoe.detonation.percent();

        for &child in children.iter() {
            if let Ok((_, mut transform_indicator)) = indicators.get_mut(child) {
                transform_indicator.scale = Vec3::splat(det_scale);
            }
        }
    }
}

pub fn aoes_detonation_system(
    mut commands: Commands,
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut players: Query<(Entity, &Transform, &mut Player)>,
    aoes: Query<(Entity, &Aoe, &Transform, &CollisionRadius)>,
) {
    for (entity_aoe, aoe, transform, radius) in &aoes {
        if !aoe.detonation.just_finished() {
            continue;
        }

        for (entity_player, transform_player, mut player) in &mut players {
            let player_pos = transform_player.translation;
            let hit = collide(transform.translation, radius.0, player_pos, 0.);

            if hit {
                player.hp -= aoe.damage;
                player.damage_taken += aoe.damage;
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_player,
                });
            }
        }

        if let Some(linger) = &aoe.linger {
            commands.entity(entity_aoe).remove::<Aoe>();
            commands.entity(entity_aoe).insert(Soup {
                damage: aoe.damage / 4., // arbitrary
                duration: Some(linger.clone()),
            });
        } else {
            commands.entity(entity_aoe).despawn_recursive();
        }
    }
}

pub fn aoes_follow_system(
    transforms: Query<&Transform, Without<Aoe>>,
    mut aoes: Query<(&AoeFollow, &mut Transform), With<Aoe>>,
) {
    for (follow, mut transform) in &mut aoes {
        if let Ok(transform_target) = transforms.get(follow.target) {
            transform.translation.x = transform_target.translation.x;
            transform.translation.y = transform_target.translation.y;
        }
    }
}

pub fn soup_duration_system(
    time: Res<Time>,
    mut commands: Commands,
    mut soups: Query<(Entity, &mut Soup)>,
) {
    for (entity, mut soup) in &mut soups {
        if soup.duration.is_none() {
            continue;
        }
        if let Some(duration) = &mut soup.duration {
            duration.tick(time.delta());
            if duration.finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn spawn_aoe(
    commands: &mut Commands,
    aoe_desc: &AoeDesc,
    position: Vec3,
    aoe: Aoe,
    aoe_follow: Option<AoeFollow>,
) -> Entity {
    let id = commands
        .spawn(MaterialMesh2dBundle {
            transform: Transform::from_translation(position),
            mesh: aoe_desc.mesh.clone(),
            material: aoe_desc.material_base.clone(),
            ..default()
        })
        .with_children(|parent| {
            let position_above = Vec3::new(0., 0., 0.1);
            parent
                .spawn(MaterialMesh2dBundle {
                    mesh: aoe_desc.mesh.clone(),
                    transform: Transform::from_translation(position_above).with_scale(Vec3::ZERO),
                    material: aoe_desc.material_detonation.clone(),
                    ..default()
                })
                .insert(AoeIndicator);
        })
        .insert(aoe)
        .insert(CollisionRadius(aoe_desc.radius))
        .insert(PhaseEntity)
        .id();

    if let Some(aoe_follow) = aoe_follow {
        commands.entity(id).insert(aoe_follow);
    }

    id
}

pub fn spawn_spew_aoe(
    commands: &mut Commands,
    start: f32,
    detonation: f32,
    aoe_desc: &AoeDesc,
    linger: Option<Timer>,
) {
    let rotation = Vec2::new(SPEW_DYDX.cos(), SPEW_DYDX.sin());

    for row in -6..=6 {
        let y = row as f32 * (SPEW_RADIUS * 2. + SPEW_SPACING);
        for col in -6..=6 {
            let x = col as f32 * (SPEW_RADIUS * 2. + SPEW_SPACING);
            let dist = (x * x + y * y).sqrt();
            if dist > GAME_RADIUS {
                continue;
            }

            // Rotation and offset are both pretty arbitrary
            let pos2 = Vec2::new(x, y)
                .rotate(rotation)
                .add(Vec2::new(SPEW_RADIUS * 0.7, SPEW_RADIUS * 0.1));
            let dy = pos2.y - GAME_RADIUS;
            let dist = (pos2.x * pos2.x + dy * dy).sqrt();
            let aoe_delay = dist / 2000.;

            let aoe = Aoe {
                visibility_start: Some(Timer::from_seconds(start + aoe_delay, TimerMode::Once)),
                // detonation: Timer::from_seconds(1.5, TimerMode::Once),
                detonation: Timer::from_seconds(detonation, TimerMode::Once),
                damage: SPEW_DAMAGE,
                linger: linger.clone(),
            };

            spawn_aoe(
                commands,
                aoe_desc,
                Vec3::new(pos2.x, pos2.y, LAYER_AOE),
                aoe,
                None,
            );
        }
    }
}
