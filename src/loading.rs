use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::{MenuContainer, MenuState, HEIGHT, WIDTH};

#[derive(Resource)]
pub struct AssetsLoading(pub Vec<UntypedHandle>);

#[derive(Component)]
pub struct LoadingText;

pub fn setup_loading_system(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut loading: ResMut<AssetsLoading>,
) {
    let font: Handle<Font> = asset_server.load("trebuchet_ms.ttf");
    loading.0.push(font.untyped());

    let images = vec![
        "blink.png",
        "crab.png",
        "dps1.png",
        "dps2.png",
        "dps3.png",
        "dps4.png",
        "dps.png",
        "echo.png",
        "goliath.png",
        "ham1.png",
        "ham2.png",
        "ham.png",
        "herald1.png",
        "herald2.png",
        "herald.png",
        "map.png",
        "noodle.png",
        "portal_exit.png",
        "portal.png",
        "pull.png",
        "reaper.png",
        "ring.png",
        "saltspray.png",
        "timecaster.png",
        "virt1.png",
        "virt2.png",
        "virt.png",
        "wave.png",
        "wyvern.png",
    ];
    for image_path in images {
        let image: Handle<Image> = asset_server.load(image_path);
        loading.0.push(image.untyped());
    }
    let sounds = vec![
        "sounds/blink.ogg",
        "sounds/enemy_shoot_big.ogg",
        "sounds/enemy_shoot.ogg",
        "sounds/green_pop.ogg",
        "sounds/hurt.ogg",
        "sounds/jump.ogg",
        "sounds/menu_click.ogg",
        "sounds/orb_hit_edge.ogg",
        "sounds/orb_hit_target.ogg",
        "sounds/phase_theme.ogg",
        "sounds/portal_enter.ogg",
        "sounds/portal_exit.ogg",
        "sounds/pull.ogg",
        "sounds/red_target.ogg",
        "sounds/roar.ogg",
        "sounds/shockwave.ogg",
        "sounds/shoot.ogg",
    ];
    for sound_path in sounds {
        let sound: Handle<AudioSource> = asset_server.load(sound_path);
        loading.0.push(sound.untyped());
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(WIDTH),
                height: Val::Px(HEIGHT),
                margin: UiRect::all(Val::Auto), // UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(HEIGHT / 4.)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|big_container| {
            let text_style = TextStyle {
                font: asset_server.load("trebuchet_ms.ttf"),
                font_size: 80.,
                color: Color::rgb(0.9, 0.9, 0.9),
            };

            big_container
                .spawn(TextBundle::from_section(
                    "Loading...".to_string(),
                    text_style.clone(),
                ))
                .insert(LoadingText);
        })
        .insert(MenuContainer);
}

pub fn update_loading_system(
    asset_server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut texts: Query<&mut Text, With<LoadingText>>,
    mut res_next_menu_state: ResMut<NextState<MenuState>>,
) {
    let loading_len = loading.0.len();
    let load_states = loading
        .0
        .iter()
        .map(|h| asset_server.get_load_state(h.id()));
    let loaded_count = load_states
        .map(|ls| match ls {
            Some(LoadState::Loaded) => 1,
            _ => 0,
        })
        .reduce(|a, b| a + b);

    let loaded_count = loaded_count.unwrap_or(0);
    let all_loaded = loaded_count == loading_len;

    if all_loaded {
        // Would unload assets which could be good for memory if necessary but also brings
        // back the brief flash of unloaded assets
        // commands.remove_resource::<AssetsLoading>();
        res_next_menu_state.set(MenuState::StartMenu);
        return;
    }
    for mut text in &mut texts {
        text.sections[0].value = format!("Loading: {}/{}", loaded_count, loading_len);
    }
}
