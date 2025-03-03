use bevy::prelude::*;
use iyes_perf_ui::prelude::*;

#[derive(Default)]
struct PerformanceUiState {
    show_performance_overlay: bool,
}

fn toggle_performance_display(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: Local<PerformanceUiState>,
    mut perf_ui_query: Query<&mut Visibility, With<PerfUiRoot>>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        settings.show_performance_overlay = !settings.show_performance_overlay;
    }
    for mut perf_ui in perf_ui_query.iter_mut() {
        *perf_ui = if settings.show_performance_overlay {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(PerfUiDefaultEntries::default());
}

#[derive(Default)]
pub(crate) struct PerformanceUiPlugin;

impl Plugin for PerformanceUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .add_plugins(PerfUiPlugin)
            .add_systems(Startup, startup)
            .add_systems(Update, toggle_performance_display);
    }
}
