use rand::seq::SliceRandom;
use std::{collections::HashMap, f32::consts::PI, time::Duration};

use bevy::{
    input::mouse::MouseWheel,
    math::bounding::{Aabb2d, RayCast2d},
    prelude::*,
    render::view::RenderLayers,
};
use rand::Rng as _;

use crate::{
    animation::{MoveAnimation, TextEvent, WobbleEffects},
    assets::{GameAssets, SpriteKind},
    despawn_after::DespawnAfter,
    lighting::UI_LAYER,
    map::{BlocksMovement, Map, MapPos, Pickup, Tile, TILE_HEIGHT, TILE_WIDTH},
    mob::{Mob, MobDamageEvent},
    renderer::PlaneMouseMovedEvent,
    ui::UiSettings,
    GameState, PrimaryCamera, SDF_RES, Z_PLAYER,
};

const PLAYER_MOVE_DELAY: Duration = Duration::from_millis(350);
const PLAYER_START: IVec2 = IVec2::new(0, 0);
const PLAYER_FOCUS_TIME_SECS: f32 = 2.0;
const PLAYER_MOVE_FOCUS_PENALTY_SECS: f32 = 1.0;
const PLAYER_SHOOT_FOCUS_PENALTY_SECS: f32 = 0.5;
pub const PLAYER_MAX_DAMAGE: i32 = 8;
pub const FLASHLIGHT_CONE_WIDTH_FOCUSED_DEGREES: f32 = 20.0;
pub const FLASHLIGHT_CONE_WIDTH_UNFOCUSED_DEGREES: f32 = 40.0;
pub const FLASHLIGHT_MAX_BATTERY: f32 = 1.0;

#[derive(Component)]
pub struct Player {
    pub damage: i32,
}

impl Player {
    pub fn is_dead(&self) -> bool {
        self.damage >= PLAYER_MAX_DAMAGE
    }
}

#[derive(Event)]
pub struct PlayerDamageEvent {
    pub damage: i32,
}

#[derive(Resource)]
pub struct PlayerDamageState {
    timer: Timer,
}

fn damage_player(
    mut player: Query<&mut Player>,
    mut ev_player_damage: EventReader<PlayerDamageEvent>,
    mut game_state: ResMut<GameState>,
    mut state: ResMut<PlayerDamageState>,
    settings: Res<UiSettings>,
    time: Res<Time>,
) {
    let mut player = player.single_mut();
    state.timer.tick(time.delta());
    for PlayerDamageEvent { damage } in ev_player_damage.read() {
        if !settings.nohurt && state.timer.finished() {
            state.timer.reset();
            player.damage += damage;
            if player.is_dead() {
                game_state.game_over = true;
            }
        }
    }
}

#[derive(Debug, Resource)]
struct ShootState {
    // At 0, player fires wildly. At 1.0, player fires perfectly accurately.
    focus: f32,
    jitter_radians: f32,
    reloading: Option<Timer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GunType {
    Pistol,
    Shotgun,
}

impl std::fmt::Display for GunType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GunType::Pistol => "pistol",
            GunType::Shotgun => "shotgun",
        })
    }
}

pub struct GunInfo {
    pub min_jitter_degrees: f32,
    pub max_jitter_degrees: f32,
    pub num_projectiles: usize,
    pub max_load: usize,
    pub loads_one_at_a_time: bool,
    pub muzzle_flash_max_intensity: f32,
    pub muzzle_flash_attenuation: f32,
    pub muzzle_flash_focus: f32,
    reload_time: Duration,
}

impl GunType {
    pub fn get_info(&self) -> GunInfo {
        match self {
            GunType::Pistol => GunInfo {
                min_jitter_degrees: 1.0,
                max_jitter_degrees: 15.0,
                num_projectiles: 1,
                max_load: 15,
                loads_one_at_a_time: false,
                muzzle_flash_max_intensity: 20000.0,
                muzzle_flash_attenuation: 3.0,
                muzzle_flash_focus: 45f32.to_radians(),
                reload_time: Duration::from_secs(2),
            },
            GunType::Shotgun => GunInfo {
                min_jitter_degrees: 5.0,
                max_jitter_degrees: 15.0,
                num_projectiles: 10,
                max_load: 2,
                loads_one_at_a_time: true,
                muzzle_flash_max_intensity: 20000.0,
                muzzle_flash_attenuation: 2.0,
                muzzle_flash_focus: 80f32.to_radians(),
                reload_time: Duration::from_millis(500),
            },
        }
    }
}

#[derive(Default, Clone)]
pub struct GunState {
    pub present: bool,
    pub ammo_loaded: usize,
    pub ammo_available: usize,
}

#[derive(Resource)]
pub struct Inventory {
    pub equipped: GunType,
    pub guns: HashMap<GunType, GunState>,
}

fn swap_gun(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut ev_scroll: EventReader<MouseWheel>,
    assets: Res<GameAssets>,
) {
    for event in ev_scroll.read() {
        let gun_types = inventory
            .guns
            .iter()
            .filter(|(_t, s)| s.present)
            .map(|(t, _s)| t)
            .copied()
            .collect::<Vec<GunType>>();
        let mut i = gun_types
            .iter()
            .enumerate()
            .find(|(_i, t)| **t == inventory.equipped)
            .map(|(i, _t)| i)
            .unwrap();
        if event.y > 0.0 {
            i = (i + 1) % gun_types.len();
        } else {
            i = i.wrapping_sub(1).min(gun_types.len() - 1);
        }

        if gun_types[i] != inventory.equipped {
            let sound = match gun_types[0] {
                GunType::Pistol => &assets.sfx.reload_pistol,
                _ => &assets.sfx.reload_shotgun,
            };
            if let Some(sound) = sound.choose(&mut rand::thread_rng()) {
                commands.spawn((
                    AudioPlayer(sound.clone()),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::new(1.0),
                        ..default()
                    },
                ));
            }
        }

        inventory.equipped = gun_types[i];
    }
}

#[derive(Resource)]
pub struct MouseWorldCoords(pub Vec2);

fn update_mouse_coords(
    mut mouse_world_coords: ResMut<MouseWorldCoords>,
    query_camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
    mut mouse_reader: EventReader<PlaneMouseMovedEvent>,
) {
    for ev in mouse_reader.read() {
        let (camera, camera_transform) = query_camera.single();
        if let Ok(world_pos) =
            camera.viewport_to_world_2d(camera_transform, (ev.0 + 0.5) * SDF_RES as f32)
        {
            mouse_world_coords.0 = world_pos;
        }
    }
}

#[derive(Resource)]
pub struct FlashlightInfo {
    pub battery: f32,
    pub cone_width_degrees: f32,
    // 0 to 1
    pub ease: EasingCurve<f32>,
    pub ease_timer: Timer,
    pub focused: bool,
    // 0 to 1
    pub focus_factor: f32,
}

impl Default for FlashlightInfo {
    fn default() -> Self {
        Self {
            battery: 1.0,
            cone_width_degrees: 0.0,
            ease: EasingCurve::new(0.0, 0.0, EaseFunction::Linear),
            ease_timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
            focused: false,
            focus_factor: 0.0,
        }
    }
}

fn update_flashlight(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut flashlight_info: ResMut<FlashlightInfo>,
    time: Res<Time>,
) {
    flashlight_info.ease_timer.tick(time.delta());
    flashlight_info.battery = if flashlight_info.focused {
        flashlight_info.battery - time.delta_secs() / 8.0
    } else {
        flashlight_info.battery + time.delta_secs() / 16.0
    };
    flashlight_info.battery = flashlight_info.battery.clamp(0.0, FLASHLIGHT_MAX_BATTERY);
    if flashlight_info.battery <= 0.0 {
        flashlight_info.focused = false;
    }
    const FLASHLIGHT_EASE_DURATION: Duration = Duration::from_millis(300);
    let mouse_pressed = mouse_button.pressed(MouseButton::Right);
    if flashlight_info.focused != mouse_pressed
        && !(flashlight_info.battery <= 0.1 && mouse_pressed)
    {
        flashlight_info.focused = mouse_pressed;
        let target: f32 = if flashlight_info.focused { 1.0 } else { 0.0 };
        flashlight_info.ease =
            EasingCurve::new(flashlight_info.focus_factor, target, EaseFunction::Linear);
        flashlight_info.ease_timer = Timer::new(FLASHLIGHT_EASE_DURATION, TimerMode::Once);
    }
    flashlight_info.focus_factor = flashlight_info
        .ease
        .sample_clamped(flashlight_info.ease_timer.fraction());
    flashlight_info.cone_width_degrees = FLASHLIGHT_CONE_WIDTH_UNFOCUSED_DEGREES.lerp(
        FLASHLIGHT_CONE_WIDTH_FOCUSED_DEGREES,
        flashlight_info.focus_factor,
    );
}

#[derive(Component)]
struct LeftSightLine;

#[derive(Component)]
struct RightSightLine;

fn make_sight_lines(mut commands: Commands, assets: Res<GameAssets>) {
    let bundle = (
        Mesh2d(assets.pixel.clone()),
        MeshMaterial2d(assets.sight_line.clone()),
        RenderLayers::layer(UI_LAYER),
        Transform::IDENTITY,
    );
    commands.spawn(bundle.clone()).insert(LeftSightLine);
    commands.spawn(bundle).insert(RightSightLine);
}

#[allow(clippy::type_complexity)]
fn update_sight_lines(
    mut set: ParamSet<(
        Query<&mut Transform, With<LeftSightLine>>,
        Query<&mut Transform, With<RightSightLine>>,
        Query<&Transform, With<Player>>,
    )>,
    shoot_state: Res<ShootState>,
    mouse_world_coords: Res<MouseWorldCoords>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<GameAssets>,
) {
    let player_translation = set.p2().single().translation;
    let mouse_offset = mouse_world_coords.0 - player_translation.truncate();
    let left_dir = Vec2::from_angle(shoot_state.jitter_radians).rotate(mouse_offset.normalize());
    let right_dir = Vec2::from_angle(-shoot_state.jitter_radians).rotate(mouse_offset.normalize());
    let line_start_distance = 60.0;
    let line_length = 180.0;

    let set_sight_line_transform = |transform: &mut Transform, dir: Vec2| {
        transform.translation = (player_translation.truncate()
            + dir * (line_start_distance + line_length / 2.0))
            .extend(0.0);
        transform.rotation = Quat::from_rotation_z((dir).to_angle());
        transform.scale = Vec3::new(line_length, 2.0, 1.0);
    };
    let alpha = ((shoot_state.focus - 0.2) / 2.0).clamp(0.0, 0.4);
    materials.get_mut(assets.sight_line.id()).unwrap().color =
        Color::Srgba(bevy::color::palettes::basic::WHITE.with_alpha(alpha));
    set_sight_line_transform(&mut set.p0().single_mut(), left_dir);
    set_sight_line_transform(&mut set.p1().single_mut(), right_dir);
}

#[derive(Component)]
struct ReloadIndicator;

fn make_reload_indicator(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player: Query<Entity, With<Player>>,
) {
    commands
        .spawn((
            ReloadIndicator,
            Mesh2d(assets.reload_indicator_mesh.clone()),
            MeshMaterial2d(assets.reload_indicator_material.clone()),
            RenderLayers::layer(UI_LAYER),
            Transform::IDENTITY.with_translation(Vec3::ZERO.with_z(3.0)),
        ))
        .set_parent(player.single());
}

fn update_reload_indicator(
    mut reload_indicator: Query<(&mut Transform, &Mesh2d), With<ReloadIndicator>>,
    shoot_state: Res<ShootState>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (mut transform, Mesh2d(mesh_handle)) = reload_indicator.single_mut();
    let fraction_left = shoot_state
        .reloading
        .as_ref()
        .map(|timer| timer.fraction_remaining())
        .unwrap_or(0.0);
    if let Some(mesh) = meshes.get_mut(mesh_handle.id()) {
        *mesh = CircularSector::from_turns(TILE_WIDTH.min(TILE_HEIGHT), fraction_left).into();
    }
    transform.rotation = Quat::from_rotation_z(fraction_left * PI);
}

#[allow(clippy::complexity)]
fn update_shooting(
    mut commands: Commands,
    player_query: Query<(&Transform, &Player)>,
    mut shoot_state: ResMut<ShootState>,
    mouse_world_coords: Res<MouseWorldCoords>,
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    map: Res<Map>,
    mobs: Query<(Entity, &Transform), (With<Mob>, Without<Player>)>,
    tiles: Query<(&Tile, &Transform), (Without<Mob>, Without<Player>)>,
    settings: Res<UiSettings>,
    assets: Res<GameAssets>,
    mut ev_spawn_bullet: EventWriter<ShootEvent>,
    mut ev_damage_mob: EventWriter<MobDamageEvent>,
    mut ev_player_move: EventReader<PlayerMoveEvent>,
    mut inventory: ResMut<Inventory>,
) {
    let (player_pos, player) = player_query.single();
    if player.is_dead() {
        return;
    }

    let player_moved = ev_player_move.read().count() > 0;

    if shoot_state.reloading.is_some() {
        shoot_state.focus = 0.0;
    } else if player_moved {
        shoot_state.focus -= PLAYER_MOVE_FOCUS_PENALTY_SECS / PLAYER_FOCUS_TIME_SECS;
    } else {
        shoot_state.focus += time.delta().as_secs_f32() / PLAYER_FOCUS_TIME_SECS;
    }
    shoot_state.focus = shoot_state.focus.clamp(0.0, 1.0);

    let mouse_offset = mouse_world_coords.0 - player_pos.translation.truncate();
    let equipped = inventory.equipped;
    let equipped_info = inventory.equipped.get_info();
    let gun_state = inventory.guns.entry(equipped).or_default();
    let jitter_degrees = equipped_info.min_jitter_degrees.lerp(
        equipped_info.max_jitter_degrees,
        (1.0 - shoot_state.focus).max(0.0),
    );
    shoot_state.jitter_radians = jitter_degrees * PI / 180.0;

    if keyboard_input.pressed(KeyCode::KeyR)
        && shoot_state.reloading.is_none()
        && gun_state.ammo_available > 0
        && gun_state.ammo_loaded < equipped_info.max_load
    {
        shoot_state.reloading = Some(Timer::new(equipped_info.reload_time, TimerMode::Once));

        // TODO(kazasrinivas3): Clean this up at some point...
        if let Some(sound) = match equipped {
            GunType::Pistol => &assets.sfx.reload_pistol,
            _ => &assets.sfx.reload_shotgun,
        }
        .choose(&mut rand::thread_rng())
        {
            commands.spawn((
                AudioPlayer(sound.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(1.0),
                    ..default()
                },
            ));
        }
    }

    if let Some(ref mut reload_timer) = shoot_state.reloading {
        reload_timer.tick(time.delta());
        if reload_timer.finished() {
            let new_ammo = if equipped_info.loads_one_at_a_time {
                1.min(gun_state.ammo_available)
            } else {
                (equipped_info.max_load - gun_state.ammo_loaded).min(gun_state.ammo_available)
            };
            gun_state.ammo_loaded += new_ammo;
            gun_state.ammo_available -= new_ammo;
            shoot_state.reloading = None;
        }
    }

    if shoot_state.reloading.is_none() && mouse_button.just_pressed(MouseButton::Left) {
        let sound = if gun_state.ammo_loaded > 0 {
            match equipped {
                GunType::Pistol => &assets.sfx.fire_pistol,
                _ => &assets.sfx.fire_shotgun,
            }
        } else {
            match equipped {
                GunType::Pistol => &assets.sfx.empty_pistol,
                _ => &assets.sfx.empty_shotgun,
            }
        };
        if let Some(sound) = sound.choose(&mut rand::thread_rng()) {
            commands.spawn((
                AudioPlayer(sound.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(1.0),
                    ..default()
                },
            ));
        }
        // shoot
        if gun_state.ammo_loaded == 0 && !settings.inf_ammo {
            return;
        }
        if !settings.inf_ammo {
            gun_state.ammo_loaded -= 1;
        }

        // TODO(kazasrinivas3): Clean this up at some point...
        if let Some(sound) = match equipped {
            GunType::Pistol => &assets.sfx.fire_pistol,
            _ => &assets.sfx.fire_shotgun,
        }
        .choose(&mut rand::thread_rng())
        {
            commands.spawn((
                AudioPlayer(sound.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(1.0),
                    ..default()
                },
            ));
        }

        for _ in 0..equipped_info.num_projectiles {
            let line_start = player_pos.translation.truncate();
            shoot_state.focus -= PLAYER_SHOOT_FOCUS_PENALTY_SECS / PLAYER_FOCUS_TIME_SECS;
            let mut collisions = vec![];
            let player_pos_ivec2 = MapPos::from_vec3(player_pos.translation).0;
            for (entity, transform) in mobs.iter_many(map.get_nearby(player_pos_ivec2, 100)) {
                collisions.push((
                    Some(entity),
                    Aabb2d::new(
                        transform.translation.truncate(),
                        Vec2::new(TILE_WIDTH, TILE_HEIGHT) / 2.0,
                    ),
                ));
            }
            for (tile, transform) in tiles.iter_many(map.get_nearby(player_pos_ivec2, 100)) {
                if tile.0.blocks_movement() {
                    collisions.push((
                        None,
                        Aabb2d::new(
                            transform.translation.truncate(),
                            Vec2::new(TILE_WIDTH, TILE_HEIGHT) / 2.0,
                        ),
                    ));
                }
            }
            let dist_to_player = |point| player_pos.translation.truncate().distance_squared(point);
            collisions.sort_by(|a, b| {
                dist_to_player((a.1.min + a.1.max) / 2.0)
                    .partial_cmp(&dist_to_player((b.1.min + b.1.max) / 2.0))
                    .unwrap()
            });

            let angle_radians =
                (rand::thread_rng().r#gen::<f32>() - 0.5) * (shoot_state.jitter_radians * 2.0);
            let dir = Vec2::from_angle(angle_radians).rotate(mouse_offset);
            if let Ok(dir) = Dir2::new(dir) {
                let ray = RayCast2d::new(line_start, dir, 4000.0);
                let mut line = None;
                let mut hit_mob = None;
                for (mob, aabb) in collisions {
                    if let Some(distance) = ray.aabb_intersection_at(&aabb) {
                        line = Some((line_start, line_start + dir * distance));
                        hit_mob = mob;
                        break;
                    }
                }
                // did not collide with any wall or mob.
                let line = line.unwrap_or((line_start, line_start + dir * 4000.0));
                let (start, end) = line;
                ev_spawn_bullet.send(ShootEvent { start, end });
                if let Some(hit_mob) = hit_mob {
                    ev_damage_mob.send(MobDamageEvent {
                        damage: 1,
                        entity: hit_mob,
                    });
                }
            }
        }
    }
}

/// Player fired their gun and it hit something.
#[derive(Event)]
pub struct ShootEvent {
    pub start: Vec2,
    pub end: Vec2,
}

fn spawn_bullets(
    mut commands: Commands,
    mut ev_shoot: EventReader<ShootEvent>,
    assets: Res<GameAssets>,
) {
    for ShootEvent { start, end } in ev_shoot.read() {
        // create a rectangle stretching between start and end.
        commands.spawn((
            DespawnAfter(Timer::new(Duration::from_millis(100), TimerMode::Once)),
            Mesh2d(assets.pixel.clone()),
            MeshMaterial2d(assets.white.clone()),
            RenderLayers::layer(1),
            Transform {
                translation: ((start + end) / 2.0).extend(0.0),
                rotation: Quat::from_rotation_z((end - start).to_angle()),
                scale: Vec3::new((start - end).length(), 1.0, 1.0),
            },
        ));
    }
}

#[derive(Resource)]
struct MoveTimer(Timer);

#[derive(Default)]
struct MovePlayerState {
    last_move_direction: IVec2,
}

#[derive(Event)]
pub struct PlayerMoveEvent {
    pub source: MapPos,
    pub dest: MapPos,
}

#[allow(clippy::complexity)]
fn move_player(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &mut MapPos, &Player)>,
    blocked_query: Query<&mut MapPos, (With<BlocksMovement>, Without<Player>)>,
    mut timer: ResMut<MoveTimer>,
    mut local_state: Local<MovePlayerState>,
    tile_map: Res<Map>,
    time: Res<Time>,
    mut ev_player_move: EventWriter<PlayerMoveEvent>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        if let Ok((entity, mut world_pos, player)) = player_query.get_single_mut() {
            if player.is_dead() {
                return;
            }
            let mut movement = IVec2::ZERO;
            for (key, dir) in [
                (KeyCode::KeyW, IVec2::new(0, 1)),
                (KeyCode::KeyA, IVec2::new(-1, 0)),
                (KeyCode::KeyS, IVec2::new(0, -1)),
                (KeyCode::KeyD, IVec2::new(1, 0)),
            ] {
                if keyboard_input.pressed(key) {
                    movement += dir;
                }
            }
            let x_move = IVec2::new(movement.x, 0);
            let y_move = IVec2::new(0, movement.y);
            let x_dest = world_pos.0 + x_move;
            let y_dest = world_pos.0 + y_move;
            let x_valid = movement.x != 0
                && blocked_query
                    .iter_many(tile_map.0.get(&x_dest).unwrap_or(&vec![]))
                    .next()
                    .is_none();
            let y_valid = movement.y != 0
                && blocked_query
                    .iter_many(tile_map.0.get(&y_dest).unwrap_or(&vec![]))
                    .next()
                    .is_none();
            if !x_valid {
                movement.x = 0;
            }
            if !y_valid {
                movement.y = 0;
            }
            if x_valid && y_valid {
                // alternate
                if local_state.last_move_direction.x == x_move.x {
                    movement.x = 0;
                } else {
                    movement.y = 0;
                }
            }
            world_pos.0 += movement;
            if movement != IVec2::ZERO {
                let from = MapPos(world_pos.0 - movement);
                commands.entity(entity).insert(MoveAnimation {
                    from: from.to_vec2(),
                    to: world_pos.to_vec2(),
                    timer: Timer::new(Duration::from_millis(100), TimerMode::Once),
                    ease: EaseFunction::QuadraticOut,
                });
                local_state.last_move_direction = movement;
                timer.0.reset();
                ev_player_move.send(PlayerMoveEvent {
                    source: from,
                    dest: world_pos.clone(),
                });
            }
        }
    }
}

fn pickup(
    mut commands: Commands,
    mut ev_player_move: EventReader<PlayerMoveEvent>,
    mut ev_text: EventWriter<TextEvent>,
    tile_map: Res<Map>,
    q_pickups: Query<(Entity, &Pickup)>,
    mut inventory: ResMut<Inventory>,
) {
    for PlayerMoveEvent { dest, .. } in ev_player_move.read() {
        for (entity, Pickup(kind)) in
            q_pickups.iter_many(tile_map.0.get(&dest.0).unwrap_or(&vec![]))
        {
            commands.entity(entity).despawn();
            match kind {
                crate::map::ItemKind::Ammo(gun_type, num_ammo) => {
                    inventory.guns.entry(*gun_type).or_default().ammo_available += num_ammo;
                }
                crate::map::ItemKind::Gun(gun_type, ammo) => {
                    let gun_state = inventory.guns.entry(*gun_type).or_default();
                    gun_state.present = true;
                    gun_state.ammo_loaded += *ammo;
                    let gun_info = gun_type.get_info();
                    if gun_state.ammo_loaded > gun_info.max_load {
                        gun_state.ammo_available += gun_state.ammo_loaded - gun_info.max_load;
                        gun_state.ammo_loaded = gun_info.max_load;
                    }
                }
            }
            ev_text.send(TextEvent {
                text: format!("got {kind}!"),
                position: dest.to_vec2(),
                duration: Duration::from_secs(5),
                ..default()
            });
        }
    }
}

fn startup(mut commands: Commands, assets: Res<GameAssets>) {
    let player_start_translation =
        Vec3::new(PLAYER_START.x as f32, PLAYER_START.y as f32, Z_PLAYER);
    commands.spawn((
        Player { damage: 0 },
        MapPos(PLAYER_START),
        assets.get_sprite(SpriteKind::Player),
        Transform::from_translation(player_start_translation),
        RenderLayers::layer(1),
        WobbleEffects::default(),
    ));
    commands.insert_resource(MoveTimer(Timer::new(PLAYER_MOVE_DELAY, TimerMode::Once)));
    commands.insert_resource(MouseWorldCoords(player_start_translation.truncate()));
    commands.insert_resource(ShootState {
        focus: 0.0,
        jitter_radians: 0.0,
        reloading: None,
    });
    let mut guns = HashMap::new();
    guns.insert(
        GunType::Pistol,
        GunState {
            present: true,
            ammo_loaded: GunType::Pistol.get_info().max_load,
            ammo_available: 15,
        },
    );
    commands.insert_resource(Inventory {
        equipped: GunType::Pistol,
        guns,
    });
    commands.insert_resource(PlayerDamageState {
        timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
    });
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (startup, make_sight_lines, make_reload_indicator).chain(),
        )
        .add_systems(
            Update,
            (
                move_player,
                pickup,
                swap_gun,
                update_mouse_coords,
                update_flashlight,
                update_shooting,
                update_sight_lines,
                spawn_bullets,
                update_reload_indicator,
                damage_player,
            )
                .chain(),
        )
        .add_event::<ShootEvent>()
        .add_event::<PlayerMoveEvent>()
        .add_event::<PlayerDamageEvent>()
        .init_resource::<FlashlightInfo>();
    }
}
