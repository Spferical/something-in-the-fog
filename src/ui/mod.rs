use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, Color32, FontData, FontFamily, TextStyle},
};

pub mod performance;

#[derive(Default)]
pub(crate) struct UiPlugin;

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

fn update(mut contexts: EguiContexts) {
    egui::TopBottomPanel::bottom("bottom_panel").show(contexts.ctx_mut(), |ui| {
        ui.centered_and_justified(|ui| ui.label("move: WASD  shoot: click"))
    });
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, startup)
            .add_systems(Update, update);
    }
}
