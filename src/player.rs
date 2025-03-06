use std::{f32::consts::PI, time::Duration};

use bevy::{
    math::bounding::{Aabb2d, RayCast2d},
    prelude::*,
    render::view::RenderLayers,
};
use rand::Rng as _;

use crate::{
    PrimaryCamera,
    animation::MoveAnimation,
    assets::GameAssets,
    map::{BlocksMovement, Map, MapPos, TILE_SIZE, Tile},
    mob::{Mob, MobDamageEvent},
};

const PLAYER_MOVE_DELAY: Duration = Duration::from_millis(200);
const PLAYER_SHOOT_DELAY: Duration = Duration::from_millis(500);
const PLAYER_START: IVec2 = IVec2::new(0, 0);
const PLAYER_FOCUS_TIME_SECS: f32 = 2.5;
const PLAYER_MOVE_FOCUS_PENALTY_SECS: f32 = 1.0;
const PLAYER_SHOOT_FOCUS_PENALTY_SECS: f32 = 1.0;
const PLAYER_AIM_JITTER_MAX_DEGREES: f32 = 15.0;

#[derive(Component)]
pub struct Player;

#[derive(Debug, Resource)]
struct ShootState {
    timer: Timer,
    // At 0, player fires wildly. At 1.0, player fires perfectly accurately.
    focus: f32,
    jitter_radians: f32,
}

#[derive(Resource)]
struct MouseWorldCoords(Vec2);

fn update_mouse_coords(
    mut mouse_world_coords: ResMut<MouseWorldCoords>,
    query_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
) {
    let (camera, camera_transform) = query_camera.single();
    if let Ok(window) = query_window.get_single() {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        {
            mouse_world_coords.0 = world_position;
        }
    }
}

#[derive(Component)]
struct DespawnAfter(Timer);

fn despawn_after(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnAfter)>,
    time: Res<Time>,
) {
    for (entity, mut clear_after) in query.iter_mut() {
        clear_after.0.tick(time.delta());
        if clear_after.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct LeftSightLine;

#[derive(Component)]
struct RightSightLine;

fn make_sight_lines(mut commands: Commands, assets: Res<GameAssets>) {
    let bundle = (
        Mesh2d(assets.pixel.clone()),
        MeshMaterial2d(assets.sight_line.clone()),
        RenderLayers::layer(1),
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
        transform.scale = Vec3::new(line_length, 1.0, 1.0);
    };
    let alpha = ((shoot_state.focus - 0.2) / 2.0).clamp(0.0, 0.4);
    materials.get_mut(assets.sight_line.id()).unwrap().color =
        Color::Srgba(bevy::color::palettes::basic::YELLOW.with_alpha(alpha));
    set_sight_line_transform(&mut set.p0().single_mut(), left_dir);
    set_sight_line_transform(&mut set.p1().single_mut(), right_dir);
}

#[allow(clippy::complexity)]
fn update_shooting(
    player_query: Query<&Transform, With<Player>>,
    mut shoot_state: ResMut<ShootState>,
    mouse_world_coords: Res<MouseWorldCoords>,
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    map: Res<Map>,
    mobs: Query<(Entity, &Transform), (With<Mob>, Without<Player>)>,
    tiles: Query<(&Tile, &Transform), (Without<Mob>, Without<Player>)>,
    mut ev_spawn_bullet: EventWriter<ShootEvent>,
    mut ev_damage_mob: EventWriter<MobDamageEvent>,
    mut ev_player_move: EventReader<PlayerMoveEvent>,
) {
    let player_pos = player_query.single();

    shoot_state.timer.tick(time.delta());
    if ev_player_move.read().count() > 0 {
        shoot_state.focus -= PLAYER_MOVE_FOCUS_PENALTY_SECS / PLAYER_FOCUS_TIME_SECS;
    } else {
        shoot_state.focus += time.delta().as_secs_f32() / PLAYER_FOCUS_TIME_SECS;
    }
    shoot_state.focus = shoot_state.focus.clamp(0.0, 1.0);

    let mouse_offset = mouse_world_coords.0 - player_pos.translation.truncate();
    let jitter_degrees = PLAYER_AIM_JITTER_MAX_DEGREES * (1.0 - shoot_state.focus).max(0.0);
    shoot_state.jitter_radians = jitter_degrees * PI / 180.0;

    if mouse_button.just_pressed(MouseButton::Left) && shoot_state.timer.finished() {
        // shoot
        let line_start = player_pos.translation.truncate();
        shoot_state.focus -= PLAYER_SHOOT_FOCUS_PENALTY_SECS / PLAYER_FOCUS_TIME_SECS;
        let mut collisions = vec![];
        let player_pos_ivec2 = MapPos::from_vec3(player_pos.translation).0;
        for (entity, transform) in mobs.iter_many(map.get_nearby(player_pos_ivec2, 100)) {
            collisions.push((
                Some(entity),
                Aabb2d::new(
                    transform.translation.truncate(),
                    Vec2::splat(TILE_SIZE / 2.0),
                ),
            ));
        }
        for (tile, transform) in tiles.iter_many(map.get_nearby(player_pos_ivec2, 100)) {
            if tile.0.blocks_movement() {
                collisions.push((
                    None,
                    Aabb2d::new(
                        transform.translation.truncate(),
                        Vec2::splat(TILE_SIZE / 2.0),
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

/// Player fired their gun and it hit something.
#[derive(Event)]
struct ShootEvent {
    start: Vec2,
    end: Vec2,
}

fn spawn_bullets(
    mut commands: Commands,
    mut ev_spawn_bullet: EventReader<ShootEvent>,
    assets: Res<GameAssets>,
) {
    for ShootEvent { start, end } in ev_spawn_bullet.read() {
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
struct PlayerMoveEvent;

#[allow(clippy::complexity)]
fn move_player(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &mut MapPos), With<Player>>,
    blocked_query: Query<&mut MapPos, (With<BlocksMovement>, Without<Player>)>,
    mut timer: ResMut<MoveTimer>,
    mut local_state: Local<MovePlayerState>,
    tile_map: Res<Map>,
    time: Res<Time>,
    mut ev_player_move: EventWriter<PlayerMoveEvent>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        if let Ok((entity, mut world_pos)) = player_query.get_single_mut() {
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
                commands.entity(entity).insert(MoveAnimation {
                    from: MapPos(world_pos.0 - movement).to_vec2(),
                    to: world_pos.to_vec2(),
                    timer: Timer::new(Duration::from_millis(100), TimerMode::Once),
                    ease: EaseFunction::QuadraticOut,
                });
                local_state.last_move_direction = movement;
                timer.0.reset();
                ev_player_move.send(PlayerMoveEvent);
            }
        }
    }
}

fn startup(mut commands: Commands, assets: Res<GameAssets>) {
    let player_start_translation = Vec3::new(PLAYER_START.x as f32, PLAYER_START.y as f32, 1.0);
    commands.spawn((
        Player,
        MapPos(PLAYER_START),
        Mesh2d(assets.circle.clone()),
        MeshMaterial2d(assets.red.clone()),
        Transform::from_translation(player_start_translation),
        RenderLayers::layer(1),
    ));
    commands.insert_resource(MoveTimer(Timer::new(PLAYER_MOVE_DELAY, TimerMode::Once)));
    commands.insert_resource(MouseWorldCoords(player_start_translation.truncate()));
    commands.insert_resource(ShootState {
        timer: Timer::new(PLAYER_SHOOT_DELAY, TimerMode::Once),
        focus: 0.0,
        jitter_radians: 0.0,
    });
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (startup, make_sight_lines))
            .add_systems(
                Update,
                (
                    move_player,
                    update_mouse_coords,
                    update_shooting,
                    update_sight_lines,
                    spawn_bullets,
                    despawn_after,
                )
                    .chain(),
            )
            .add_event::<ShootEvent>()
            .add_event::<PlayerMoveEvent>();
    }
}
