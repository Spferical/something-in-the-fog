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
use map::{LightsUp, Map, MapPos, Tile, TileKind, Zones};
use mob::{Mob, MobDamageEvent, MobKind};
use player::{GunType, Inventory, Player, PlayerDamageEvent, ShootEvent};
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

#[derive(Component)]
struct FadeOutEndScreen {
    color: Color,
    timer: Timer,
}

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
    last_known_boss_pos: Option<IVec2>,
    waves_spawned: usize,
    boss_dead: bool,
}

fn setup(mut window: Query<&mut Window>) {
    window.single_mut().resizable = true;
    window.single_mut().fit_canvas_to_parent = true;
}

fn handle_game_over(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    fade_out: Query<&FadeOutEndScreen>,
) {
    if (game_state.victory || game_state.game_over) && fade_out.get_single().is_err() {
        let color = if game_state.victory {
            Color::srgba(0.0, 0.0, 0.0, 0.0)
        } else {
            Color::srgba(1.0, 0.0, 0.0, 0.0)
        };
        commands.spawn((
            FadeOutEndScreen {
                color,
                timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
            },
            Mesh3d(meshes.add(Plane3d::default().mesh().size(6.0, 6.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.2, 0.0),
            RenderLayers::layer(crate::lighting::LIGHTING_LAYER),
        ));
    }
}

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

#[derive(Component)]
pub struct Eyeball;

fn final_boss(
    q_boss: Query<(&LightsUp, &MapPos), With<Eyeball>>,
    mut game_state: ResMut<GameState>,
    mut ev_spawn: EventWriter<SpawnEvent>,
    zones: Res<Zones>,
) {
    if let Ok((lit, pos)) = q_boss.get_single() {
        game_state.last_known_boss_pos = Some(pos.0);
        if (lit.lit_factor / 5.0) as usize > game_state.waves_spawned {
            let spawns = match game_state.waves_spawned {
                0 => vec![
                    (3, Spawn::Mob(MobKind::Zombie)),
                    (1, Spawn::Item(map::ItemKind::Ammo(GunType::Shotgun, 10))),
                ],
                1 => vec![
                    (3, Spawn::Mob(MobKind::Ghost)),
                    (1, Spawn::Item(map::ItemKind::Ammo(GunType::Shotgun, 10))),
                ],
                2 => vec![(1, Spawn::Mob(MobKind::KoolAidMan))],
                3 => vec![(1, Spawn::Mob(MobKind::Sculpture))],
                4 => vec![(3, Spawn::Mob(MobKind::Zombie))],
                5 => vec![(3, Spawn::Mob(MobKind::Hider))],
                _ => vec![],
            };
            for (count, spawn) in spawns {
                for _ in 0..count {
                    ev_spawn.send(SpawnEvent(pos.0, spawn.clone()));
                }
            }
            game_state.waves_spawned += 1;
        }
    } else if !game_state.boss_dead {
        game_state.boss_dead = true;
        // set the win tile
        let pos = game_state
            .last_known_boss_pos
            .unwrap_or(zones.0.iter().last().unwrap().center());
        ev_spawn.send(SpawnEvent(pos, Spawn::Tile(TileKind::Lever)));
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
                final_boss,
                handle_victory,
                handle_game_over,
            )
                .chain(),
        )
        .insert_resource(GameState {
            game_over: false,
            victory: false,
            boss_dead: false,
            waves_spawned: 0,
            last_known_boss_pos: None,
        })
        .run();
}
