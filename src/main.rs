use std::time::Duration;

use bevy::prelude::*;

mod performance_ui;
mod renderer;
mod sdf;
mod world;

const CAMERA_DECAY_RATE: f32 = 2.;

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}

#[derive(Resource)]
struct MoveTimer(Timer);

impl Default for MoveTimer {
    fn default() -> Self {
        MoveTimer(Timer::new(Duration::from_secs_f64(0.25), TimerMode::Once))
    }
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

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    window.single_mut().resizable = true;
    commands.spawn((
        Player,
        Mesh2d(meshes.add(Rectangle::new(24.0, 24.0))),
        MeshMaterial2d(materials.add(Color::LinearRgba(LinearRgba::RED))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));
    commands.init_resource::<MoveTimer>();
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    mut timer: ResMut<MoveTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        if let Ok(mut transform) = query.get_single_mut() {
            let mut movement = Vec3::ZERO;
            for (key, dir) in [
                (KeyCode::KeyW, Vec3::new(0.0, 1.0, 0.0)),
                (KeyCode::KeyA, Vec3::new(-1.0, 0.0, 0.0)),
                (KeyCode::KeyS, Vec3::new(0.0, -1.0, 0.0)),
                (KeyCode::KeyD, Vec3::new(1.0, 0.0, 0.0)),
            ] {
                if keyboard_input.pressed(key) {
                    movement += dir * Vec3::splat(24.0);
                }
            }
            transform.translation += movement;
            if movement != Vec3::ZERO {
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
        .add_plugins(world::WorldPlugin)
        .add_systems(Startup, (setup_camera, setup))
        .add_systems(Update, (update_camera, on_resize))
        .add_systems(FixedUpdate, move_player)
        .run();
}
