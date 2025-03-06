use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    EguiContexts,
    egui::{self, Color32},
};

use super::UiSettings;

fn draw(mut contexts: EguiContexts, settings: Res<UiSettings>, diagnostics: Res<DiagnosticsStore>) {
    if settings.show_performance_overlay {
        egui::SidePanel::right("performance_ui_panel")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_black_alpha(240))
                    .inner_margin(12.0),
            )
            .show(contexts.ctx_mut(), |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
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
                    ui.label(format!("Frame Time: {value:>7.3}ms"));
                }
                if let Some(value) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                    .map(|time| time.values().fold(f64::NEG_INFINITY, |a, &b| a.max(b)))
                {
                    ui.label(format!("Worst Frame: {value:>7.3}ms"));
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
            .add_systems(Update, draw);
    }
}
