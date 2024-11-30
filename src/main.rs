use ai::{player_ai_purification_phase_system, AiRole};
use bevy::render::camera::CameraProjection;
use bevy::{prelude::*, sprite::Anchor, time::Stopwatch, window::WindowResolution};
use core::f32::consts::PI;
use loading::{setup_loading_system, update_loading_system, AssetsLoading};
use rand::Rng;
use std::ops::Add;
use std::time::Duration;

mod ai;
mod aoes;
mod audio;
mod boss_phase;
mod collisions;
mod damage_flash;
mod game;
mod greens;
mod hints;
mod loading;
mod menu;
mod mobs;
mod orbs;
mod phase;
mod purification_phase;
mod ui;
mod waves;

use crate::aoes::*;
use crate::boss_phase::*;
use crate::collisions::*;
use crate::damage_flash::*;
use crate::game::*;
use crate::greens::*;
use crate::menu::*;
use crate::mobs::*;
use crate::orbs::*;
use crate::phase::*;
use crate::purification_phase::*;
use crate::ui::*;
use crate::waves::*;

#[derive(Component)]
struct OhNoNotTheBees {
    bees_cooldown: Timer,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

const CHOMP_TARGET_Y: f32 = 1024. - 380.;
const MINICHOMP_TARGET_Y: f32 = 380.;

const BEE_SPEED: f32 = 50.;

const ROTATING_SOUP_RADIUS: f32 = 40.;
const ROTATING_SOUP_DTHETA: f32 = 0.2;

const LASER_RADIUS: f32 = 25.;

const SWIPE_CHONK_RADIUS: f32 = 650. * GAME_TO_PX;
const SWIPE_CENTER: Vec3 = Vec3::new(-428. * GAME_TO_PX, 1061. * GAME_TO_PX, LAYER_WAVE);
// not quiiite correct-looking
// const SWIPE_START_THETA: f32 = PI + 0.2;
// const SWIPE_END_THETA: f32 = 2. * PI - 0.973;
const SWIPE_START_THETA: f32 = PI + 0.05;
const SWIPE_END_THETA: f32 = 2. * PI - 0.8;
const SWIPE_BALL_RADIUS: f32 = 120. * GAME_TO_PX;
const SWIPE_BALL_OFFSET: f32 = SWIPE_CHONK_RADIUS + SWIPE_BALL_RADIUS * 2.;
const SWIPE_BALL_BOUNCE_COUNT: i32 = 4;
const SWIPE_BALL_COUNT: usize = 8;
const SWIPE_DETONATION: f32 = 2.;
const SWIPE_DAMAGE: f32 = 40.;

fn setup(mut commands: Commands, mut players: Query<&mut Player>) {
    let projection = OrthographicProjection {
        far: LAYER_MAX,
        ..OrthographicProjection::default_2d()
    };
    let transform = Transform::from_xyz(0., 0., LAYER_MAX - 0.1);
    let frustum = projection.compute_frustum(&GlobalTransform::from(transform));

    commands.spawn((Camera2d, projection, transform, frustum));

    for mut player in &mut players {
        player.dodge_cooldown.tick(Duration::from_secs_f32(1000.));
        player.blink_cooldown.tick(Duration::from_secs_f32(1000.));
        player.portal_cooldown.tick(Duration::from_secs_f32(1000.));
        player.pull_cooldown.tick(Duration::from_secs_f32(1000.));
        player.invuln.tick(Duration::from_secs_f32(1000.));
        player.jump.tick(Duration::from_secs_f32(1000.));
    }
}

fn setup_purification_one(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let crab_positions = vec![
        Vec3::new(-350., 200., LAYER_MOB),
        Vec3::new(-312.5, 237.5, LAYER_MOB),
        Vec3::new(-275., 275., LAYER_MOB),
        Vec3::new(-237.5, 312.5, LAYER_MOB),
        Vec3::new(-200., 350., LAYER_MOB),
    ];

    for crab_pos in crab_positions {
        spawn_crab(&mut commands, &asset_server, crab_pos);
    }

    let orb_target_mesh: Handle<Mesh> = meshes.add(Circle::new(ORB_TARGET_RADIUS));
    let orb_target_material = ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5));

    commands.spawn((
        Mesh2d(orb_target_mesh.clone()),
        MeshMaterial2d(materials.add(orb_target_material.clone())),
        Transform::from_xyz(-240., 240., LAYER_TARGET),
        OrbTarget(0),
        PhaseEntity,
    ));

    commands.spawn((
        Mesh2d(orb_target_mesh.clone()),
        MeshMaterial2d(materials.add(orb_target_material.clone())),
        Transform::from_xyz(-240., -240., LAYER_TARGET),
        OrbTarget(1),
        PhaseEntity,
    ));

    commands.spawn((
        Mesh2d(orb_target_mesh.clone()),
        MeshMaterial2d(materials.add(orb_target_material.clone())),
        Transform::from_xyz(240., -240., LAYER_TARGET),
        OrbTarget(2),
        PhaseEntity,
    ));
}

fn setup_purification_two(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bee_mesh: Handle<Mesh> = meshes.add(Circle::new(ORB_RADIUS));
    let bee_material = materials.add(ColorMaterial::from(Color::srgba(0.9, 0.0, 0.0, 0.7)));

    commands
        .spawn(OhNoNotTheBees {
            bees_cooldown: Timer::from_seconds(5., TimerMode::Once),
            mesh: bee_mesh,
            material: bee_material,
        })
        .insert(PhaseEntity);

    let crab_positions = vec![
        // Vec3::new(-350., 200., LAYER_MOB),
        Vec3::new(-312.5, 237.5, LAYER_MOB),
        Vec3::new(-275., 275., LAYER_MOB),
        Vec3::new(-237.5, 312.5, LAYER_MOB),
        Vec3::new(-200., 350., LAYER_MOB),
        Vec3::new(-275., -275., LAYER_MOB),
    ];

    for crab_pos in crab_positions {
        spawn_crab(&mut commands, &asset_server, crab_pos);
    }

    let orb_target_mesh: Handle<Mesh> = meshes.add(Circle::new(ORB_TARGET_RADIUS));
    let orb_target_material = ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5));

    commands.spawn((
        Mesh2d(orb_target_mesh.clone()),
        MeshMaterial2d(materials.add(orb_target_material.clone())),
        Transform::from_xyz(-240., 240., LAYER_TARGET),
        OrbTarget(0),
        PhaseEntity,
    ));

    commands.spawn((
        Mesh2d(orb_target_mesh.clone()),
        MeshMaterial2d(materials.add(orb_target_material.clone())),
        Transform::from_xyz(240., -240., LAYER_TARGET),
        OrbTarget(1),
        PhaseEntity,
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("timecaster.png"),
            custom_size: Some(Vec2::new(BIGBOY_RADIUS * 2., BIGBOY_RADIUS * 2.)),
            ..default()
        },
        Transform::from_xyz(150., -150., LAYER_MOB),
        MobTimeCaster {
            shoot_cooldown: Timer::from_seconds(0.5, TimerMode::Repeating),
        },
        Enemy,
        Hp(10.),
        CollisionRadius(BIGBOY_RADIUS),
        PhaseEntity,
    ));
}

fn unleash_the_bees(
    time: Res<Time>,
    game: Res<Game>,
    mut commands: Commands,
    orb: Query<&Transform, With<MobOrb>>,
    mut onntb: Query<&mut OhNoNotTheBees>,
) {
    if orb.is_empty() || onntb.is_empty() {
        return;
    }

    let bee_speed = if game.orb_target == 0 { BEE_SPEED } else { 0. };

    let transform_orb = orb.single();

    let mut bees = onntb.single_mut();

    bees.bees_cooldown.tick(time.delta());
    if game.orb_target > 0 {
        // Double bee rate when they're stationary
        bees.bees_cooldown.tick(time.delta());
    }
    if !bees.bees_cooldown.finished() {
        return;
    }
    bees.bees_cooldown.reset();

    let dir = rand::thread_rng().gen_range(0..8);
    let theta = (dir as f32) / 4. * PI;
    let vel = Vec3::new(theta.cos() * bee_speed, theta.sin() * bee_speed, 0.);
    let orb_pos = transform_orb.translation;
    let pos = Vec3::new(orb_pos.x, orb_pos.y, LAYER_AOE);

    commands.spawn((
        Mesh2d(bees.mesh.clone()),
        MeshMaterial2d(bees.material.clone()),
        Transform::from_translation(pos),
        CollisionRadius(ORB_RADIUS),
        Velocity(vel),
        Soup {
            damage: 25.,
            duration: None,
        },
        PhaseEntity,
    ));
}

fn setup_purification_three(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let crab_positions = vec![
        // Vec3::new(-350., 200., LAYER_MOB),
        Vec3::new(-312.5, 237.5, LAYER_MOB),
        Vec3::new(-275., 275., LAYER_MOB),
        Vec3::new(-237.5, 312.5, LAYER_MOB),
        Vec3::new(-200., 350., LAYER_MOB),
        Vec3::new(-275., -275., LAYER_MOB),
    ];

    for crab_pos in crab_positions {
        spawn_crab(&mut commands, &asset_server, crab_pos);
    }

    let laser_mesh: Handle<Mesh> = meshes.add(Circle::new(LASER_RADIUS));
    let laser_material = materials.add(ColorMaterial::from(Color::srgba(0.7, 0.9, 1.0, 0.5)));
    let material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));

    let orb_target_mesh: Handle<Mesh> = meshes.add(Circle::new(ORB_TARGET_RADIUS));
    let orb_target_material = ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5));

    commands.spawn((
        Mesh2d(orb_target_mesh.clone()),
        MeshMaterial2d(materials.add(orb_target_material.clone())),
        Transform::from_xyz(-240., 240., LAYER_TARGET),
        OrbTarget(0),
        PhaseEntity,
    ));

    commands.spawn((
        Mesh2d(orb_target_mesh.clone()),
        MeshMaterial2d(materials.add(orb_target_material.clone())),
        Transform::from_xyz(-240., -240., LAYER_TARGET),
        OrbTarget(1),
        PhaseEntity,
    ));

    let mut shoot_cooldown = Timer::from_seconds(6., TimerMode::Once);
    shoot_cooldown.tick(Duration::from_secs_f32(3.));

    commands.spawn((
        Sprite {
            image: asset_server.load("saltspray.png"),
            custom_size: Some(Vec2::new(BIGBOY_RADIUS * 2., BIGBOY_RADIUS * 2.)),
            ..default()
        },
        Transform::from_xyz(-150., 150., LAYER_MOB),
        MobSaltspray {
            shoot_cooldown,
            aoe_desc: AoeDesc {
                mesh: laser_mesh,
                radius: LASER_RADIUS,
                material_base: laser_material,
                material_detonation,
            },
        },
        Enemy,
        Hp(15.),
        CollisionRadius(BIGBOY_RADIUS),
        PhaseEntity,
    ));
}

fn setup_purification_four(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(ORB_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0., 0., 0.)))),
        Transform::from_xyz(0., 0., LAYER_MOB),
        MobOrb,
        Velocity(Vec3::new(0., 0., 0.)),
        Enemy,
        CollisionRadius(ORB_RADIUS),
        Hp(50.),
        Boss { max_hp: 50. },
        PhaseEntity,
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(1., 0., 0.),
            custom_size: Some(Vec2::new(256., 32.)),
            anchor: Anchor::CenterLeft,
            ..default()
        },
        Transform::from_xyz(-WIDTH / 2. + 20., -HEIGHT / 2. + 128. + 24., LAYER_UI),
        BossHealthbar,
        PhaseEntity,
    ));

    commands.spawn((
        Text::new("100"),
        TextFont {
            font: asset_server.load("trebuchet_ms.ttf"),
            font_size: 16.,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout {
            justify: JustifyText::Center,
            ..default()
        },
        Anchor::Center,
        Transform::from_xyz(
            -WIDTH / 2. + 20. + 128.,
            -HEIGHT / 2. + 128. + 24.,
            LAYER_TEXT,
        ),
        BossHealthbarText,
        PhaseEntity,
    ));

    commands.spawn((
        Text::new("Dark Orb"),
        TextFont {
            font: asset_server.load("trebuchet_ms.ttf"),
            font_size: 32.,
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.8, 0.8)),
        TextLayout::new_with_justify(JustifyText::Left),
        Anchor::BottomLeft,
        Transform::from_xyz(
            -WIDTH / 2. + 20.,
            -HEIGHT / 2. + 128. + 8. + 32. + 8.,
            LAYER_TEXT,
        ),
        PhaseEntity,
    ));
}

fn setup_claw_swipes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    claw_swipe_starts: Vec<f32>,
) {
    let chonk_mesh: Handle<Mesh> = meshes.add(Circle::new(SWIPE_CHONK_RADIUS));
    let ball_mesh: Handle<Mesh> = meshes.add(Circle::new(SWIPE_BALL_RADIUS));
    let material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));

    let aoe_desc_chonk = AoeDesc {
        mesh: chonk_mesh,
        radius: SWIPE_CHONK_RADIUS,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    let aoe_desc = AoeDesc {
        mesh: ball_mesh,
        radius: SWIPE_BALL_RADIUS,
        material_base,
        material_detonation,
    };

    for claw_swipe_start in claw_swipe_starts {
        let chonk_start = Timer::from_seconds(claw_swipe_start, TimerMode::Once);
        let chonk_pos = SWIPE_CENTER;
        spawn_aoe(
            commands,
            &aoe_desc_chonk,
            chonk_pos,
            Aoe {
                visibility_start: Some(chonk_start),
                detonation: Timer::from_seconds(SWIPE_DETONATION, TimerMode::Once),
                damage: SWIPE_DAMAGE,
                linger: None,
            },
            None,
        );

        for bounce in 0..SWIPE_BALL_BOUNCE_COUNT {
            let offset = SWIPE_BALL_OFFSET + (bounce as f32) * SWIPE_BALL_RADIUS * 3.;
            for ball_i in 0..SWIPE_BALL_COUNT {
                let percent = (ball_i as f32) / (SWIPE_BALL_COUNT as f32 - 1.);
                let theta = percent * (SWIPE_END_THETA - SWIPE_START_THETA) + SWIPE_START_THETA;
                let pos = Vec3::new(offset * -theta.cos(), offset * theta.sin(), LAYER_WAVE)
                    .add(chonk_pos);

                let timer = Timer::from_seconds(
                    claw_swipe_start + 0.6 * (bounce as f32 + 1.),
                    TimerMode::Once,
                );

                spawn_aoe(
                    commands,
                    &aoe_desc,
                    pos,
                    Aoe {
                        visibility_start: Some(timer),
                        detonation: Timer::from_seconds(SWIPE_DETONATION, TimerMode::Once),
                        damage: SWIPE_DAMAGE,
                        linger: None,
                    },
                    None,
                );
            }
        }
    }
}

fn setup_boss_phase(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    game: &Res<Game>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    boss_name: String,
    green_spawns: Vec<GreenSpawn>,
    puddle_starts: Vec<f32>,
    spread_starts: Vec<f32>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(BOSS_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgba(1.0, 0.0, 0.0, 0.5)))),
        Transform::from_xyz(0., HEIGHT / 2. + 20., LAYER_MOB),
        Boss { max_hp: 130. },
        Enemy,
        Hp(130.),
        CollisionRadius(BOSS_RADIUS),
        PhaseEntity,
    ));

    setup_greens(commands, meshes, materials, green_spawns.to_vec());

    commands.spawn((
        Sprite {
            color: Color::srgb(1., 0., 0.),
            custom_size: Some(Vec2::new(256., 32.)),
            anchor: Anchor::CenterLeft,
            ..default()
        },
        Transform::from_xyz(-WIDTH / 2. + 20., -HEIGHT / 2. + 128. + 24., LAYER_UI),
        BossHealthbar,
        PhaseEntity,
    ));

    commands.spawn((
        Text::new("100"),
        TextFont {
            font: asset_server.load("trebuchet_ms.ttf"),
            font_size: 16.,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout::new_with_justify(JustifyText::Center),
        Anchor::Center,
        Transform::from_xyz(
            -WIDTH / 2. + 20. + 128.,
            -HEIGHT / 2. + 128. + 24.,
            LAYER_TEXT,
        ),
        BossHealthbarText,
        PhaseEntity,
    ));

    commands.spawn((
        Text(boss_name),
        TextFont {
            font: asset_server.load("trebuchet_ms.ttf"),
            font_size: 32.,
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.8, 0.8)),
        TextLayout::new_with_justify(JustifyText::Left),
        Anchor::BottomLeft,
        Transform::from_xyz(
            -WIDTH / 2. + 20.,
            -HEIGHT / 2. + 128. + 8. + 32. + 8.,
            LAYER_TEXT,
        ),
        PhaseEntity,
    ));

    let void_zone_positions = [Vec3::new(0., 0., LAYER_VOID)];

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

    let puddle_mesh: Handle<Mesh> = meshes.add(Circle::new(PUDDLE_RADIUS));
    let puddle_material = ColorMaterial::from(Color::srgba(0.5, 0.0, 0.0, 0.3));

    if game.puddles_enabled {
        for puddle_start in puddle_starts {
            commands
                .spawn(PuddleSpawn {
                    mesh: puddle_mesh.clone(),
                    material: puddle_material.clone(),
                    visibility_start: Timer::from_seconds(puddle_start, TimerMode::Once),
                })
                .insert(PhaseEntity);
        }
    }

    let spread_mesh: Handle<Mesh> = meshes.add(Circle::new(SPREAD_RADIUS));
    let spread_material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let spread_material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));
    commands
        .spawn(SpreadAoeSpawn {
            timers: spread_starts
                .iter()
                .map(|start| Timer::from_seconds(*start, TimerMode::Once))
                .collect(),
            aoe_desc: AoeDesc {
                mesh: spread_mesh,
                material_base: spread_material_base,
                material_detonation: spread_material_detonation,
                radius: SPREAD_RADIUS,
            },
        })
        .insert(PhaseEntity);
}

fn setup_jormag(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puddle_starts: Vec<f32> = vec![5., 45., 85.];
    let spread_starts: Vec<f32> = vec![28., 68.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &game,
        &mut meshes,
        &mut materials,
        "Jormag".to_string(),
        GREEN_SPAWNS_JORMAG.to_vec(),
        puddle_starts,
        spread_starts,
    );

    // TODO roving frost beam things properly

    let rotating_soup_mesh: Handle<Mesh> = meshes.add(Circle::new(70.));
    let rotating_soup_material =
        materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 0.3)));

    for i in 0..4 {
        let radius = 0.;
        let theta = i as f32 * PI / 2.;
        let dtheta = 0.5;

        commands.spawn((
            Mesh2d(rotating_soup_mesh.clone()),
            MeshMaterial2d(rotating_soup_material.clone()),
            Transform::from_xyz(0., radius, LAYER_ROTATING_SOUP),
            RotatingSoup {
                radius,
                theta,
                dtheta,
            },
            CollisionRadius(70.),
            Soup {
                damage: 5.,
                duration: None,
            },
            PhaseEntity,
        ));
    }
}

fn setup_primordus(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puddle_starts: Vec<f32> = vec![17., 71.];
    let spread_starts: Vec<f32> = vec![13., 67.];
    let chomp_starts: Vec<f32> = vec![13., 67.];
    let minichomp_starts: Vec<f32> = vec![26., 39., 52., 80., 93., 106.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &game,
        &mut meshes,
        &mut materials,
        "Primordus".to_string(),
        GREEN_SPAWNS_PRIMORDUS.to_vec(),
        puddle_starts,
        spread_starts,
    );

    let chomp_y = HEIGHT / 2. - BOSS_RADIUS;
    let chomp_radius = CHOMP_TARGET_Y - BOSS_RADIUS;
    let minichomp_radius = MINICHOMP_TARGET_Y - BOSS_RADIUS;

    let chomp_mesh: Handle<Mesh> = meshes.add(Circle::new(chomp_radius));
    let minichomp_mesh: Handle<Mesh> = meshes.add(Circle::new(minichomp_radius));
    let material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));

    let aoe_desc_chomp = AoeDesc {
        mesh: chomp_mesh,
        radius: chomp_radius,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    let aoe_desc_minichomp = AoeDesc {
        mesh: minichomp_mesh,
        radius: minichomp_radius,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    for chomp_start in chomp_starts {
        spawn_aoe(
            &mut commands,
            &aoe_desc_chomp,
            Vec3::new(0., chomp_y, LAYER_AOE),
            Aoe {
                visibility_start: Some(Timer::from_seconds(chomp_start, TimerMode::Once)),
                detonation: Timer::from_seconds(7., TimerMode::Once),
                damage: 100.,
                linger: Some(Timer::from_seconds(5., TimerMode::Once)),
            },
            None,
        );
    }

    for minichomp_start in minichomp_starts {
        spawn_aoe(
            &mut commands,
            &aoe_desc_minichomp,
            Vec3::new(0., chomp_y, LAYER_AOE),
            Aoe {
                visibility_start: Some(Timer::from_seconds(minichomp_start, TimerMode::Once)),
                detonation: Timer::from_seconds(3., TimerMode::Once),
                damage: 90.,
                linger: None,
            },
            None,
        );
    }
}

fn setup_kralkatorrik(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puddle_starts: Vec<f32> = vec![18., 43., 68., 93.];
    let spread_starts: Vec<f32> = vec![];
    let line_delay: f32 = 2.;
    let line_duration: f32 = 5.;
    let double_line_starts: Vec<f32> = vec![5., 30., 54., 78., 102.];
    let mid_line_starts: Vec<f32> = vec![18., 42., 66., 90.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &game,
        &mut meshes,
        &mut materials,
        "Kralkatorrik".to_string(),
        vec![],
        puddle_starts,
        spread_starts,
    );

    let line_radius = BOSS_RADIUS * 0.9;
    let line_x = BOSS_RADIUS * 0.3 + line_radius;
    let line_spacing = line_radius;
    let line_circles = (GAME_WIDTH / line_spacing) as i32;

    let mesh: Handle<Mesh> = meshes.add(Circle::new(line_radius));
    let material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let material_detonation = materials.add(ColorMaterial::from(Color::srgb(0., 0., 0.)));

    let aoe_desc = AoeDesc {
        mesh,
        radius: line_radius,
        material_base,
        material_detonation,
    };

    for line_start in double_line_starts {
        for i in 0..line_circles {
            let delay = 0.5 - i as f32 / (2. * line_circles as f32);
            let mut pos = Vec3::new(line_x, i as f32 * line_spacing - GAME_WIDTH / 2., LAYER_AOE);

            spawn_aoe(
                &mut commands,
                &aoe_desc,
                pos,
                Aoe {
                    visibility_start: Some(Timer::from_seconds(
                        line_start + delay,
                        TimerMode::Once,
                    )),
                    detonation: Timer::from_seconds(line_delay, TimerMode::Once),
                    damage: SPEW_DAMAGE,
                    linger: Some(Timer::from_seconds(line_duration, TimerMode::Once)),
                },
                None,
            );

            pos.x *= -1.;
            spawn_aoe(
                &mut commands,
                &aoe_desc,
                pos,
                Aoe {
                    visibility_start: Some(Timer::from_seconds(
                        line_start + delay,
                        TimerMode::Once,
                    )),
                    detonation: Timer::from_seconds(line_delay, TimerMode::Once),
                    damage: SPEW_DAMAGE,
                    linger: Some(Timer::from_seconds(line_duration, TimerMode::Once)),
                },
                None,
            );
        }
    }

    for line_start in mid_line_starts {
        for i in 0..line_circles {
            let delay = i as f32 / (2. * line_circles as f32);
            let pos = Vec3::new(0., i as f32 * line_spacing - GAME_WIDTH / 2., LAYER_AOE);

            spawn_aoe(
                &mut commands,
                &aoe_desc,
                pos,
                Aoe {
                    visibility_start: Some(Timer::from_seconds(
                        line_start + delay,
                        TimerMode::Once,
                    )),
                    detonation: Timer::from_seconds(line_delay, TimerMode::Once),
                    damage: SPREAD_DAMAGE,
                    linger: Some(Timer::from_seconds(line_duration, TimerMode::Once)),
                },
                None,
            );
        }
    }
}

fn setup_mordremoth(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puddle_starts: Vec<f32> = vec![8., 30., 53., 82.];
    let spread_starts: Vec<f32> = vec![18., 40., 63., 91.];
    let boop_starts: Vec<f32> = vec![22., 44., 67., 95.];
    let boop_delays: Vec<f32> = vec![0., 2., 4.]; // 21.5, 24, 26 hmm
    let spew_starts: Vec<f32> = vec![13., 35., 58., 87.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &game,
        &mut meshes,
        &mut materials,
        "Mordremoth".to_string(),
        vec![],
        puddle_starts,
        spread_starts,
    );

    let wave_texture = asset_server.load("wave.png");

    for boop_start in &boop_starts {
        for boop_delay in &boop_delays {
            commands.spawn((
                Sprite {
                    image: wave_texture.clone(),
                    custom_size: Some(Vec2::new(WAVE_MAX_RADIUS * 2., WAVE_MAX_RADIUS * 2.)),
                    ..default()
                },
                Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
                Wave {
                    visibility_start: Timer::from_seconds(boop_start + boop_delay, TimerMode::Once),
                    ..default()
                },
                PhaseEntity,
            ));
        }
    }

    let spew_mesh: Handle<Mesh> = meshes.add(Circle::new(SPEW_RADIUS));
    let material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));

    let aoe_desc_spew = AoeDesc {
        mesh: spew_mesh,
        radius: SPEW_RADIUS,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    for spew_start in spew_starts {
        spawn_spew_aoe(&mut commands, spew_start, 1.5, &aoe_desc_spew, None);
    }
}

fn setup_zhaitan(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Timed relative to the first green (-5 seconds)
    let puddle_starts: Vec<f32> = vec![9., 42., 74.].iter().map(|x| x + 5.).collect();
    let spread_starts: Vec<f32> = vec![18., 51., 83.].iter().map(|x| x + 5.).collect();
    let fear_starts: Vec<f32> = vec![14., 47., 79.].iter().map(|x| x + 5.).collect();
    let spew_starts: Vec<f32> = vec![3., 68.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &game,
        &mut meshes,
        &mut materials,
        "Zhaitan".to_string(),
        GREEN_SPAWNS_ZHAITAN.to_vec(),
        puddle_starts,
        spread_starts,
    );

    let spew_radius_nerfed = SPEW_RADIUS * 0.9;
    let spew_mesh: Handle<Mesh> = meshes.add(Circle::new(spew_radius_nerfed));
    let fear_mesh: Handle<Mesh> = meshes.add(Circle::new(WIDTH / 2.));
    let noodle_aoe_mesh: Handle<Mesh> = meshes.add(Circle::new(NOODLE_SLAM_RADIUS));
    let material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));

    let aoe_desc_spew = AoeDesc {
        mesh: spew_mesh,
        radius: spew_radius_nerfed,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    for spew_start in spew_starts {
        spawn_spew_aoe(
            &mut commands,
            spew_start,
            3.,
            &aoe_desc_spew,
            Some(Timer::from_seconds(10., TimerMode::Once)),
        );
    }

    let aoe_desc_fear = AoeDesc {
        mesh: fear_mesh,
        radius: WIDTH / 2.,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    for fear_start in fear_starts {
        spawn_aoe(
            &mut commands,
            &aoe_desc_fear,
            Vec3::new(0., 0., LAYER_AOE),
            Aoe {
                visibility_start: Some(Timer::from_seconds(fear_start, TimerMode::Once)),
                detonation: Timer::from_seconds(2.5, TimerMode::Once),
                damage: 30.,
                linger: None,
            },
            None,
        );
    }

    let aoe_desc_noodle = AoeDesc {
        mesh: noodle_aoe_mesh,
        radius: NOODLE_SLAM_RADIUS,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    // There is a third spawn but it doesn't really do much all things considered
    let noodle_spawns = vec![
        (
            5.,
            vec![
                Transform::from_xyz(-36., 224., LAYER_MOB),
                Transform::from_xyz(375., -80., LAYER_MOB),
                Transform::from_xyz(-120., -255., LAYER_MOB),
            ],
        ),
        (
            37.,
            vec![
                Transform::from_xyz(-36., 400., LAYER_MOB),
                Transform::from_xyz(-142., -142., LAYER_MOB),
                Transform::from_xyz(275., -104., LAYER_MOB),
            ],
        ),
    ];

    for (visibility_start, noodle_positions) in noodle_spawns {
        for noodle_pos in noodle_positions {
            commands.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(NOODLE_RADIUS * 2., NOODLE_RADIUS * 2.)),
                    image: asset_server.load("noodle.png"),
                    ..default()
                },
                Visibility::Hidden,
                noodle_pos,
                MobNoodle {
                    visibility_start: Timer::from_seconds(visibility_start, TimerMode::Once),
                    slam_cooldown: Timer::from_seconds(5., TimerMode::Repeating),
                    aoe_desc: aoe_desc_noodle.clone(),
                },
                Enemy,
                Hp(2.),
                CollisionRadius(NOODLE_RADIUS),
                PhaseEntity,
            ));
        }
    }
}

fn setup_soowonone(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puddle_starts: Vec<f32> = vec![4., 25., 49., 70., 94.];
    let spread_starts: Vec<f32> = vec![12., 58., 103.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &game,
        &mut meshes,
        &mut materials,
        "Soo-Won 1".to_string(),
        GREEN_SPAWNS_SOOWONONE.to_vec(),
        puddle_starts,
        spread_starts,
    );

    let wave_texture = asset_server.load("wave.png");

    let wave_sprite = Sprite {
        custom_size: Some(Vec2::new(WAVE_MAX_RADIUS * 2., WAVE_MAX_RADIUS * 2.)),
        image: wave_texture.clone(),
        ..default()
    };

    commands.spawn((
        wave_sprite.clone(),
        Sprite {
            custom_size: Some(Vec2::new(WAVE_MAX_RADIUS * 2., WAVE_MAX_RADIUS * 2.)),
            image: wave_texture.clone(),
            ..default()
        },
        Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(7., TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(32., TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(52., TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(77., TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(97., TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    let rotating_soup_mesh: Handle<Mesh> = meshes.add(Circle::new(ROTATING_SOUP_RADIUS));
    let rotating_soup_material =
        materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 0.3)));

    for i in 1..=5 {
        let radius = (i as f32) / 5. * (HEIGHT / 2. - 20.);
        let theta = i as f32 * 6. * PI / 5.;
        let mut dtheta = (7. - (i as f32)) / 5. * ROTATING_SOUP_DTHETA;
        if i % 2 == 0 {
            dtheta = -dtheta;
        }

        commands.spawn((
            Mesh2d(rotating_soup_mesh.clone()),
            MeshMaterial2d(rotating_soup_material.clone()),
            Transform::from_xyz(0., radius, LAYER_ROTATING_SOUP),
            RotatingSoup {
                radius,
                theta,
                dtheta,
            },
            CollisionRadius(ROTATING_SOUP_RADIUS),
            Soup {
                damage: 5.,
                duration: None,
            },
            PhaseEntity,
        ));
    }

    setup_claw_swipes(
        &mut commands,
        &mut meshes,
        &mut materials,
        vec![15., 60., 105.],
    );
}

fn setup_soowontwo(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puddle_starts: Vec<f32> = vec![10.7, 28.3, 52., 73.9, 103.];
    let spread_starts: Vec<f32> = vec![20.2, 65.9, 103. + 9.5];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &game,
        &mut meshes,
        &mut materials,
        "Soo-Won 2".to_string(),
        GREEN_SPAWNS_SOOWONTWO.to_vec(),
        puddle_starts,
        spread_starts,
    );

    let wave_texture = asset_server.load("wave.png");
    let wave_sprite = Sprite {
        custom_size: Some(Vec2::new(WAVE_MAX_RADIUS * 2., WAVE_MAX_RADIUS * 2.)),
        image: wave_texture.clone(),
        ..default()
    };

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(13.8, TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(34.9, TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(55.4, TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(80.6, TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    commands.spawn((
        wave_sprite.clone(),
        Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        Wave {
            visibility_start: Timer::from_seconds(106., TimerMode::Once),
            ..default()
        },
        PhaseEntity,
    ));

    let rotating_soup_mesh: Handle<Mesh> = meshes.add(Circle::new(ROTATING_SOUP_RADIUS));
    let rotating_soup_material =
        materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 0.3)));

    for i in 1..=5 {
        let radius = (i as f32) / 5. * (HEIGHT / 2. - 20.);
        let theta = i as f32 * 6. * PI / 5.;
        let mut dtheta = (7. - (i as f32)) / 5. * ROTATING_SOUP_DTHETA;
        if i % 2 == 0 {
            dtheta = -dtheta;
        }

        commands.spawn((
            Mesh2d(rotating_soup_mesh.clone()),
            MeshMaterial2d(rotating_soup_material.clone()),
            Transform::from_xyz(0., radius, LAYER_ROTATING_SOUP),
            RotatingSoup {
                radius,
                theta,
                dtheta,
            },
            CollisionRadius(ROTATING_SOUP_RADIUS),
            Soup {
                damage: 5.,
                duration: None,
            },
            PhaseEntity,
        ));
    }

    setup_claw_swipes(
        &mut commands,
        &mut meshes,
        &mut materials,
        vec![22.3, 68., 103. + 9.5 + 2.1],
    );

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(BIGBOY_RADIUS * 2., BIGBOY_RADIUS * 2.)),
            image: asset_server.load("wyvern.png"),
            ..default()
        },
        Transform::from_xyz(400., 0., LAYER_MOB),
        MobWyvern {
            shoot_cooldown: Timer::from_seconds(1., TimerMode::Repeating),
            shockwave_cooldown: Timer::from_seconds(18., TimerMode::Repeating),
            charge_cooldown: Timer::from_seconds(11., TimerMode::Repeating),
        },
        Enemy,
        Hp(15.),
        CollisionRadius(BIGBOY_RADIUS),
        PhaseEntity,
    ));

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(BIGBOY_RADIUS * 2., BIGBOY_RADIUS * 2.)),
            image: asset_server.load("goliath.png"),
            ..default()
        },
        Transform::from_xyz(300., 0., LAYER_MOB),
        MobGoliath {
            shoot_cooldown: Timer::from_seconds(5., TimerMode::Repeating),
        },
        Enemy,
        Hp(10.),
        Velocity(Vec3::ZERO),
        CollisionRadius(BIGBOY_RADIUS),
        PhaseEntity,
    ));
}

fn jormag_soup_beam_system(time: Res<Time>, mut soups: Query<&mut RotatingSoup>) {
    for mut soup in &mut soups {
        let radius = (WIDTH / 2. - 70.) * ((time.elapsed_secs() / 8.).cos() + 1.) / 2. + 35.;
        soup.radius = radius;
    }
}

fn run_if_phase_update(
    menu_state: Res<State<MenuState>>,
    game_state: Res<State<GameState>>,
) -> bool {
    if *menu_state != MenuState::Unpaused {
        return false;
    }
    match game_state.get() {
        GameState::Nothing => false,
        GameState::PurificationOne
        | GameState::Jormag
        | GameState::Primordus
        | GameState::Kralkatorrik
        | GameState::PurificationTwo
        | GameState::Mordremoth
        | GameState::Zhaitan
        | GameState::PurificationThree
        | GameState::SooWonOne
        | GameState::PurificationFour
        | GameState::SooWonTwo => true,
    }
}

fn run_if_boss_phase_update(
    menu_state: Res<State<MenuState>>,
    game_state: Res<State<GameState>>,
) -> bool {
    if *menu_state != MenuState::Unpaused {
        return false;
    }
    match game_state.get() {
        GameState::Nothing
        | GameState::PurificationOne
        | GameState::PurificationTwo
        | GameState::PurificationThree
        | GameState::PurificationFour => false,

        GameState::Jormag
        | GameState::Primordus
        | GameState::Kralkatorrik
        | GameState::Mordremoth
        | GameState::Zhaitan
        | GameState::SooWonOne
        | GameState::SooWonTwo => true,
    }
}

fn run_if_purification_phase_update(
    menu_state: Res<State<MenuState>>,
    game_state: Res<State<GameState>>,
) -> bool {
    if *menu_state != MenuState::Unpaused {
        return false;
    }
    match game_state.get() {
        GameState::Nothing
        | GameState::Jormag
        | GameState::Primordus
        | GameState::Kralkatorrik
        | GameState::Mordremoth
        | GameState::Zhaitan
        | GameState::SooWonOne
        | GameState::SooWonTwo
        | GameState::PurificationFour => false,

        GameState::PurificationOne | GameState::PurificationTwo | GameState::PurificationThree => {
            true
        }
    }
}

fn main() {
    let game = Game {
        time_elapsed: Stopwatch::new(),
        player_damage_taken: 0.,
        continuous: false,
        orb_target: -1,
        echo_enabled: false,
        hints_enabled: true,
        hint: None,
        greens_enabled: true,
        puddles_enabled: true,
        unlimited_range_enabled: true,
        ai_enabled: true,
        ai_bars_enabled: true,
        player_role: Some(AiRole::Virt1),
        audio_enabled: true,
    };

    let binding = App::new();
    let mut app = binding;
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(WIDTH, HEIGHT).with_scale_factor_override(1.),
            ..default()
        }),
        ..default()
    }))
    .init_state::<GameState>()
    .init_state::<MenuState>()
    .add_event::<DamageFlashEvent>()
    .add_event::<RestartEvent>()
    .insert_resource(game)
    .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
    .insert_resource(AssetsLoading(vec![]))
    .add_systems(Startup, setup)
    .add_systems(OnEnter(MenuState::Loading), setup_loading_system)
    .add_systems(
        Update,
        (update_loading_system).run_if(in_state(MenuState::Loading)),
    )
    .add_systems(OnExit(MenuState::Loading), cleanup_menu_system)
    .add_systems(OnEnter(MenuState::StartMenu), setup_menu_system)
    .add_systems(
        Update,
        (update_menu_system, update_menu_onoff_system).run_if(in_state(MenuState::StartMenu)),
    )
    .add_systems(OnExit(MenuState::StartMenu), cleanup_menu_system)
    .add_systems(OnEnter(MenuState::Paused), setup_pause_menu_system)
    .add_systems(
        Update,
        update_menu_system.run_if(in_state(MenuState::Paused)),
    )
    .add_systems(OnExit(MenuState::Paused), cleanup_menu_system)
    .add_systems(OnEnter(MenuState::PausedShowHint), setup_show_hint_system)
    .add_systems(
        Update,
        update_menu_system.run_if(in_state(MenuState::PausedShowHint)),
    )
    .add_systems(OnExit(MenuState::PausedShowHint), cleanup_menu_system)
    .add_systems(OnEnter(MenuState::Success), setup_success_system)
    .add_systems(
        Update,
        update_menu_system.run_if(in_state(MenuState::Success)),
    )
    .add_systems(OnExit(MenuState::Success), cleanup_menu_system)
    .add_systems(OnEnter(MenuState::Failure), setup_failure_system)
    .add_systems(
        Update,
        update_menu_system.run_if(in_state(MenuState::Failure)),
    )
    .add_systems(OnExit(MenuState::Failure), cleanup_menu_system)
    .add_systems(Update, restart_event_system);

    add_update_phase_set(&mut app);
    add_update_purification_phase_set(&mut app);
    add_update_boss_phase_set(&mut app);

    app.add_systems(OnEnter(GameState::PurificationOne), setup_phase)
        .add_systems(OnEnter(GameState::Jormag), setup_phase)
        .add_systems(OnEnter(GameState::Primordus), setup_phase)
        .add_systems(OnEnter(GameState::Kralkatorrik), setup_phase)
        .add_systems(OnEnter(GameState::PurificationTwo), setup_phase)
        .add_systems(OnEnter(GameState::Mordremoth), setup_phase)
        .add_systems(OnEnter(GameState::Zhaitan), setup_phase)
        .add_systems(OnEnter(GameState::PurificationThree), setup_phase)
        .add_systems(OnEnter(GameState::SooWonOne), setup_phase)
        .add_systems(OnEnter(GameState::PurificationFour), setup_phase)
        .add_systems(OnEnter(GameState::SooWonTwo), setup_phase);

    app.add_systems(OnEnter(GameState::PurificationOne), setup_purification)
        .add_systems(OnEnter(GameState::PurificationTwo), setup_purification)
        .add_systems(OnEnter(GameState::PurificationThree), setup_purification);

    app.add_systems(OnExit(GameState::PurificationOne), cleanup_phase)
        .add_systems(OnExit(GameState::Jormag), cleanup_phase)
        .add_systems(OnExit(GameState::Primordus), cleanup_phase)
        .add_systems(OnExit(GameState::Kralkatorrik), cleanup_phase)
        .add_systems(OnExit(GameState::PurificationTwo), cleanup_phase)
        .add_systems(OnExit(GameState::Mordremoth), cleanup_phase)
        .add_systems(OnExit(GameState::Zhaitan), cleanup_phase)
        .add_systems(OnExit(GameState::PurificationThree), cleanup_phase)
        .add_systems(OnExit(GameState::SooWonOne), cleanup_phase)
        .add_systems(OnExit(GameState::PurificationFour), cleanup_phase)
        .add_systems(OnExit(GameState::SooWonTwo), cleanup_phase);

    app.configure_sets(Update, (PhaseSet::UpdatePhase).run_if(run_if_phase_update));

    app.configure_sets(
        Update,
        (PhaseSet::UpdatePurificationPhase).run_if(run_if_purification_phase_update),
    );

    app.configure_sets(
        Update,
        (PhaseSet::UpdateBossPhase).run_if(run_if_boss_phase_update),
    );

    app.add_systems(
        OnEnter(GameState::PurificationOne),
        setup_purification_one.after(setup_purification),
    )
    .add_systems(OnEnter(GameState::Jormag), setup_jormag.after(setup_phase))
    .add_systems(
        Update,
        jormag_soup_beam_system
            .run_if(in_state(GameState::Jormag))
            .run_if(in_state(MenuState::Unpaused)),
    )
    .add_systems(
        OnEnter(GameState::Primordus),
        setup_primordus.after(setup_phase),
    )
    .add_systems(
        OnEnter(GameState::Kralkatorrik),
        setup_kralkatorrik.after(setup_phase),
    )
    .add_systems(
        OnEnter(GameState::PurificationTwo),
        setup_purification_two.after(setup_purification),
    )
    .add_systems(
        Update,
        (
            timecaster_system
                .run_if(in_state(GameState::PurificationTwo))
                .run_if(in_state(MenuState::Unpaused)),
            unleash_the_bees
                .run_if(in_state(GameState::PurificationTwo))
                .run_if(in_state(MenuState::Unpaused)),
        ),
    )
    .add_systems(
        OnEnter(GameState::Mordremoth),
        setup_mordremoth.after(setup_phase),
    )
    .add_systems(
        OnEnter(GameState::Zhaitan),
        setup_zhaitan.after(setup_phase),
    )
    .add_systems(
        Update,
        noodle_system
            .run_if(in_state(GameState::Zhaitan))
            .run_if(in_state(MenuState::Unpaused)),
    )
    .add_systems(
        OnEnter(GameState::PurificationThree),
        setup_purification_three.after(setup_purification),
    )
    .add_systems(
        Update,
        (
            saltspray_system
                .run_if(in_state(GameState::PurificationThree))
                .run_if(in_state(MenuState::Unpaused)),
            aoes_system
                .run_if(in_state(GameState::PurificationThree))
                .run_if(in_state(MenuState::Unpaused)),
            aoes_detonation_system
                .run_if(in_state(GameState::PurificationThree))
                .run_if(in_state(MenuState::Unpaused)),
        ),
    )
    .add_systems(
        OnEnter(GameState::SooWonOne),
        setup_soowonone.after(setup_phase),
    )
    .add_systems(
        OnEnter(GameState::PurificationFour),
        setup_purification_four.after(setup_phase),
    )
    .add_systems(
        Update,
        (
            collisions_bullets_orbs_system
                .run_if(in_state(GameState::PurificationFour))
                .run_if(in_state(MenuState::Unpaused)),
            collisions_orbs_edge_system
                .run_if(in_state(GameState::PurificationFour))
                .run_if(in_state(MenuState::Unpaused)),
            boss_existence_check_system
                .run_if(in_state(GameState::PurificationFour))
                .run_if(in_state(MenuState::Unpaused)),
            boss_healthbar_system
                .run_if(in_state(GameState::PurificationFour))
                .run_if(in_state(MenuState::Unpaused)),
            player_ai_purification_phase_system
                .run_if(in_state(GameState::PurificationFour))
                .run_if(in_state(MenuState::Unpaused)),
        ),
    )
    .add_systems(
        OnEnter(GameState::SooWonTwo),
        setup_soowontwo.after(setup_phase),
    )
    .add_systems(
        Update,
        (
            goliath_system
                .run_if(in_state(GameState::SooWonTwo))
                .run_if(in_state(MenuState::Unpaused)),
            wyvern_system
                .run_if(in_state(GameState::SooWonTwo))
                .run_if(in_state(MenuState::Unpaused)),
        ),
    )
    .run();
}
