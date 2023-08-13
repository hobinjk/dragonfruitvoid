use bevy::{
    prelude::*,
    render::color::Color,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::ops::{Add, Mul, Sub};

use crate::game::*;
use crate::aoes::*;
use crate::mobs::*;
use crate::collisions::*;
use crate::orbs::*;
use crate::phase::*;

const CRAB_SPEED: f32 = 15.;

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

fn void_zone_crab_system(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut void_zone_crab_spawns: Query<&mut VoidZoneCrabSpawn>,
    void_zones: Query<(&VoidZone, &Transform)>
    ) {
    let void_zone_crab_spawn = &mut void_zone_crab_spawns.single_mut().0;
    void_zone_crab_spawn.tick(time.delta());

    if !void_zone_crab_spawn.just_finished() {
        return;
    }

    for (_, transform) in &void_zones {
        let mut pos = transform.translation.clone();
        pos.z = LAYER_MOB;

        spawn_crab(&mut commands, &asset_server, pos);
    }
}

pub fn add_update_purification_phase_set(app: &mut App) {
    app.add_systems(PhaseSet::UpdatePurificationPhase, (
        move_crabs_system,
        collisions_crabs_orbs_system,
        collisions_enemies_orbs_system,
        collisions_bullets_orbs_system,
        collisions_orb_targets_system,
        collisions_orbs_edge_system,
        game_orb_target_progression_system,
        void_zone_crab_system,
    ));
}

pub fn setup_purification(
    mut commands: Commands, mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    game.orb_target = 0;

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(ORB_RADIUS).into()).into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.9, 1.0, 1.0))),
        transform: Transform::from_xyz(0., 0., LAYER_MOB),
        ..default()
    }).insert(MobOrb).insert(Velocity(Vec3::new(0., 0., 0.)));

    let void_zone_offset = 420.;
    let void_zone_positions = [
        Vec3::new(-void_zone_offset, 0., LAYER_VOID),
        Vec3::new(void_zone_offset, 0., LAYER_VOID),
        Vec3::new(0., -void_zone_offset, LAYER_VOID),
        Vec3::new(0., void_zone_offset, LAYER_VOID),
    ];

    let void_zone_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(VOID_ZONE_START_RADIUS).into()).into();
    let void_zone_material = ColorMaterial::from(Color::rgba(0.0, 0.0, 0.0, 0.9));

    for pos in void_zone_positions {
        commands.spawn(MaterialMesh2dBundle {
            mesh: void_zone_mesh.clone(),
            material: materials.add(void_zone_material.clone()),
            transform: Transform::from_translation(pos),
            ..default()
        })
        .insert(VoidZone)
        .insert(CollisionRadius(VOID_ZONE_START_RADIUS))
        .insert(Soup {
            damage: 25.,
            duration: None,
        });
    }
}
