use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, Color32, FontData, FontFamily, TextStyle},
};

pub mod performance;

#[derive(Default)]
pub(crate) struct UiPlugin;

#[derive(Default, Resource)]
pub struct UiSettings {
    pub show_performance_overlay: bool,
    pub show_debug_settings: bool,
}

fn startup(mut contexts: EguiContexts) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "pressstart2p".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../../assets/PressStart2P/PressStart2P-Regular.ttf"
        ))),
    );

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "pressstart2p".to_owned());

    let egui_ctx = contexts.ctx_mut();
    egui_ctx.set_fonts(fonts);

    egui_ctx.style_mut(|style| {
        style.override_text_style = Some(TextStyle::Monospace);
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
    })
}

fn update(keyboard_input: Res<ButtonInput<KeyCode>>, mut settings: ResMut<UiSettings>) {
    settings.show_performance_overlay ^= keyboard_input.just_pressed(KeyCode::F3);
    settings.show_debug_settings ^= keyboard_input.just_pressed(KeyCode::F4);
}

#[derive(Event)]
pub enum UiEvent {
    TeleportPlayer(usize),
}

fn draw(mut contexts: EguiContexts, settings: Res<UiSettings>, mut ev: EventWriter<UiEvent>) {
    egui::TopBottomPanel::bottom("bottom_panel").show(contexts.ctx_mut(), |ui| {
        ui.centered_and_justified(|ui| ui.label("move: WASD  shoot: click"))
    });
    if settings.show_debug_settings {
        egui::TopBottomPanel::bottom("debug_panel").show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Teleport to... ");
                for i in 0..=5 {
                    if ui.button(format!("{i}")).clicked() {
                        ev.send(UiEvent::TeleportPlayer(i));
                    }
                }
            })
        });
    }
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<UiSettings>()
            .add_event::<UiEvent>()
            .add_systems(Startup, startup)
            .add_systems(Update, (update, draw).chain());
    }
}
