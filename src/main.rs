use std::time::Duration;

use animation::{MuzzleFlash, TextEvent, WobbleEffect, WobbleEffects};
use bevy::asset::AssetMetaCheck;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use map::{Map, MapPos, Tile, TileKind, Zones};
use mob::{Mob, MobDamageEvent};
use player::{Inventory, Player, PlayerDamageEvent, ShootEvent};
use spawn::{Spawn, SpawnEvent};
use ui::{UiEvent, UiSettings};

mod animation;
mod assets;
mod despawn_after;
mod edge;
mod intro;
mod lighting;
mod map;
mod mapgen;
mod mob;
mod player;
mod renderer;
mod sdf;
mod sound;
mod spawn;
mod ui;

pub const SDF_RES: u32 = 768;

const CAMERA_DECAY_RATE: f32 = 2.;

// Z-coordinates for everything in the game world.
const Z_PLAYER: f32 = 1.0;
const Z_TILES: f32 = 3.0;
const Z_ITEMS: f32 = 4.0;
const Z_MOBS: f32 = 2.0;
const Z_TEXT: f32 = 9.0;

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}

#[derive(Component)]
struct PrimaryCamera;

#[derive(Component)]
struct CameraFollow;

fn create_texture() -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            // width: window.resolution.physical_width(), // does this work?
            // height: window.resolution.physical_height(),
            width: SDF_RES,
            height: SDF_RES,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image
}

fn create_camera(
    mut window: Single<&mut Window>,
    mut commands: Commands,
    // camera_query: Query<(Entity, &Camera), With<PrimaryCamera>>,
    mut resize_reader: EventReader<bevy::window::WindowResized>,
    mut images: ResMut<Assets<Image>>,
) {
    window.resizable = true;

    let image_handle = images.add(create_texture());
    let image_handle_ui = images.add(create_texture());

    let camera = Camera {
        target: image_handle.clone().into(),
        clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
        hdr: true,
        order: 0,
        ..default()
    };
    let camera_ui = Camera {
        target: image_handle_ui.clone().into(),
        clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
        hdr: true,
        order: 0,
        ..default()
    };

    let texture_cpu_occluder = renderer::OccluderTexture(image_handle);
    commands.spawn(texture_cpu_occluder.clone());

    let texture_cpu_nonoccluder = renderer::NonOccluderTexture(image_handle_ui);
    commands.spawn(texture_cpu_nonoccluder.clone());

    commands.spawn((
        Camera2d,
        camera,
        RenderLayers::layer(1),
        PrimaryCamera,
        CameraFollow,
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
        OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        },
    ));

    commands.spawn((
        Camera2d,
        CameraFollow,
        camera_ui,
        RenderLayers::layer(crate::lighting::UI_LAYER),
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
        OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        },
    ));

    for e in resize_reader.read() {
        println!("Resize happened {:?}", e);
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<CameraFollow>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    time: Res<Time>,
    mut ev_scroll: EventReader<MouseWheel>,
    ui_settings: Res<UiSettings>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    // let Ok(mut camera) = camera.get_single_mut() else {
    //    return;
    //};
    for mut camera in camera.iter_mut() {
        let Vec3 { x, y, .. } = player.translation;
        let direction = Vec3::new(x, y, camera.translation.z);

        camera
            .translation
            .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());

        for event in ev_scroll.read() {
            if ui_settings.debug_scroll {
                let factor = match event.unit {
                    MouseScrollUnit::Line => 0.2,
                    MouseScrollUnit::Pixel => 0.01,
                };
                camera.scale -= event.y * factor;
            }
        }
    }
}

#[derive(Resource)]
struct GameState {
    game_over: bool,
    #[allow(unused)]
    victory: bool,
}

fn setup(mut window: Query<&mut Window>) {
    window.single_mut().resizable = true;
    window.single_mut().fit_canvas_to_parent = true;
}

/*fn handle_game_over(
    mut commands: Commands,
    game_state: Res<GameState>,
    fade_out: Query<&FadeOut>,
    assets: Res<GameAssets>,
) {
    if game_state.game_over && fade_out.is_empty() {
        commands.spawn((
            FadeOut,
            Mesh2d(assets.square.clone()),
            MeshMaterial2d(assets.fade_out_material.clone()),
            RenderLayers::layer(1),
            PlayerInjuryEffect {
                timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
                ease: EasingCurve::new(0.0, 1.0, EaseFunction::CubicOut),
            },
            Transform::from_translation(Vec3::ZERO.with_z(10.0)).with_scale(Vec3::splat(99999.0)),
        ));
    }
}*/

fn animate_player_damage(
    mut query: Query<&mut WobbleEffects, With<Player>>,
    mut ev_player_damage: EventReader<PlayerDamageEvent>,
) {
    if ev_player_damage.read().count() > 0 {
        query.single_mut().effects.push(WobbleEffect {
            timer: Timer::new(Duration::from_millis(200), TimerMode::Once),
            ease: EasingCurve::new(1.0, 0.0, EaseFunction::ElasticInOut),
        });
    }
}

fn animate_mob_damage(
    mut query: Query<(&mut WobbleEffects, &Mob)>,
    mut ev_mob_damage: EventReader<MobDamageEvent>,
) {
    for ev in ev_mob_damage.read() {
        if let Ok((mut wobble, mob)) = query.get_mut(ev.entity) {
            if mob.kind.max_damage() < 99 {
                wobble.effects.push(WobbleEffect {
                    timer: Timer::new(Duration::from_millis(200), TimerMode::Once),
                    ease: EasingCurve::new(1.0, 0.0, EaseFunction::ElasticInOut),
                });
            }
        }
    }
}

fn animate_muzzle_flash(
    mut commands: Commands,
    mut query: Query<&mut MuzzleFlash>,
    gun: Res<Inventory>,
    mut ev_shoot_event: EventReader<ShootEvent>,
) {
    if ev_shoot_event.read().count() > 0 {
        let timer = Timer::new(Duration::from_millis(100), TimerMode::Once);
        let info = gun.equipped.get_info();
        if let Ok(mut flash) = query.get_single_mut() {
            flash.timer = timer;
            flash.info = info;
        } else {
            commands.spawn(MuzzleFlash {
                timer,
                ease: EasingCurve::new(0.25, 0.0, EaseFunction::CubicInOut),
                info,
            });
        };
    }
}

fn handle_ui_event(
    mut ev: EventReader<UiEvent>,
    mut ev_spawn: EventWriter<SpawnEvent>,
    zones: Res<Zones>,
    mut player_query: Query<(&mut Transform, &mut map::MapPos), With<Player>>,
) {
    for ev in ev.read() {
        match ev {
            UiEvent::TeleportPlayer(zone_idx) => {
                if let Some(zone) = zones.0.get(*zone_idx) {
                    let (mut transform, mut map_pos) = player_query.single_mut();
                    let dest = zone.center();
                    map_pos.0 = dest;
                    transform.translation = map_pos.to_vec2().extend(transform.translation.z);
                }
            }
            UiEvent::Spawn(spawn) => {
                ev_spawn.send(SpawnEvent(
                    player_query.single().1.0 + IVec2::new(1, 0),
                    spawn.clone(),
                ));
            }
        }
    }
}

fn handle_victory(
    mut commands: Commands,
    player_query: Query<&map::MapPos, With<Player>>,
    tile_query: Query<(Entity, &Tile), Without<Player>>,
    mut state: ResMut<GameState>,
    map: Res<Map>,
    mut ev_text: EventWriter<TextEvent>,
    mut ev_spawn: EventWriter<SpawnEvent>,
) {
    let player_pos = player_query.single();
    let mut iter = tile_query.iter_many(map.get(player_pos.0));
    while let Some((entity, tile)) = iter.fetch_next() {
        if matches!(tile.0, TileKind::Lever) {
            commands.entity(entity).despawn_recursive();
            state.victory = true;
            ev_spawn.send(SpawnEvent(player_pos.0, Spawn::Tile(TileKind::LeverPulled)));
            ev_text.send(TextEvent {
                text: "You win!".into(),
                position: MapPos(player_pos.0 + IVec2::new(0, 1)).to_vec2(),
                duration: Duration::from_secs(10),
                ..default()
            });
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins((
            ui::UiPlugin,
            ui::performance::PerformanceUiPlugin,
            // LogDiagnosticsPlugin::default(),
            assets::AssetsPlugin,
            intro::IntroPlugin,
            map::WorldPlugin,
            animation::AnimatePlugin,
            renderer::Renderer,
            spawn::SpawnPlugin,
            player::PlayerPlugin,
            mob::MobPlugin,
            sound::SoundPlugin,
            despawn_after::DespawnAfterPlugin,
        ))
        .add_systems(Startup, (create_camera, setup))
        .add_systems(
            Update,
            (
                handle_ui_event,
                update_camera,
                on_resize,
                animate_player_damage,
                animate_mob_damage,
                animate_muzzle_flash,
                handle_victory,
                //handle_game_over,
            )
                .chain(),
        )
        .insert_resource(GameState {
            game_over: false,
            victory: false,
        })
        .run();
}
