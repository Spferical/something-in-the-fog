use bevy::prelude::*;

mod performance_ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(performance_ui::PerformanceUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, on_resize)
        .run();
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>) {
    window.single_mut().resizable = true;
    commands.spawn(Camera2d);
}

fn on_resize(mut resize_reader: EventReader<bevy::window::WindowResized>) {
    for _e in resize_reader.read() {}
}
