use bevy::{
    prelude::*,
    render::color::Color,
    sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle},
    window::CursorMoved,
};
use core::f32::consts::PI;
use std::collections::HashSet;
use std::time::Duration;
use std::ops::{Add, Mul, Sub};

#[derive(Component)]
struct Name(String);

enum TextValue {
    Hp,
    CooldownDodge,
    CooldownBlink,
    CooldownPortal,
    CooldownPull,
    StatusJump,
}

#[derive(Component)]
struct TextDisplay {
    value: TextValue,
    sprite: Option<Entity>,
}

#[derive(Component)]
struct MobOrb;

#[derive(Component)]
struct MobCrab;

#[derive(Component)]
struct MobGoliath {
    shoot_cooldown: Timer,
}

#[derive(Component)]
struct MobWyvern {
    shoot_cooldown: Timer,
    shockwave_cooldown: Timer,
    charge_cooldown: Timer,
}

#[derive(Component)]
struct EnemyBullet {
    damage: f32,
    knockback: f32,
}

#[derive(Component)]
struct VoidZone;

#[derive(Component)]
struct RotatingSoup {
    radius: f32,
    theta: f32,
    dtheta: f32,
}

struct DamageFlashEvent {
    entity: Entity,
}

#[derive(Component)]
struct TintUntint {
    color: Color,
    tint_color: Color,
    untint_timer: Timer,
    tint_timer: Timer,
}

// 312 - 60 / 2 @ 56 (14 ticks)

const VOID_ZONE_GROWTH_DURATION_SECS: f32 = 4.;
const VOID_ZONE_START_RADIUS: f32 = 30.;
const VOID_ZONE_GROWTH_AMOUNT: f32 = 252. / 14.;
const VOID_ZONE_CRAB_SPAWN_DURATION_SECS: f32 = 10.;

const BOSS_RADIUS: f32 = 420. * GAME_TO_PX;
const BIGBOY_RADIUS: f32 = 120. * GAME_TO_PX;

const GOLIATH_MOVE_SPEED: f32 = 20.;
const GOLIATH_BULLET_SPEED: f32 = 50.;
const GOLIATH_BULLET_DAMAGE: f32 = 20.;
const GOLIATH_BULLET_KNOCKBACK: f32 = 120. * GAME_TO_PX;

const WYVERN_CHARGE_RANGE: f32 = 1200. * GAME_TO_PX;
const WYVERN_BULLET_SPEED: f32 = 200.;
const WYVERN_BULLET_DAMAGE: f32 = 10.;

const PLAYER_RADIUS: f32 = 20.;

const PUDDLE_RADIUS: f32 = 450. * GAME_TO_PX;
const PUDDLE_DAMAGE: f32 = 20.;

const ROTATING_SOUP_RADIUS: f32 = 40.;
const ROTATING_SOUP_DTHETA: f32 = 0.2;

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

#[derive(Component)]
struct Puddle {
    visibility_start: Timer,
    drop: Timer,
}

#[derive(Component)]
struct Soup {
    damage: f32,
}

#[derive(Component)]
struct OrbTarget(i32);

const ORB_TARGET_COLOR_BASE: Color = Color::rgb(0.5, 0.5, 0.5);
const ORB_TARGET_COLOR_ACTIVE: Color = Color::rgb(0.7, 1., 0.7);

#[derive(Component)]
struct CursorMark;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct PlayerTag;

#[derive(Component)]
struct Hp(f32);

#[derive(Component)]
struct HasHit(HashSet<Entity>);

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Boss {
    max_hp: f32,
}

#[derive(Component)]
struct BossHealthbar;

#[derive(Component)]
struct BossHealthbarText;

#[derive(Component)]
struct CollisionRadius(f32);

#[derive(Component)]
struct EffectForcedMarch {
    target: Vec3,
    speed: f32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    StartMenu,
    PurificationOne,
    Jormag,
    // Primordus, -> big aoe and void zone
    // Kralkatorrik, -> line aoes
    // PurificationTwo, -> kill big boy without cleaving
    Mordremoth,
    // Zhaitan, -> noodles and grid aoe
    // PurificationThree, -> kill bigger boy without cleaving
    // SooWonOne, -> soowontwo minus big boys
    // PurificationFour, -> damage orb
    SooWonTwo,
    Failure,
    Success,
}

#[derive(Component)]
struct MenuContainer;

#[derive(Component)]
struct ButtonNextState(GameState);

const CRAB_SIZE: f32 = 30.;
const CRAB_SPEED: f32 = 15.;
const BULLET_SIZE: f32 = 10.;
const BULLET_DAMAGE: f32 = 0.3;
const BULLET_COOLDOWN: f32 = 0.2;
const ORB_RADIUS: f32 = 190. * GAME_TO_PX;
const ORB_TARGET_RADIUS: f32 = 190. * GAME_TO_PX;
const ORB_VELOCITY_DECAY: f32 = 0.5;
const GREEN_RADIUS: f32 = 160. * GAME_TO_PX;
const PLAYER_REGEN: f32 = 1.;

const LAYER_PLAYER: f32 = 100.;
const LAYER_CURSOR: f32 = LAYER_PLAYER - 5.;
const LAYER_MOB: f32 = 20.;
const LAYER_WAVE: f32 = 15.;
const LAYER_AOE: f32 = 12.;
const LAYER_TARGET: f32 = 10.;
const LAYER_ROTATING_SOUP: f32 = 11.;
const LAYER_MAP: f32 = 0.;
const LAYER_VOID: f32 = 0.5;
const LAYER_UI: f32 = 1.;
const LAYER_TEXT: f32 = 2.;

const WIDTH: f32 = 1024.;
const HEIGHT: f32 = 1024.;
const GAME_WIDTH: f32 = 2849.;
const GAME_RADIUS: f32 = GAME_WIDTH / 2.;
const PX_TO_GAME: f32 = GAME_WIDTH / WIDTH;
const GAME_TO_PX: f32 = 1. / PX_TO_GAME;

const MAP_RADIUS: f32 = WIDTH / 2.;

#[derive(Copy, Clone)]
struct GreenSpawn {
    start: f32,
    positions: [Vec3; 3]
}

struct Player {
    hp: f32,
    shoot_cooldown: Timer,
    dodge_cooldown: Timer,
    blink_cooldown: Timer,
    portal_cooldown: Timer,
    pull_cooldown: Timer,
    jump_cooldown: Timer,
    invuln: Timer,
    jump: Timer,
    entity: Option<Entity>,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            hp: 100.,
            shoot_cooldown: Timer::from_seconds(BULLET_COOLDOWN, false),
            dodge_cooldown: Timer::from_seconds(10., false),
            blink_cooldown: Timer::from_seconds(16., false),
            portal_cooldown: Timer::from_seconds(60., false),
            jump_cooldown: Timer::from_seconds(0.6, false),
            pull_cooldown: Timer::from_seconds(20., false),
            invuln: Timer::from_seconds(0.75, false),
            jump: Timer::from_seconds(0.75, false),
            entity: None,
        }
    }
}

struct Game {
    player: Player,
    orb_target: i32,
}

#[derive(Component)]
struct VoidZoneGrowth(Timer);

#[derive(Component)]
struct VoidZoneCrabSpawn(Timer);

#[derive(Component)]
struct SpreadAoeSpawn {
    timers: Vec<Timer>,
    aoe_desc: AoeDesc,
}

const SPREAD_DAMAGE: f32 = 10.;
const SPREAD_DETONATION: f32 = 5.;
const SPREAD_RADIUS: f32 = 240. * GAME_TO_PX;

const SPEW_DAMAGE: f32 = 40.;
const SPEW_RADIUS: f32 = 240. * GAME_TO_PX;
const SPEW_SPACING: f32 = 10. * GAME_TO_PX;
const SPEW_DYDX: f32 = -0.3;

#[derive(Component)]
struct StackGreen {
    visibility_start: Timer,
    detonation: Timer,
}

#[derive(Component)]
struct StackGreenIndicator;

const AOE_BASE_COLOR: Color = Color::rgba(0.9, 0.9, 0., 0.4);
const AOE_DETONATION_COLOR: Color = Color::rgba(0.7, 0., 0., 0.7);

#[derive(Component)]
struct Aoe {
    visibility_start: Option<Timer>,
    detonation: Timer,
    damage: f32,
}

#[derive(Component)]
struct AoeFollow {
    target: Entity,
}

#[derive(Component)]
struct AoeIndicator;

struct AoeDesc {
    mesh: Mesh2dHandle,
    radius: f32,
    material_base: Handle<ColorMaterial>,
    material_detonation: Handle<ColorMaterial>,
}

#[derive(Component)]
struct Wave {
    visibility_start: Timer,
    growth: Timer,
}

impl Default for Wave {
    fn default() -> Wave {
        Wave {
            visibility_start: Timer::from_seconds(0., false),
            growth: Timer::from_seconds(WAVE_GROWTH_DURATION, false),
        }
    }
}

const WAVE_MAX_RADIUS: f32 = WIDTH / 2.;
const WAVE_VELOCITY: f32 = GAME_RADIUS / 3.2 * GAME_TO_PX;
const WAVE_GROWTH_DURATION: f32 = WAVE_MAX_RADIUS / WAVE_VELOCITY;
const WAVE_DAMAGE: f32 = 75.;

const GREEN_SPAWNS_JORMAG: [GreenSpawn; 2] = [
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

const _GREEN_SPAWNS_PRIMORDUS: [GreenSpawn; 2] = [
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

const _GREEN_SPAWNS_ZHAITAN: [GreenSpawn; 3] = [
    GreenSpawn {
        start: 0., // actually -5., not entirely sure what to do here
        positions: [
            Vec3::new(158., -110., 0.),
            Vec3::new(-158., -114., 0.),
            Vec3::new(1., 258., 0.),
        ],
    },
    GreenSpawn {
        start: 28.,
        positions: [
            Vec3::new(197., -131., 0.),
            Vec3::new(-201., -131., 0.),
            Vec3::new(1., 258., 0.),
        ],
    },
    GreenSpawn {
        start: 60.,
        positions: [
            Vec3::new(308., -179., 0.),
            Vec3::new(-308., -189., 0.),
            Vec3::new(2., 387., 0.),
        ],
    }
];

const _GREEN_SPAWNS_SOOWONONE: [GreenSpawn; 2] = [
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

const GREEN_SPAWNS_SOOWONTWO: [GreenSpawn; 2] = [
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
    }
    // there's another around 102
];

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup_menu_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_size = Size::new(Val::Px(350.0), Val::Px(65.0));
    let button_margin = UiRect::all(Val::Px(10.));

    let button_style = Style {
        size: button_size,
        // center button
        margin: button_margin,
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 40.0,
        color: Color::rgb(0.9, 0.9, 0.9),
    };

    commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Px(WIDTH), Val::Px(HEIGHT)),
            flex_direction: FlexDirection::ColumnReverse,
            // horizontally center children
            justify_content: JustifyContent::Center,
            // vertically center children
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    })
    .with_children(|container| {
        container.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Purification One",
                text_style.clone(),
            ));
        })
        .insert(ButtonNextState(GameState::PurificationOne));

        container.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Jormag",
                text_style.clone(),
            ));
        })
        .insert(ButtonNextState(GameState::Jormag));


        container.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Mordremoth",
                text_style.clone(),
            ));
        })
        .insert(ButtonNextState(GameState::Mordremoth));

        container.spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Soo-Won Two",
                text_style.clone(),
            ));
        })
        .insert(ButtonNextState(GameState::SooWonTwo));

    })
    .insert(MenuContainer);
}

fn update_menu_system(
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &ButtonNextState),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, next_state) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(next_state.0).unwrap();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu_system(mut commands: Commands, containers: Query<(Entity, &MenuContainer)>) {
    for (entity, _) in &containers {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_success_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(WIDTH), Val::Px(100.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.2).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Phase cleared!",
                TextStyle {
                    font: asset_server.load("trebuchet_ms.ttf"),
                    font_size: 80.,
                    color: Color::rgb(0.3, 1.0, 0.3),
                },
            ));
        });
}

fn setup_failure_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(WIDTH), Val::Px(100.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.2).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "You died :(",
                TextStyle {
                    font: asset_server.load("trebuchet_ms.ttf"),
                    font_size: 80.,
                    color: Color::rgb(0.9, 0.2, 0.2),
                },
            ));
        });
}

fn spawn_crab(commands: &mut Commands, asset_server: &Res<AssetServer>, crab_pos: Vec3) {
    commands.spawn_bundle(SpriteBundle {
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
    .insert(Hp(0.3));
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_phase(
    mut commands: Commands, asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    // Reset all cooldowns and invuln timings
    game.player.dodge_cooldown.tick(Duration::from_secs_f32(1000.));
    game.player.blink_cooldown.tick(Duration::from_secs_f32(1000.));
    game.player.portal_cooldown.tick(Duration::from_secs_f32(1000.));
    game.player.pull_cooldown.tick(Duration::from_secs_f32(1000.));
    game.player.invuln.tick(Duration::from_secs_f32(1000.));
    game.player.jump.tick(Duration::from_secs_f32(1000.));

    game.player.entity = Some(
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(PLAYER_RADIUS * 2., PLAYER_RADIUS * 2.)),
                ..default()
            },
            texture: asset_server.load("virt.png"),
            transform: Transform::from_xyz(0., 200., LAYER_PLAYER),
            ..default()
        }).insert(PlayerTag).id()
    );

    commands.spawn()
        .insert(VoidZoneGrowth(Timer::from_seconds(VOID_ZONE_GROWTH_DURATION_SECS, true)))
        .insert(VoidZoneCrabSpawn(Timer::from_seconds(VOID_ZONE_CRAB_SPAWN_DURATION_SECS, true)));

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.9, 0., 0.),
            custom_size: Some(Vec2::new(4., 4.)),
            ..default()
        },
        transform: Transform::from_xyz(0., 0., LAYER_CURSOR),
        ..default()
    }).insert(CursorMark);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("map.png"),
        transform: Transform::from_xyz(0., 0., LAYER_MAP),
        ..default()
    });

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 64.,
        color: Color::rgb(0.1, 0.7, 0.1),
    };

    let text_binding_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 28.,
        color: Color::rgb(0.4, 0.2, 0.),
    };
    let binding_y = 18.;

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("hp", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::Hp,
        sprite: None,
    });

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.6, 0.1, 0.1))),
        transform: Transform::from_xyz(0., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("0",
            TextStyle {
                font: asset_server.load("trebuchet_ms.ttf"),
                font_size: 80.,
                color: Color::rgb(0.7, 0.7, 0.1),
            })
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., -HEIGHT / 2. + 155., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownDodge,
        sprite: None,
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("0",
            TextStyle {
                font: asset_server.load("trebuchet_ms.ttf"),
                font_size: 80.,
                color: Color::rgb(0.1, 0.7, 0.7),
            })
            .with_alignment(TextAlignment::CENTER_RIGHT),
        transform: Transform::from_xyz(-90., -HEIGHT / 2. + 155., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::StatusJump,
        sprite: None,
    });

    let sprite_pull = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("pull.png"),
        transform: Transform::from_xyz(-128., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(-128., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownPull,
        sprite: Some(sprite_pull),
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("4", text_binding_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(-128., -HEIGHT / 2. + binding_y, LAYER_TEXT),
        ..default()
    });

    let sprite_blink = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("blink.png"),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownBlink,
        sprite: Some(sprite_blink),
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("E", text_binding_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(128., -HEIGHT / 2. + binding_y, LAYER_TEXT),
        ..default()
    });

    let sprite_portal = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("portal.png"),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_UI),
        ..default()
    }).id();

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + 55., LAYER_TEXT),
        ..default()
    }).insert(TextDisplay {
        value: TextValue::CooldownPortal,
        sprite: Some(sprite_portal),
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("R", text_binding_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(256., -HEIGHT / 2. + binding_y, LAYER_TEXT),
        ..default()
    });
}

fn setup_purification_one(
    mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {

    game.orb_target = 0;

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

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(ORB_RADIUS).into()).into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.9, 1.0, 1.0))),
        transform: Transform::from_xyz(0., 0., LAYER_MOB),
        ..default()
    }).insert(MobOrb).insert(Velocity(Vec3::new(0., 0., 0.)));

    let orb_target_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(ORB_TARGET_RADIUS).into()).into();
    let orb_target_material = ColorMaterial::from(Color::rgb(0.5, 0.5, 0.5));

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: orb_target_mesh.clone(),
        material: materials.add(orb_target_material.clone()),
        transform: Transform::from_xyz(-240., 240., LAYER_TARGET),
        ..default()
    }).insert(OrbTarget(0));

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: orb_target_mesh.clone(),
        material: materials.add(orb_target_material.clone()),
        transform: Transform::from_xyz(-240., -240., LAYER_TARGET),
        ..default()
    }).insert(OrbTarget(1));

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: orb_target_mesh.clone(),
        material: materials.add(orb_target_material.clone()),
        transform: Transform::from_xyz(240., -240., LAYER_TARGET),
        ..default()
    }).insert(OrbTarget(2));

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
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: void_zone_mesh.clone(),
            material: materials.add(void_zone_material.clone()),
            transform: Transform::from_translation(pos),
            ..default()
        })
        .insert(VoidZone)
        .insert(CollisionRadius(VOID_ZONE_START_RADIUS))
        .insert(Soup { damage: 25. });
    }
}

fn spread_aoe_spawn_system(
    time: ResMut<Time>,
    players: Query<Entity, With<PlayerTag>>,
    mut commands: Commands,
    mut spread_aoe_spawns: Query<&mut SpreadAoeSpawn>,
    ) {
    for mut spread_aoe_spawn in &mut spread_aoe_spawns {
        let mut do_spawn = false;
        for timer in &mut spread_aoe_spawn.timers {
            timer.tick(time.delta());

            if timer.just_finished() {
                do_spawn = true;
            }
        }

        if do_spawn {
            spawn_aoe(&mut commands, &spread_aoe_spawn.aoe_desc, Vec3::new(0., 0., LAYER_WAVE), Aoe {
                visibility_start: None,
                detonation: Timer::from_seconds(SPREAD_DETONATION, false),
                damage: SPREAD_DAMAGE,
            }, Some(AoeFollow { target: players.single() }));
        }
    }
}

fn spawn_aoe(
    commands: &mut Commands,
    aoe_desc: &AoeDesc,
    position: Vec3, aoe: Aoe, aoe_follow: Option<AoeFollow>) -> Entity {
    let id = commands.spawn_bundle(MaterialMesh2dBundle {
        transform: Transform::from_translation(position),
        mesh: aoe_desc.mesh.clone(),
        material: aoe_desc.material_base.clone(),
        ..default()
    }).with_children(|parent| {
        let position_above = Vec3::new(0., 0., 0.1);
        parent.spawn_bundle(MaterialMesh2dBundle {
            mesh: aoe_desc.mesh.clone(),
            transform: Transform::from_translation(position_above).with_scale(Vec3::ZERO),
            material: aoe_desc.material_detonation.clone(),
            ..default()
        }).insert(AoeIndicator);
    })
    .insert(aoe)
    .insert(CollisionRadius(aoe_desc.radius))
    .id();

    if let Some(aoe_follow) = aoe_follow {
        commands.entity(id).insert(aoe_follow);
    }

    id
}

fn spawn_spew_aoe(
    commands: &mut Commands,
    start: f32,
    aoe_desc: &AoeDesc,
    _linger: Option<f32>,
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
            let pos2 = Vec2::new(x, y).rotate(rotation).add(Vec2::new(SPEW_RADIUS * 0.7, SPEW_RADIUS * 0.1));
            let dy = pos2.y - GAME_RADIUS;
            let dist = (pos2.x * pos2.x + dy * dy).sqrt();
            let aoe_delay = dist / 2000.;

            let aoe = Aoe {
                visibility_start: Some(Timer::from_seconds(start + aoe_delay, false)),
                detonation: Timer::from_seconds(1.5, false),
                damage: SPEW_DAMAGE,
            };

            spawn_aoe(commands, aoe_desc, Vec3::new(pos2.x, pos2.y, LAYER_AOE), aoe, None);
        }
    }
}

fn setup_greens(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    green_spawns: Vec<GreenSpawn>,
    ) {

    let green_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(GREEN_RADIUS).into()).into();
    let green_bright_material = ColorMaterial::from(Color::rgb(0., 1.0, 0.));
    let green_dull_material = ColorMaterial::from(Color::rgba(0., 0.7, 0., 0.5));

    for green_spawn in &green_spawns {
        commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0., 0., LAYER_TARGET),
            visibility: Visibility { is_visible: false },
            ..default()
        }).with_children(|parent| {
            for position in green_spawn.positions {
                // let mut position = position_absolute.sub(Vec3::new(WIDTH / 2., HEIGHT / 2., 0.));
                // position.x *= -1.;
                // position.y *= -1.;
                parent.spawn_bundle(MaterialMesh2dBundle {
                    mesh: green_mesh.clone(),
                    transform: Transform::from_translation(position),
                    material: materials.add(green_dull_material.clone()),
                    ..default()
                });

                let position_above = position.add(Vec3::new(0., 0., 0.1));
                parent.spawn_bundle(MaterialMesh2dBundle {
                    mesh: green_mesh.clone(),
                    transform: Transform::from_translation(position_above).with_scale(Vec3::ZERO),
                    material: materials.add(green_bright_material.clone()),
                    ..default()
                }).insert(StackGreenIndicator);
            }
        }).insert(StackGreen {
            visibility_start: Timer::from_seconds(green_spawn.start, false),
            detonation: Timer::from_seconds(5., false),
        });
    }
}

fn setup_claw_swipes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    claw_swipe_starts: Vec<f32>,
    ) {
    let chonk_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(SWIPE_CHONK_RADIUS).into()).into();
    let ball_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(SWIPE_BALL_RADIUS).into()).into();
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
        let chonk_start = Timer::from_seconds(claw_swipe_start, false);
        let chonk_pos = SWIPE_CENTER;
        spawn_aoe(commands, &aoe_desc_chonk, chonk_pos, Aoe {
            visibility_start: Some(chonk_start),
            detonation: Timer::from_seconds(SWIPE_DETONATION, false),
            damage: SWIPE_DAMAGE,
        }, None);

        for bounce in 0..SWIPE_BALL_BOUNCE_COUNT {
            let offset = SWIPE_BALL_OFFSET + (bounce as f32) * SWIPE_BALL_RADIUS * 3.;
            for ball_i in 0..SWIPE_BALL_COUNT {
                let percent = (ball_i as f32) / (SWIPE_BALL_COUNT as f32 - 1.);
                let theta = percent * (SWIPE_END_THETA - SWIPE_START_THETA) + SWIPE_START_THETA;
                let pos = Vec3::new(
                    offset * -theta.cos(),
                    offset * theta.sin(),
                    LAYER_WAVE,
                ).add(chonk_pos);

                let timer = Timer::from_seconds(claw_swipe_start + 0.6 * (bounce as f32 + 1.), false);

                spawn_aoe(commands, &aoe_desc, pos, Aoe {
                    visibility_start: Some(timer),
                    detonation: Timer::from_seconds(SWIPE_DETONATION, false),
                    damage: SWIPE_DAMAGE,
                }, None);
            }
        }
    }
}

fn setup_boss_phase(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    boss_name: String,
    green_spawns: Vec<GreenSpawn>,
    puddle_starts: Vec<f32>,
    spread_starts: Vec<f32>,
    ) {
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(BOSS_RADIUS).into()).into(),
        material: materials.add(ColorMaterial::from(Color::rgba(1.0, 0.0, 0.0, 0.5))),
        transform: Transform::from_xyz(0., HEIGHT / 2. + 20., LAYER_MOB),
        ..default()
    }).insert(Boss {
        max_hp: 100.,
    }).insert(Enemy)
    .insert(Hp(100.))
    .insert(CollisionRadius(BOSS_RADIUS));

    setup_greens(
        commands,
        meshes,
        materials,
        green_spawns.to_vec()
    );

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1., 0., 0.),
            custom_size: Some(Vec2::new(256., 32.)),
            anchor: Anchor::CenterLeft,
            ..default()
        },
        transform: Transform::from_xyz(-WIDTH / 2. + 20., -HEIGHT / 2. + 128. + 24., LAYER_UI),
        ..default()
    }).insert(BossHealthbar);

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section(
            "100",
            TextStyle {
                font: asset_server.load("trebuchet_ms.ttf"),
                font_size: 16.,
                color: Color::rgb(1.0, 1.0, 1.0),
            },
        ).with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(-WIDTH / 2. + 20. + 128., -HEIGHT / 2. + 128. + 24., LAYER_TEXT),
        ..default()
    }).insert(BossHealthbarText);

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section(
            boss_name,
            TextStyle {
                font: asset_server.load("trebuchet_ms.ttf"),
                font_size: 32.,
                color: Color::rgb(0.0, 0.8, 0.8),
            },
        ).with_alignment(TextAlignment::BOTTOM_LEFT),
        transform: Transform::from_xyz(-WIDTH / 2. + 20., -HEIGHT / 2. + 128. + 8. + 32. + 8., LAYER_TEXT),
        ..default()
    });

    let void_zone_positions = [
        Vec3::new(0., 0., LAYER_VOID),
    ];

    let void_zone_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(VOID_ZONE_START_RADIUS).into()).into();
    let void_zone_material = ColorMaterial::from(Color::rgba(0.0, 0.0, 0.0, 0.9));

    for pos in void_zone_positions {
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: void_zone_mesh.clone(),
            material: materials.add(void_zone_material.clone()),
            transform: Transform::from_translation(pos),
            ..default()
        }).insert(VoidZone)
        .insert(CollisionRadius(VOID_ZONE_START_RADIUS))
        .insert(Soup { damage: 25. });
    }

    let puddle_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(PUDDLE_RADIUS).into()).into();
    let puddle_material = ColorMaterial::from(Color::rgba(0.5, 0.0, 0.0, 0.3));

    for puddle_start in puddle_starts {
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: puddle_mesh.clone(),
            material: materials.add(puddle_material.clone()),
            visibility: Visibility { is_visible: false },
            transform: Transform::from_xyz(0., 0., 0.,),
            ..default()
        }).insert(Puddle {
            visibility_start: Timer::from_seconds(puddle_start, false),
            drop: Timer::from_seconds(6., false),
        })
        .insert(CollisionRadius(PUDDLE_RADIUS))
        .insert(Soup { damage: 0. });
    }

    let spread_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(SPREAD_RADIUS).into()).into();
    let spread_material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let spread_material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));
    commands.spawn()
        .insert(SpreadAoeSpawn {
            timers: spread_starts.iter().map(|start| {
                Timer::from_seconds(*start, false)
            }).collect(),
            aoe_desc: AoeDesc {
                mesh: spread_mesh,
                material_base: spread_material_base,
                material_detonation: spread_material_detonation,
                radius: SPREAD_RADIUS,
            }
        });
}

fn setup_jormag(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {

    let puddle_starts: Vec<f32> = vec![5., 45., 85.];
    let spread_starts: Vec<f32> = vec![28., 68.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        "Jormag".to_string(),
        GREEN_SPAWNS_JORMAG.to_vec(),
        puddle_starts,
        spread_starts,
    );

    // TODO roving frost beam things properly

    let rotating_soup_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(70.).into()).into();
    let rotating_soup_material = materials.add(ColorMaterial::from(Color::rgba(0.0, 0.0, 0.0, 0.3)));

    for i in 0..4 {
        let radius = 0.;
        let theta = i as f32 * PI / 2.;
        let dtheta = 0.5;

        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: rotating_soup_mesh.clone(),
            material: rotating_soup_material.clone(),
            transform: Transform::from_xyz(0., radius, LAYER_ROTATING_SOUP),
            ..default()
        })
        .insert(RotatingSoup {
            radius,
            theta,
            dtheta,
        })
        .insert(CollisionRadius(70.))
        .insert(Soup { damage: 5. });
    }

}

fn setup_mordremoth(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {

    let puddle_starts: Vec<f32> = vec![8., 30., 53., 82.];
    let spread_starts: Vec<f32> = vec![18., 40., 63., 91.];
    let boop_starts: Vec<f32> = vec![22., 44., 67., 95.];
    let boop_delays: Vec<f32> = vec![0., 2., 4.]; // 21.5, 24, 26 hmm
    let spew_starts: Vec<f32> = vec![13., 35., 58., 87.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        "Mordremoth".to_string(),
        vec![],
        puddle_starts,
        spread_starts,
    );

    let wave_sprite = Sprite {
        custom_size: Some(Vec2::new(WAVE_MAX_RADIUS * 2., WAVE_MAX_RADIUS * 2.)),
        ..default()
    };
    let wave_texture = asset_server.load("wave.png");

    for boop_start in &boop_starts {
        for boop_delay in &boop_delays {
            commands.spawn_bundle(SpriteBundle {
                sprite: wave_sprite.clone(),
                texture: wave_texture.clone(),
                transform: Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
                ..default()
            }).insert(Wave {
                visibility_start: Timer::from_seconds(boop_start + boop_delay, false),
                ..default()
            });
        }
    }

    let spew_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(SPEW_RADIUS).into()).into();
    let material_base = materials.add(ColorMaterial::from(AOE_BASE_COLOR));
    let material_detonation = materials.add(ColorMaterial::from(AOE_DETONATION_COLOR));

    let aoe_desc_spew = AoeDesc {
        mesh: spew_mesh,
        radius: SPEW_RADIUS,
        material_base: material_base.clone(),
        material_detonation: material_detonation.clone(),
    };

    for spew_start in spew_starts {
        spawn_spew_aoe(&mut commands, spew_start, &aoe_desc_spew, None);
    }
}

fn setup_soowontwo(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>,
    ) {

    let puddle_starts: Vec<f32> = vec![11., 32., 57., 77., 103.];
    let spread_starts: Vec<f32> = vec![21., 67.];

    setup_boss_phase(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        "Soo-Won 2".to_string(),
        GREEN_SPAWNS_SOOWONTWO.to_vec(),
        puddle_starts,
        spread_starts,
    );

    let wave_sprite = Sprite {
        custom_size: Some(Vec2::new(WAVE_MAX_RADIUS * 2., WAVE_MAX_RADIUS * 2.)),
        ..default()
    };
    let wave_texture = asset_server.load("wave.png");

    commands.spawn_bundle(SpriteBundle {
        sprite: wave_sprite.clone(),
        texture: wave_texture.clone(),
        transform: Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        ..default()
    }).insert(Wave {
        visibility_start: Timer::from_seconds(13.5, false),
        ..default()
    });

    commands.spawn_bundle(SpriteBundle {
        sprite: wave_sprite.clone(),
        texture: wave_texture.clone(),
        transform: Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
        ..default()
    }).insert(Wave {
        visibility_start: Timer::from_seconds(38.5, false),
        ..default()
    });

    commands.spawn_bundle(SpriteBundle {
        sprite: wave_sprite.clone(),
        texture: wave_texture.clone(),
        transform: Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        ..default()
    }).insert(Wave {
        visibility_start: Timer::from_seconds(54., false),
        ..default()
    });

    commands.spawn_bundle(SpriteBundle {
        sprite: wave_sprite.clone(),
        texture: wave_texture.clone(),
        transform: Transform::from_xyz(0., 0., LAYER_WAVE).with_scale(Vec3::ZERO),
        ..default()
    }).insert(Wave {
        visibility_start: Timer::from_seconds(79., false),
        ..default()
    });

    commands.spawn_bundle(SpriteBundle {
        sprite: wave_sprite.clone(),
        texture: wave_texture.clone(),
        transform: Transform::from_xyz(-140., 300., LAYER_WAVE).with_scale(Vec3::ZERO),
        ..default()
    }).insert(Wave {
        visibility_start: Timer::from_seconds(99.5, false),
        ..default()
    });

    let rotating_soup_mesh: Mesh2dHandle = meshes.add(shape::Circle::new(ROTATING_SOUP_RADIUS).into()).into();
    let rotating_soup_material = materials.add(ColorMaterial::from(Color::rgba(0.0, 0.0, 0.0, 0.3)));

    for i in 1..=5 {
        let radius = (i as f32) / 5. * (HEIGHT / 2. - 20.);
        let theta = i as f32 * 6. * PI / 5.;
        let mut dtheta = (7. - (i as f32)) / 5. * ROTATING_SOUP_DTHETA;
        if i % 2 == 0 {
            dtheta = -dtheta;
        }

        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: rotating_soup_mesh.clone(),
            material: rotating_soup_material.clone(),
            transform: Transform::from_xyz(0., radius, LAYER_ROTATING_SOUP),
            ..default()
        })
        .insert(RotatingSoup {
            radius,
            theta,
            dtheta,
        })
        .insert(CollisionRadius(ROTATING_SOUP_RADIUS))
        .insert(Soup { damage: 5. });
    }

    setup_claw_swipes(
        &mut commands,
        &mut meshes,
        &mut materials,
        vec![22., 62., 107.]
    );

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(BIGBOY_RADIUS * 2., BIGBOY_RADIUS * 2.)),
            ..default()
        },
        texture: asset_server.load("wyvern.png"),
        transform: Transform::from_xyz(400., 0., LAYER_MOB),
        ..default()
    })
    .insert(MobWyvern {
        shoot_cooldown: Timer::from_seconds(1., true),
        shockwave_cooldown: Timer::from_seconds(18., true),
        charge_cooldown: Timer::from_seconds(11., true),
    })
    .insert(Enemy)
    .insert(Hp(20.))
    .insert(CollisionRadius(BIGBOY_RADIUS));

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(BIGBOY_RADIUS * 2., BIGBOY_RADIUS * 2.)),
            ..default()
        },
        texture: asset_server.load("goliath.png"),
        transform: Transform::from_xyz(300., 0., LAYER_MOB),
        ..default()
    })
    .insert(MobGoliath {
        shoot_cooldown: Timer::from_seconds(5., true),
    })
    .insert(Enemy)
    .insert(Hp(20.))
    .insert(Velocity(Vec3::ZERO))
    .insert(CollisionRadius(BIGBOY_RADIUS));
}

fn greens_system(time: Res<Time>,
                 mut greens: Query<(&mut StackGreen, &mut Visibility, &Children)>,
                 mut indicators: Query<(&StackGreenIndicator, &mut Transform), Without<StackGreen>>,
                 ) {
    for (mut green, mut visibility, children) in &mut greens {
        let mut visible = true;
        if !green.visibility_start.finished() {
            green.visibility_start.tick(time.delta());
            visible = false;
        } else {
            green.detonation.tick(time.delta());
        }

        if green.detonation.finished() {
            visible = false;
        }

        visibility.is_visible = visible;

        if !visible {
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

fn greens_detonation_system(mut game: ResMut<Game>,
                 players: Query<&Transform, With<PlayerTag>>,
                 greens: Query<(&StackGreen, &Children)>,
                 indicators: Query<(&StackGreenIndicator, &Transform)>,
                 ) {
    for (green, children) in &greens {
        if green.detonation.just_finished() {
            let transform_player = players.single();
            let mut any_collide = false;

            for &child in children.iter() {
                if let Ok((_, transform_indicator)) = indicators.get(child) {
                    any_collide = any_collide || collide(transform_player.translation, 0., transform_indicator.translation, GREEN_RADIUS);
                }
                if any_collide {
                    break;
                }
            }

            if !any_collide {
                game.player.hp = 0.;
            }
        }
    }
}

fn aoes_system(
    time: Res<Time>,
    mut aoes: Query<(&mut Aoe, &mut Visibility, &Children)>,
    mut indicators: Query<(&AoeIndicator, &mut Transform)>,
    ) {

    for (mut aoe, mut visibility, children) in &mut aoes {
        let mut visible = false;
        match &mut aoe.visibility_start {
            Some(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    visible = true;
                }
            },
            None => {
                visible = true;
            }
        }
        visibility.is_visible = visible;

        if !visible {
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

fn aoes_detonation_system(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    players: Query<(Entity, &Transform), With<PlayerTag>>,
    aoes: Query<(Entity, &Aoe, &Transform, &CollisionRadius)>,
    ) {

    for (entity_aoe, aoe, transform, radius) in &aoes {
        if !aoe.detonation.just_finished() {
            continue;
        }

        let (entity_player, transform_player) = players.single();
        let player_pos = transform_player.translation;
        let hit = collide(transform.translation, radius.0, player_pos, 0.);

        if hit {
            game.player.hp -= aoe.damage;
            damage_flash_events.send(DamageFlashEvent {
                entity: entity_player,
            });
        }
        commands.entity(entity_aoe).despawn_recursive();
    }
}

fn aoes_follow_system(
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

fn puddles_system(time: Res<Time>,
    players: Query<&Transform, (With<PlayerTag>, Without<Puddle>)>,
    mut puddles: Query<(&mut Puddle, &mut Soup, &mut Transform, &mut Visibility, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    for (mut puddle, mut soup, mut transform, mut visibility, material) in &mut puddles {
        if puddle.drop.finished() {
            continue;
        }

        if !puddle.visibility_start.finished() {
            puddle.visibility_start.tick(time.delta());
        } else {
            puddle.drop.tick(time.delta());

            if puddle.drop.percent() < 4. / 6. {
                let transform_player = players.single();
                transform.translation = transform_player.translation;
            }
        }

        if puddle.visibility_start.just_finished() {
            visibility.is_visible = true;
        }

        if puddle.drop.just_finished() {
            soup.damage = PUDDLE_DAMAGE;
            materials.get_mut(material).unwrap().color.set_a(0.9);
        } else if puddle.drop.percent() > 4. / 6. {
            materials.get_mut(material).unwrap().color.set_a(0.7);
        }
    }
}

fn waves_system(
    time: Res<Time>,
    mut waves: Query<(&mut Wave, &mut Visibility, &mut Transform)>,
    ) {
    for (mut wave, mut visibility, mut transform) in &mut waves {
        let mut visible = true;
        if !wave.visibility_start.finished() {
            wave.visibility_start.tick(time.delta());
            visible = false;
        } else {
            wave.growth.tick(time.delta());
        }

        if wave.growth.finished() {
            visible = false;
        }

        visibility.is_visible = visible;

        if !visible {
            continue;
        }

        transform.scale = Vec3::splat(wave.growth.percent());
    }
}

fn goliath_system(
    time: Res<Time>,
    mut commands: Commands,
    mut goliaths: Query<(&mut MobGoliath, &Transform, &mut Velocity), Without<EffectForcedMarch>>,
    players: Query<&Transform, With<PlayerTag>>,
    ) {
    let player = players.single();
    for (mut goliath, transform, mut velocity) in &mut goliaths {
        let mut vel = player.translation.sub(transform.translation);
        vel.z = 0.;
        vel = vel.clamp_length(0., GOLIATH_MOVE_SPEED);
        velocity.0 = vel;

        let bullet_radius = BULLET_SIZE * 4.;
        let bullet_speed = GOLIATH_BULLET_SPEED;
        vel = vel.clamp_length(bullet_speed, bullet_speed);

        goliath.shoot_cooldown.tick(time.delta());
        if goliath.shoot_cooldown.finished() {
            goliath.shoot_cooldown.reset();

            commands.spawn_bundle(SpriteBundle {
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
            .insert(CollisionRadius(bullet_radius));
        }
    }
}

fn wyvern_system(
    time: Res<Time>,
    mut commands: Commands,
    mut wyverns: Query<(Entity, &mut MobWyvern, &Transform), Without<EffectForcedMarch>>,
    players: Query<&Transform, With<PlayerTag>>,
    ) {
    let player = players.single();
    for (entity, mut wyvern, transform) in &mut wyverns {
        let mut vel = player.translation.sub(transform.translation);
        vel.z = 0.;
        let bullet_speed = WYVERN_BULLET_SPEED;
        vel = vel.clamp_length(bullet_speed, bullet_speed);


        wyvern.shoot_cooldown.tick(time.delta());
        if wyvern.shoot_cooldown.finished() {
            wyvern.shoot_cooldown.reset();

            commands.spawn_bundle(SpriteBundle {
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
            .insert(CollisionRadius(BULLET_SIZE / 2.));
        }

        wyvern.shockwave_cooldown.tick(time.delta());
        if wyvern.shockwave_cooldown.finished() {
            wyvern.shockwave_cooldown.reset();

            for bullet_i in 0..16 {
                let theta = (bullet_i as f32) / 16. * 2. * PI;
                let vel = Vec3::new(theta.cos() * WYVERN_BULLET_SPEED, theta.sin() * WYVERN_BULLET_SPEED, 0.);
                let bullet_radius = BULLET_SIZE / 3.;

                commands.spawn_bundle(SpriteBundle {
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
                .insert(CollisionRadius(bullet_radius));
            }

        }

        wyvern.charge_cooldown.tick(time.delta());
        if wyvern.charge_cooldown.finished() {
            wyvern.charge_cooldown.reset();
            let speed = WYVERN_CHARGE_RANGE / 0.75;
            let diff = player.translation.sub(transform.translation).clamp_length(0., WYVERN_CHARGE_RANGE);
            let target = transform.translation.add(diff);

            commands.entity(entity).insert(EffectForcedMarch {
                target,
                speed,
            });
        }
    }
}


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

fn move_player_system(time: Res<Time>, keyboard_input: Res<Input<KeyCode>>,
               mut transforms: Query<&mut Transform, (With<PlayerTag>, Without<EffectForcedMarch>)>) {
    // Much slower than actual movement
    let speed = 250.0 * GAME_TO_PX * time.delta_seconds();
    for mut transform in &mut transforms {
        let mut movement = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            movement.y += speed;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            movement.y -= speed;
        }
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            movement.x -= speed;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            movement.x += speed;
        }
        movement.clamp_length(0., speed);
        transform.translation = transform.translation.add(movement);
    }
}

fn effect_forced_march_system(time: Res<Time>, mut commands: Commands, mut pulleds: Query<(Entity, &mut Transform, &EffectForcedMarch)>) {
    for (ent, mut transform, effect) in &mut pulleds {
        let target = effect.target;
        let mut diff = target.sub(transform.translation);
        diff.z = 0.;
        let speed = effect.speed * time.delta_seconds();
        let vel = diff.clamp_length(speed, speed);
        if vel.length_squared() > diff.length_squared() {
            transform.translation = target;
            commands.entity(ent).remove::<EffectForcedMarch>();
        } else {
            transform.translation = transform.translation.add(vel);
        }
    }
}

fn velocities_system(
    time: Res<Time>,
    mut commands: Commands,
    mut tra_vels: Query<(Entity, &mut Transform, &Velocity)>
    ) {
    for (entity, mut transform, velocity) in &mut tra_vels {
        transform.translation = transform.translation.add(velocity.0.mul(time.delta_seconds()));

        let pos = transform.translation;

        if pos.x < -WIDTH ||
           pos.x > WIDTH ||
           pos.y < -HEIGHT ||
           pos.y > HEIGHT {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn damage_flash_system(
    mut events: EventReader<DamageFlashEvent>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite, Without<TintUntint>>,
    ) {
    let mut touched = HashSet::new();

    for event in events.iter() {
        if touched.contains(&event.entity) {
            continue;
        }
        if let Ok(sprite) = sprites.get_mut(event.entity) {
            let prev_color = sprite.color.clone();
            touched.insert(event.entity);
            commands.entity(event.entity).insert(TintUntint {
                color: prev_color,
                tint_color: Color::rgba(1.0, 0., 0., 0.7),
                tint_timer: Timer::from_seconds(0.2, false),
                untint_timer: Timer::from_seconds(0.5, false),
            });
        }
    }
}

fn tint_untint_system(
    time: Res<Time>,
    mut commands: Commands,
    mut sprites: Query<(Entity, &mut TintUntint, &mut Sprite)>,
    ) {
    for (entity, mut tut, mut sprite) in &mut sprites {
        tut.tint_timer.tick(time.delta());
        tut.untint_timer.tick(time.delta());
        if !tut.tint_timer.finished() {
            sprite.color = tut.tint_color;
        } else {
            sprite.color = tut.color;
        }
        if tut.untint_timer.finished() {
            sprite.color = tut.color;
            commands.entity(entity).remove::<TintUntint>();
        }
    }
}

fn move_rotating_soup_system(
    time: Res<Time>,
    mut soups: Query<(&mut Transform, &mut RotatingSoup)>
    ) {
    for (mut transform, mut soup) in &mut soups {
        soup.theta += soup.dtheta * time.delta_seconds();
        transform.translation.x = soup.theta.cos() * soup.radius;
        transform.translation.y = soup.theta.sin() * soup.radius;
    }
}

fn jormag_soup_beam_system(
    time: Res<Time>,
    mut soups: Query<&mut RotatingSoup>
    ) {

    for mut soup in &mut soups {
        let radius = (WIDTH / 2. - 70.) * ((time.seconds_since_startup() as f32 / 8.).cos() + 1.) / 2. + 35.;
        soup.radius = radius;
    }
}

fn collide(pos_a: Vec3, radius_a: f32, pos_b: Vec3, radius_b: f32) -> bool {
    let mut diff = pos_b.sub(pos_a);
    diff.z = 0.;
    return diff.length_squared() < (radius_a + radius_b) * (radius_a + radius_b);
}

fn collisions_bullets_orbs_system(
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
                velocity_orb.0 = velocity_orb.0.add(diff.clamp_length(push_str, push_str)).clamp_length(0., orb_max_vel);
            }
        }
    }
}

fn collisions_bullets_enemies_system(
    mut damage_flash_events: EventWriter<DamageFlashEvent>,
    mut bullets: Query<(&Transform, &mut HasHit), (With<Bullet>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &Transform, &CollisionRadius, &mut Hp), (With<Enemy>, Without<Bullet>)>,
    ) {
    for (transform_bullet, mut has_hit) in &mut bullets {
        let bullet_pos = transform_bullet.translation;
        for (entity_enemy, transform_enemy, radius_enemy, mut hp) in &mut enemies {
            if has_hit.0.contains(&entity_enemy) {
                continue;
            }

            let enemy_pos = transform_enemy.translation;
            if !collide(bullet_pos, BULLET_SIZE / 2., enemy_pos, radius_enemy.0) {
                continue;
            }

            has_hit.0.insert(entity_enemy);
            hp.0 -= BULLET_DAMAGE;
            if hp.0 > 0. {
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_enemy,
                });
            }
        }
    }
}

fn collisions_crabs_orbs_system(
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
            }
        }
    }
}

fn collisions_orb_targets_system(
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
        state.set(GameState::Success).unwrap();
    }
}

fn collisions_players_edge_system(
    mut game: ResMut<Game>,
    players: Query<&Transform, (With<PlayerTag>, Without<MobOrb>)>,
    ) {
    let transform_player = players.single();
    if !collide(transform_player.translation, 0., Vec3::ZERO, MAP_RADIUS) {
        game.player.hp = 0.;
    }
}

fn collisions_players_soups_system(
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
        game.player.hp -= soup.damage * time.delta_seconds();
        if soup.damage > 0.1 {
            damage_flash_events.send(DamageFlashEvent {
                entity: entity_player,
            });
        }
    }
}

fn collisions_players_waves_system(
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
                damage_flash_events.send(DamageFlashEvent {
                    entity: entity_player,
                });
                // Brief invuln from being knocked (not actually knocked because Reasons)
                game.player.invuln = Timer::from_seconds(1., false);
            }
        }
    }
}

fn collisions_orbs_edge_system(
    mut game: ResMut<Game>,
    orbs: Query<(&MobOrb, &Transform)>,
    ) {
    for (_, transform_orb) in &orbs {
        if !collide(transform_orb.translation, 0., Vec3::ZERO, MAP_RADIUS - ORB_RADIUS) {
            game.player.hp = 0.;
        }
    }
}

fn collisions_players_enemy_bullets_system(
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

fn enemies_hp_check_system(
    mut commands: Commands,
    enemies: Query<(Entity, &Hp), With<Enemy>>,
    ) {
    for (entity_enemy, hp) in &enemies {
        if hp.0 <= 0. {
            commands.entity(entity_enemy).despawn_recursive();
        }
    }
}

fn boss_existence_check_system(
    bosses: Query<&Boss>,
    mut state: ResMut<State<GameState>>,
    ) {
    if let Ok(_) = bosses.get_single() {
        return;
    }
    state.set(GameState::Success).unwrap();
}

fn boss_healthbar_system(
    bosses: Query<(&Boss, &Hp)>,
    mut boss_healthbars: Query<&mut Transform, With<BossHealthbar>>,
    mut texts: Query<&mut Text, With<BossHealthbarText>>,
    ) {
    if let Err(_) = bosses.get_single() {
        return;
    }

    let (boss, boss_hp) = bosses.single();
    let remaining = boss_hp.0 / boss.max_hp;
    for mut transform in &mut boss_healthbars {
        transform.scale.x = remaining;
    }
    for mut text in &mut texts {
        let left = remaining * 100.;
        text.sections[0].value = format!("{left:.0}");
    }
}

fn handle_mouse_events_system(
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut cursor_moved_events: EventReader<CursorMoved>,
    players: Query<&Transform, (With<PlayerTag>, Without<CursorMark>)>,
    mut cursors: Query<&mut Transform, With<CursorMark>>,
    mut game: ResMut<Game>,
    time: Res<Time>) {

    let player_loc = {
        let transform = players.single();
        transform.translation
    };

    game.player.shoot_cooldown.tick(time.delta());
    game.player.dodge_cooldown.tick(time.delta());
    game.player.pull_cooldown.tick(time.delta());
    game.player.blink_cooldown.tick(time.delta());
    game.player.portal_cooldown.tick(time.delta());
    game.player.jump_cooldown.tick(time.delta());
    game.player.invuln.tick(time.delta());
    game.player.jump.tick(time.delta());
    game.player.hp += time.delta_seconds() * PLAYER_REGEN;
    if game.player.hp > 100. {
        game.player.hp = 100.;
    }

    if game.player.shoot_cooldown.finished() &&
       (mouse_button_input.pressed(MouseButton::Left) ||
        keyboard_input.pressed(KeyCode::Key1)) {
        let cursor = cursors.single();
        let mut vel = cursor.translation.sub(player_loc);
        vel.z = 0.;
        let bullet_speed = 200.0;
        vel = vel.clamp_length(bullet_speed, bullet_speed);

        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.89, 0.39, 0.95),
                custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(player_loc),
            ..default()
        }).insert(Velocity(vel))
          .insert(Bullet)
          .insert(HasHit(HashSet::new()));
        game.player.shoot_cooldown.reset();
    }

    for event in cursor_moved_events.iter() {
        let mut cursor = cursors.single_mut();
        // info!("{:?}", event);
        // let mut cursor_transform = cursors.single_mut();
        cursor.translation.x = event.position.x - WIDTH / 2.;
        cursor.translation.y = event.position.y - HEIGHT / 2.;
    }
}

fn handle_spellcasts_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    players: Query<&Transform, (With<PlayerTag>, Without<CursorMark>)>,
    cursors: Query<&Transform, With<CursorMark>>,
    crabs: Query<(Entity, &Transform, &MobCrab)>,
    mut game: ResMut<Game>) {

    let player_loc = {
        let transform = players.single();
        transform.translation
    };

    let cursor_loc = cursors.single().translation;

    if game.player.jump_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::Space) {

        game.player.jump = Timer::from_seconds(0.5, false);
        game.player.jump_cooldown.reset();
    }

    if game.player.dodge_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::V) {
        let dodge_range = 300. * GAME_TO_PX;
        let dodge_speed = dodge_range / 0.75;
        let diff = cursor_loc.sub(player_loc).clamp_length(dodge_range, dodge_range);
        let target = player_loc.add(diff);


        commands.entity(game.player.entity.unwrap()).insert(EffectForcedMarch {
            target,
            speed: dodge_speed,
        });

        game.player.invuln = Timer::from_seconds(0.75, false);
        game.player.dodge_cooldown.reset();
    }

    if game.player.blink_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::E) {
        let blink_range = 1200.0 * GAME_TO_PX;
        let blink_speed = blink_range / 0.1;
        let mut diff = cursor_loc.sub(player_loc);
        diff.z = 0.;
        diff = diff.clamp_length(0., blink_range);
        let target = player_loc.add(diff);


        commands.entity(game.player.entity.unwrap()).insert(EffectForcedMarch {
            target,
            speed: blink_speed,
        });

        game.player.invuln = Timer::from_seconds(0.1, false);
        game.player.blink_cooldown.reset();
    }

    if game.player.pull_cooldown.finished() &&
        keyboard_input.pressed(KeyCode::Key4) {
        let pull_loc = cursor_loc;
        let pull_range = 600.0 * GAME_TO_PX;
        let pull_speed = pull_range / 0.3;

        for (entity_crab, transform_crab, _) in &crabs {
            let crab_loc = transform_crab.translation;
            let mut diff = crab_loc.sub(pull_loc);
            diff.z = 0.;
            if diff.length_squared() > pull_range * pull_range {
                continue;
            }

            let target = pull_loc;
            commands.entity(entity_crab).insert(EffectForcedMarch {
                target,
                speed: pull_speed,
            });
        }

        game.player.pull_cooldown.reset();
    }
}

fn set_cooldown_text_display(timer: &Timer, text: &mut Text, text_display: &TextDisplay, sprites: &mut Query<&mut Sprite>) {
    let dur = timer.duration().as_secs_f32();
    let elapsed = timer.elapsed_secs();
    let left = dur - elapsed;

    if left < 5. {
        text.sections[0].value = format!("{left:.1}");
    } else {
        text.sections[0].value = format!("{left:.0}");
    }

    if left < 0.001 {
        text.sections[0].style.color.set_a(0.0);
    } else {
        text.sections[0].style.color.set_a(1.0);
    }

    if let Some(sprite_handle) = text_display.sprite {
        let color = if left < 0.001 {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.7, 0.7, 0.7, 0.7)
        };
        sprites.get_mut(sprite_handle).unwrap().color = color;
    }
}

fn text_system(game: Res<Game>,
               mut text_displays: Query<(&mut Text, &TextDisplay)>,
               mut sprites: Query<&mut Sprite>) {
    for (mut text, text_display) in &mut text_displays {
        match text_display.value {
            TextValue::Hp => {
                let hp = game.player.hp;
                text.sections[0].value = format!("{hp:.0}");
            },
            TextValue::CooldownBlink => {
                set_cooldown_text_display(&game.player.blink_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownDodge => {
                set_cooldown_text_display(&game.player.dodge_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::StatusJump => {
                set_cooldown_text_display(&game.player.jump, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownPortal => {
                set_cooldown_text_display(&game.player.portal_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownPull => {
                set_cooldown_text_display(&game.player.pull_cooldown, &mut text, &text_display, &mut sprites);
            }
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

fn void_zone_growth_system(
    time: Res<Time>,
    mut void_zone_growths: Query<&mut VoidZoneGrowth>,
    mut void_zones: Query<(&mut CollisionRadius, &mut Transform), With<VoidZone>>
    ) {
    let void_zone_growth = &mut void_zone_growths.single_mut().0;

    void_zone_growth.tick(time.delta());

    let growing = void_zone_growth.just_finished();

    for (mut collision_radius, mut transform) in &mut void_zones {
        if growing {
            collision_radius.0 += VOID_ZONE_GROWTH_AMOUNT;
            let new_scale = collision_radius.0 / VOID_ZONE_START_RADIUS;
            transform.scale.x = new_scale;
            transform.scale.y = new_scale;
        }
    }
}

fn player_hp_check_system(game: ResMut<Game>,
                          mut state: ResMut<State<GameState>>,
                          ) {
    if game.player.hp <= 0.1 {
        state.set(GameState::Failure).unwrap();
    }
}

fn build_update_phase(phase: GameState) -> SystemSet {
    SystemSet::on_update(phase)
        .with_system(handle_mouse_events_system)
        .with_system(handle_spellcasts_system)
        .with_system(velocities_system)
        .with_system(move_player_system)
        .with_system(move_rotating_soup_system)
        .with_system(effect_forced_march_system)
        .with_system(collisions_players_edge_system)
        .with_system(collisions_bullets_enemies_system)
        .with_system(collisions_players_soups_system)
        .with_system(collisions_players_enemy_bullets_system)
        .with_system(text_system)
        .with_system(enemies_hp_check_system)
        .with_system(damage_flash_system)
        .with_system(tint_untint_system.after(damage_flash_system))
        .with_system(void_zone_growth_system)
        .with_system(player_hp_check_system)
}

fn build_update_boss_phase(phase: GameState) -> SystemSet {
    SystemSet::on_update(phase)
        .with_system(collisions_players_waves_system)
        .with_system(greens_system)
        .with_system(greens_detonation_system)
        .with_system(spread_aoe_spawn_system)
        .with_system(aoes_system)
        .with_system(aoes_detonation_system)
        .with_system(aoes_follow_system)
        .with_system(waves_system)
        .with_system(boss_existence_check_system)
        .with_system(boss_healthbar_system)
        .with_system(puddles_system)
}

fn build_update_purification_phase(phase: GameState) -> SystemSet {
    SystemSet::on_update(phase)
        .with_system(move_crabs_system)
        .with_system(collisions_crabs_orbs_system)
        .with_system(collisions_bullets_orbs_system)
        .with_system(collisions_orb_targets_system)
        .with_system(collisions_orbs_edge_system)
        .with_system(game_orb_target_progression_system)
        .with_system(void_zone_crab_system)
}

fn main() {
    let game = Game {
        player: Player {
            ..default()
        },
        orb_target: -1,
    };

    App::new()
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            scale_factor_override: Some(1.),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_state(GameState::StartMenu)

        .add_event::<DamageFlashEvent>()

        .insert_resource(game)

        .add_startup_system(setup)

        .add_system_set(SystemSet::on_enter(GameState::StartMenu).with_system(setup_menu_system))
        .add_system_set(SystemSet::on_update(GameState::StartMenu).with_system(update_menu_system))
        .add_system_set(SystemSet::on_exit(GameState::StartMenu).with_system(cleanup_menu_system))

        .add_system_set(SystemSet::on_enter(GameState::Success).with_system(setup_success_system))
        .add_system_set(SystemSet::on_enter(GameState::Failure).with_system(setup_failure_system))

        .add_system_set(SystemSet::on_enter(GameState::PurificationOne)
                        .with_system(setup_phase)
                        .with_system(setup_purification_one.after(setup_phase)))
        .add_system_set(build_update_phase(GameState::PurificationOne))
        .add_system_set(build_update_purification_phase(GameState::PurificationOne))

        .add_system_set(SystemSet::on_enter(GameState::Jormag)
                        .with_system(setup_phase)
                        .with_system(setup_jormag.after(setup_phase)))
        .add_system_set(build_update_phase(GameState::Jormag))
        .add_system_set(build_update_boss_phase(GameState::Jormag))
        .add_system_set(SystemSet::on_update(GameState::Jormag)
            .with_system(jormag_soup_beam_system))

        .add_system_set(SystemSet::on_enter(GameState::Mordremoth)
                        .with_system(setup_phase)
                        .with_system(setup_mordremoth.after(setup_phase)))
        .add_system_set(build_update_phase(GameState::Mordremoth))
        .add_system_set(build_update_boss_phase(GameState::Mordremoth))
        // .add_system_set(SystemSet::on_update(GameState::Mordremoth)
        //     .with_system(goliath_system)
        //     .with_system(wyvern_system))

        .add_system_set(SystemSet::on_enter(GameState::SooWonTwo)
                        .with_system(setup_phase)
                        .with_system(setup_soowontwo.after(setup_phase)))
        .add_system_set(build_update_phase(GameState::SooWonTwo))
        .add_system_set(build_update_boss_phase(GameState::SooWonTwo))
        .add_system_set(SystemSet::on_update(GameState::SooWonTwo)
            .with_system(goliath_system)
            .with_system(wyvern_system))

        .run();
}
