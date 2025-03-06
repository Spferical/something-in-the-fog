use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, FontData, FontFamily, TextStyle},
};

pub mod performance;

#[derive(Default)]
pub(crate) struct UiPlugin;

fn startup(mut contexts: EguiContexts) {
    let mut fonts = egui::FontDefinitions::default();
    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert(
        "pressstart2p".to_owned(),
        std::sync::Arc::new(
            // .ttf and .otf supported
            FontData::from_static(include_bytes!(
                "../../assets/PressStart2P/PressStart2P-Regular.ttf"
            )),
        ),
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
    })
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin).add_systems(Startup, startup);
    }
}
