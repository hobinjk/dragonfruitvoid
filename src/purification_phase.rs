use bevy::prelude::*;
use std::ops::{Add, Mul, Sub};

use crate::aoes::*;
use crate::collisions::*;
use crate::mobs::*;
use crate::orbs::*;
use crate::phase::*;
use crate::{ai::player_ai_purification_phase_system, game::*};

const CRAB_SPEED: f32 = 15.;

fn move_crabs_system(
    time: Res<Time>,
    mut crabs: Query<&mut Transform, (With<MobCrab>, Without<EffectForcedMarch>)>,
    orb: Query<(&MobOrb, &Transform), Without<MobCrab>>,
) {
    for mut transform in &mut crabs {
        let (_, orb_transform) = orb.single();
        let vel = orb_transform.translation.sub(transform.translation);
        transform.translation = transform
            .translation
            .add(vel.mul(CRAB_SPEED / vel.length()).mul(time.delta_seconds()));
    }
}

fn game_orb_target_progression_system(
    game: ResMut<Game>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    orb_targets: Query<(&OrbTarget, &mut MeshMaterial2d<ColorMaterial>)>,
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
    void_zones: Query<(&VoidZone, &Transform)>,
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
    app.add_systems(
        Update,
        (
            move_crabs_system,
            collisions_crabs_orbs_system,
            collisions_enemies_orbs_system,
            collisions_bullets_orbs_system,
            collisions_orb_targets_system,
            collisions_orbs_edge_system,
            game_orb_target_progression_system,
            void_zone_crab_system,
            player_ai_purification_phase_system,
        )
            .in_set(PhaseSet::UpdatePurificationPhase),
    );
}

pub fn setup_purification(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    game.orb_target = 0;

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(ORB_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.9, 1.0, 1.0)))),
        Transform::from_xyz(0., 0., LAYER_MOB),
        MobOrb,
        Velocity(Vec3::new(0., 0., 0.)),
        PhaseEntity,
    ));

    let void_zone_offset = 420.;
    let void_zone_positions = [
        Vec3::new(-void_zone_offset, 0., LAYER_VOID),
        Vec3::new(void_zone_offset, 0., LAYER_VOID),
        Vec3::new(0., -void_zone_offset, LAYER_VOID),
        Vec3::new(0., void_zone_offset, LAYER_VOID),
    ];

    let void_zone_mesh: Handle<Mesh> = meshes.add(Circle::new(VOID_ZONE_START_RADIUS));
    let void_zone_material = ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 0.9));

    for pos in void_zone_positions {
        commands.spawn((
            Mesh2d(void_zone_mesh.clone()),
            MeshMaterial2d(materials.add(void_zone_material.clone())),
            Transform::from_translation(pos),
            VoidZone,
            CollisionRadius(VOID_ZONE_START_RADIUS),
            Soup {
                damage: 25.,
                duration: None,
            },
            PhaseEntity,
        ));
    }
}
