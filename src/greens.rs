use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::ops::Add;

use crate::game::*;
use crate::collisions::collide;

pub const GREEN_RADIUS: f32 = 160. * GAME_TO_PX;

#[derive(Copy, Clone)]
pub struct GreenSpawn {
    start: f32,
    positions: [Vec3; 3]
}

pub const GREEN_SPAWNS_JORMAG: [GreenSpawn; 2] = [
    GreenSpawn {
        start: 15.,
        positions: [
            Vec3::new(269., 3., 0.),
            Vec3::new(-270., 0., 0.),
            Vec3::new(-78., 240., 0.),
        ],
    },
    GreenSpawn {
        start: 55.,
        positions: [
            Vec3::new(312., 3., 0.),
            Vec3::new(-303., 1., 0.),
            Vec3::new(-78., 299., 0.),
        ],
    }
];

pub const GREEN_SPAWNS_PRIMORDUS: [GreenSpawn; 2] = [
    GreenSpawn {
        start: 23.,
        positions: [
            Vec3::new(269., -111., 0.),
            Vec3::new(-274., -113., 0.),
            Vec3::new(-62., -290., 0.),
        ],
    },
    GreenSpawn {
        start: 77.,
        positions: [
            Vec3::new(365., -153., 0.),
            Vec3::new(-364., -155., 0.),
            Vec3::new(-82., -387., 0.),
        ],
    }
];

pub const GREEN_SPAWNS_ZHAITAN: [GreenSpawn; 3] = [
    GreenSpawn {
        start: 0., // actually -5., not entirely sure what to do here
        positions: [
            Vec3::new(158., -110., 0.),
            Vec3::new(-158., -114., 0.),
            Vec3::new(1., 258., 0.),
        ],
    },
    GreenSpawn {
        start: 28. + 5.,
        positions: [
            Vec3::new(197., -131., 0.),
            Vec3::new(-201., -131., 0.),
            Vec3::new(1., 258., 0.),
        ],
    },
    GreenSpawn {
        start: 60. + 5.,
        positions: [
            Vec3::new(308., -179., 0.),
            Vec3::new(-308., -189., 0.),
            Vec3::new(2., 387., 0.),
        ],
    }
];

pub const GREEN_SPAWNS_SOOWONONE: [GreenSpawn; 2] = [
    GreenSpawn {
        start: 5.,
        positions: [
            Vec3::new(-131., 75., 0.),
            Vec3::new(-47., 351., 0.),
            Vec3::new(-199., -64., 0.),
        ],
    },
    GreenSpawn {
        start: 50.,
        positions: [
            Vec3::new(-268., 174., 0.),
            Vec3::new(-47., 351., 0.),
            Vec3::new(-290., -101., 0.),
        ],
    }
    // there's another at 90 :(
];

pub const GREEN_SPAWNS_SOOWONTWO: [GreenSpawn; 3] = [
    GreenSpawn {
        start: 12.,
        positions: [
            Vec3::new(-131., 75., 0.),
            Vec3::new(-47., 351., 0.),
            Vec3::new(-199., -64., 0.),
        ],
    },
    GreenSpawn {
        start: 58.,
        positions: [
            Vec3::new(-268., 174., 0.),
            Vec3::new(-47., 351., 0.),
            Vec3::new(-290., -101., 0.),
        ],
    },
    GreenSpawn {
        start: 104.,
        positions: [
            Vec3::new(-30., WIDTH / 2. - GREEN_RADIUS * 1.2, 0.),
            {
                let r = WIDTH / 2. - GREEN_RADIUS * 1.2;
                let cos = -0.924;
                let sin = 0.383;
                Vec3::new(r * cos, r * sin, 0.)
            },
            {
                let r = WIDTH / 2. - GREEN_RADIUS * 1.2;
                let cos = -0.809;
                let sin = -0.588;
                Vec3::new(r * cos, r * sin, 0.)
            },
        ],
    }
];

#[derive(Component)]
pub struct StackGreen {
    visibility_start: Timer,
    detonation: Timer,
}

#[derive(Component)]
pub struct StackGreenIndicator;

pub fn greens_system(time: Res<Time>,
                 mut greens: Query<(&mut StackGreen, &mut Visibility, &Children)>,
                 mut indicators: Query<(&StackGreenIndicator, &mut Transform), Without<StackGreen>>,
                 ) {
    for (mut green, mut visibility, children) in &mut greens {
        let mut visible = Visibility::Inherited;
        if !green.visibility_start.finished() {
            green.visibility_start.tick(time.delta());
            visible = Visibility::Hidden;
        } else {
            green.detonation.tick(time.delta());
        }

        if green.detonation.finished() {
            visible = Visibility::Hidden;
        }

        *visibility = visible;

        if visible == Visibility::Hidden {
            continue;
        }

        let det_scale = green.detonation.percent_left();

        for &child in children.iter() {
            if let Ok((_, mut transform_indicator)) = indicators.get_mut(child) {
                transform_indicator.scale = Vec3::splat(det_scale);
            }
        }
    }
}

pub fn greens_detonation_system(
    game: ResMut<Game>,
    mut players: Query<(&mut Player, &Transform)>,
    greens: Query<(&StackGreen, &Children)>,
    indicators: Query<(&StackGreenIndicator, &Transform)>,
    ) {
    for (green, children) in &greens {
        if green.detonation.just_finished() {
            let mut any_collide = false;
            for (_, transform_player) in &players {
                for &child in children.iter() {
                    if let Ok((_, transform_indicator)) = indicators.get(child) {
                        any_collide = any_collide || collide(transform_player.translation, 0., transform_indicator.translation, GREEN_RADIUS);
                    }
                    if any_collide {
                        break;
                    }
                }
            }

            if !any_collide {
                if game.greens_enabled {
                    for (mut player, _) in &mut players {
                        player.hp = 0.;
                    }
                }
                info!("green exploded");
            }
        }
    }
}

pub fn setup_greens(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    green_spawns: Vec<GreenSpawn>,
    ) {

    let green_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(GREEN_RADIUS).into()).into();
    let green_bright_material = ColorMaterial::from(Color::rgb(0., 1.0, 0.));
    let green_dull_material = ColorMaterial::from(Color::rgba(0., 0.7, 0., 0.5));

    for green_spawn in &green_spawns {
        commands.spawn(SpriteBundle {
            transform: Transform::from_xyz(0., 0., LAYER_TARGET),
            visibility: Visibility::Hidden,
            ..default()
        }).with_children(|parent| {
            for position in green_spawn.positions {
                // let mut position = position_absolute.sub(Vec3::new(WIDTH / 2., HEIGHT / 2., 0.));
                // position.x *= -1.;
                // position.y *= -1.;
                parent.spawn(MaterialMesh2dBundle {
                    mesh: green_mesh.clone(),
                    transform: Transform::from_translation(position),
                    material: materials.add(green_dull_material.clone()),
                    ..default()
                });

                let position_above = position.add(Vec3::new(0., 0., 0.1));
                parent.spawn(MaterialMesh2dBundle {
                    mesh: green_mesh.clone(),
                    transform: Transform::from_translation(position_above).with_scale(Vec3::ZERO),
                    material: materials.add(green_bright_material.clone()),
                    ..default()
                }).insert(StackGreenIndicator);
            }
        }).insert(StackGreen {
            visibility_start: Timer::from_seconds(green_spawn.start, TimerMode::Once),
            detonation: Timer::from_seconds(5., TimerMode::Once),
        }).insert(PhaseEntity);
    }
}
