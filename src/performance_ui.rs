use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPlugin, egui};

#[derive(Default, Resource)]
struct PerformanceUiState {
    show_performance_overlay: bool,
}

fn toggle_performance_display(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<PerformanceUiState>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        settings.show_performance_overlay = !settings.show_performance_overlay;
    }
}

fn draw_performance_overlay(
    mut contexts: EguiContexts,
    settings: Res<PerformanceUiState>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if settings.show_performance_overlay {
        egui::SidePanel::right("performance_ui_panel").show(contexts.ctx_mut(), |ui| {
            if let Some(value) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
            {
                ui.label(format!("FPS: {value:>4.0}"));
            }
            if let Some(value) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(|time| time.smoothed())
            {
                ui.label(format!("Frame Time: {value:.3}ms"));
            }
            if let Some(value) = diagnostics
                .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .and_then(|v| v.value())
            {
                ui.label(format!("Entities: {value:>4}"));
            }
        });
    }
}

#[derive(Default)]
pub(crate) struct PerformanceUiPlugin;

impl Plugin for PerformanceUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .add_plugins(EguiPlugin)
            .init_resource::<PerformanceUiState>()
            .add_systems(
                Update,
                (toggle_performance_display, draw_performance_overlay),
            );
    }
}
