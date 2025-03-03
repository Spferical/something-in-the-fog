use std::time::Duration;

use bevy::prelude::*;

mod performance_ui;

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

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    window.single_mut().resizable = true;
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Mesh2d(meshes.add(Rectangle::new(24.0, 24.0))),
        MeshMaterial2d(materials.add(Color::LinearRgba(LinearRgba::RED))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
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
        .add_plugins(performance_ui::PerformanceUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, on_resize)
        .add_systems(FixedUpdate, move_player)
        .run();
}
