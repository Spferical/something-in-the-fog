use std::{f32::consts::PI, time::Duration};

use assets::GameAssets;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::math::bounding::{Aabb2d, RayCast2d};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use map::{Map, MapPos, Mob, TILE_SIZE, Tile};
use rand::Rng as _;

mod assets;
mod edge;
mod map;
mod mapgen;
mod performance_ui;
mod renderer;
mod sdf;

const CAMERA_DECAY_RATE: f32 = 2.;
const PLAYER_MOVE_DELAY: Duration = Duration::from_millis(100);
const PLAYER_SHOOT_DELAY: Duration = Duration::from_millis(500);
const PLAYER_START: IVec2 = IVec2::new(0, 0);

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PrimaryCamera;

fn create_camera(
    mut window: Single<&mut Window>,
    mut commands: Commands,
    // camera_query: Query<(Entity, &Camera), With<PrimaryCamera>>,
    mut resize_reader: EventReader<bevy::window::WindowResized>,
    mut images: ResMut<Assets<Image>>,
) {
    window.resizable = true;
    let mut image = Image::new_fill(
        Extent3d {
            width: window.resolution.physical_width(), // does this work?
            height: window.resolution.physical_height(),
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    let camera = Camera {
        clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
        hdr: true,
        order: 0,
        ..default()
    };

    let texture_cpu = renderer::OccluderTextureCpu(image_handle);
    commands.spawn(texture_cpu.clone());

    commands.spawn((
        Camera2d,
        camera,
        RenderLayers::layer(1),
        PrimaryCamera,
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
    ));

    /*match camera_query.get_single() {
        Ok((camera_entity, _)) => {
            commands
                .entity(camera_entity)
                .remove::<Camera>()
                .insert(camera);
        }
        Err(_) => {
            commands.spawn((
                Camera2d,
                camera,
                RenderLayers::layer(1),
                PrimaryCamera,
                Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
            ));
        }
    };*/

    for e in resize_reader.read() {
        println!("Resize happened {:?}", e);
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<PrimaryCamera>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<PrimaryCamera>)>,
    time: Res<Time>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>, assets: Res<GameAssets>) {
    window.single_mut().resizable = true;
    window.single_mut().present_mode = bevy::window::PresentMode::AutoNoVsync;
    let player_start_translation = Vec3::new(PLAYER_START.x as f32, PLAYER_START.y as f32, 1.0);
    commands.spawn((
        Player,
        map::MapPos(PLAYER_START),
        Mesh2d(assets.circle.clone()),
        MeshMaterial2d(assets.red.clone()),
        Transform::from_translation(player_start_translation),
        RenderLayers::layer(1),
    ));
    commands.insert_resource(MoveTimer(Timer::new(PLAYER_MOVE_DELAY, TimerMode::Once)));
    commands.insert_resource(MouseWorldCoords(player_start_translation.truncate()));
    commands.insert_resource(ShootState {
        timer: Timer::new(PLAYER_SHOOT_DELAY, TimerMode::Once),
        player_last_position: player_start_translation.truncate(),
        focus: 0.0,
        jitter_radians: 0.0,
    });
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

#[derive(Debug, Resource)]
struct ShootState {
    timer: Timer,
    player_last_position: Vec2,
    focus: f32,
    jitter_radians: f32,
}

#[derive(Component)]
struct ClearAfter(Timer);

fn clear_after(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ClearAfter)>,
    time: Res<Time>,
) {
    for (entity, mut bullet_trail) in query.iter_mut() {
        bullet_trail.0.tick(time.delta());
        if bullet_trail.0.finished() {
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
    mobs: Query<(&Mob, &Transform), Without<Player>>,
    tiles: Query<(&Tile, &Transform), (Without<Mob>, Without<Player>)>,
    mut ev_spawn_bullet: EventWriter<SpawnBulletEvent>,
) {
    let player_pos = player_query.single();

    shoot_state.timer.tick(time.delta());
    if shoot_state.player_last_position != player_pos.translation.truncate() {
        shoot_state.player_last_position = player_pos.translation.truncate();
        shoot_state.focus -= 1.0;
    } else {
        shoot_state.focus += time.delta().as_secs_f32();
    }
    shoot_state.focus = shoot_state.focus.clamp(0.0, 2.5);

    let mouse_offset = mouse_world_coords.0 - player_pos.translation.truncate();
    let aim_angle_degrees = (15.0 - shoot_state.focus * 6.0).max(0.0);
    shoot_state.jitter_radians = aim_angle_degrees * PI / 180.0;

    if mouse_button.just_pressed(MouseButton::Left) {
        // shoot
        let line_start = player_pos.translation.truncate();
        shoot_state.focus -= 1.0;
        let mut collisions = vec![];
        let player_pos_ivec2 = MapPos::from_vec3(player_pos.translation).0;
        for (_mob, transform) in mobs.iter_many(map.get_nearby(player_pos_ivec2, 100)) {
            collisions.push(Aabb2d::new(
                transform.translation.truncate(),
                Vec2::splat(TILE_SIZE / 2.0),
            ));
        }
        for (tile, transform) in tiles.iter_many(map.get_nearby(player_pos_ivec2, 100)) {
            if tile.0.blocks_movement() {
                collisions.push(Aabb2d::new(
                    transform.translation.truncate(),
                    Vec2::splat(TILE_SIZE / 2.0),
                ));
            }
        }
        let dist_to_player = |point| player_pos.translation.truncate().distance_squared(point);
        collisions.sort_by(|a, b| {
            dist_to_player((a.min + a.max) / 2.0)
                .partial_cmp(&dist_to_player((b.min + b.max) / 2.0))
                .unwrap()
        });

        let angle_radians =
            (rand::thread_rng().r#gen::<f32>() - 0.5) * (shoot_state.jitter_radians * 2.0);
        let dir = Vec2::from_angle(angle_radians).rotate(mouse_offset);
        if let Ok(dir) = Dir2::new(dir) {
            let ray = RayCast2d::new(line_start, dir, 4000.0);
            let mut line = None;
            for c in collisions {
                if let Some(distance) = ray.aabb_intersection_at(&c) {
                    line = Some((line_start, line_start + dir * distance));
                    break;
                }
            }
            // did not collide with any wall or mob.
            let line = line.unwrap_or((line_start, line_start + dir * 4000.0));
            let (start, end) = line;
            ev_spawn_bullet.send(SpawnBulletEvent { start, end });
        }
    }
}

#[derive(Event)]
struct SpawnBulletEvent {
    start: Vec2,
    end: Vec2,
}

fn spawn_bullets(
    mut commands: Commands,
    mut ev_spawn_bullet: EventReader<SpawnBulletEvent>,
    assets: Res<GameAssets>,
) {
    for SpawnBulletEvent { start, end } in ev_spawn_bullet.read() {
        // create a rectangle stretching between start and end.
        commands.spawn((
            ClearAfter(Timer::new(Duration::from_millis(100), TimerMode::Once)),
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

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut map::MapPos), With<Player>>,
    blocked_query: Query<&mut map::MapPos, (With<map::BlocksMovement>, Without<Player>)>,
    mut timer: ResMut<MoveTimer>,
    mut local_state: Local<MovePlayerState>,
    tile_map: Res<map::Map>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        if let Ok((mut transform, mut world_pos)) = player_query.get_single_mut() {
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
            transform.translation = world_pos.to_vec3(transform.translation.z);
            if movement != IVec2::ZERO {
                local_state.last_move_direction = movement;
                timer.0.reset();
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(performance_ui::PerformanceUiPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(map::WorldPlugin)
        .add_plugins(renderer::Renderer)
        .add_systems(Startup, (create_camera, setup, make_sight_lines))
        .add_systems(
            Update,
            (
                update_camera,
                on_resize,
                update_mouse_coords,
                update_shooting,
                update_sight_lines,
                spawn_bullets,
                clear_after,
            )
                .chain(),
        )
        .add_systems(FixedUpdate, move_player)
        .add_event::<SpawnBulletEvent>()
        .run();
}
