use std::time::Duration;

use bevy::prelude::*;

mod map;
mod mapgen;
mod performance_ui;
mod renderer;
mod sdf;

const CAMERA_DECAY_RATE: f32 = 2.;
const PLAYER_MOVE_DELAY: Duration = Duration::from_millis(100);
const PLAYER_SHOOT_DELAY: Duration = Duration::from_millis(500);
const PLAYER_START: IVec2 = IVec2::new(100, 0);

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}

#[derive(Component)]
struct Player;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::linear_rgba(0.0, 0.0, 0.0, 0.0)),
            hdr: true,
            ..default()
        },
    ));
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>, assets: Res<map::WorldAssets>) {
    window.single_mut().resizable = true;
    let player_start_translation = Vec3::new(PLAYER_START.x as f32, PLAYER_START.y as f32, 1.0);
    commands.spawn((
        Player,
        map::MapPos(PLAYER_START),
        Mesh2d(assets.circle.clone()),
        MeshMaterial2d(assets.red.clone()),
        Transform::from_translation(player_start_translation),
    ));
    commands.insert_resource(MoveTimer(Timer::new(PLAYER_MOVE_DELAY, TimerMode::Once)));
    commands.insert_resource(MouseWorldCoords(player_start_translation.truncate()));
    commands.insert_resource(ShootState {
        timer: Timer::new(PLAYER_SHOOT_DELAY, TimerMode::Once),
    });
}

#[derive(Resource)]
struct MouseWorldCoords(Vec2);

fn update_mouse_coords(
    mut mouse_world_coords: ResMut<MouseWorldCoords>,
    query_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let (camera, camera_transform) = query_camera.single();
    let window = query_window.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        mouse_world_coords.0 = world_position;
    }
}

#[derive(Resource)]
struct ShootState {
    timer: Timer,
}

fn update_shooting(
    player_query: Query<&Transform, With<Player>>,
    shoot_state: ResMut<ShootState>,
    mouse_world_coords: Res<MouseWorldCoords>,
    mut gizmos: Gizmos,
) {
    let player_pos = player_query.single();
    let mouse_offset = mouse_world_coords.0 - player_pos.translation.truncate();
    gizmos.ray_gradient_2d(
        player_pos.translation.truncate(),
        mouse_offset.normalize() * 240.0,
        bevy::color::palettes::basic::YELLOW,
        Color::NONE.into(),
    );
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
        .add_plugins(renderer::Renderer)
        .add_plugins(performance_ui::PerformanceUiPlugin)
        .add_plugins(map::WorldPlugin)
        .add_systems(Startup, (setup_camera, setup))
        .add_systems(
            Update,
            (
                update_camera,
                on_resize,
                update_mouse_coords,
                update_shooting,
            ),
        )
        .add_systems(FixedUpdate, move_player)
        .run();
}
