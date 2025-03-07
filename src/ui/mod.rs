use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, Align, Color32, FontData, FontFamily, Separator, TextStyle},
};

use crate::{
    assets::PRESS_START_2P_BYTES,
    player::{GunInfo, GunState, Inventory},
};

pub mod performance;

#[derive(Default)]
pub(crate) struct UiPlugin;

#[derive(Resource)]
pub struct UiSettings {
    pub show_performance_overlay: bool,
    pub show_debug_settings: bool,
    pub debug_scroll: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_performance_overlay: false,
            show_debug_settings: true,
            debug_scroll: true,
        }
    }
}

fn startup(mut contexts: EguiContexts) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "pressstart2p".to_owned(),
        std::sync::Arc::new(FontData::from_static(PRESS_START_2P_BYTES)),
    );

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "pressstart2p".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "pressstart2p".to_owned());

    let egui_ctx = contexts.ctx_mut();
    egui_ctx.set_fonts(fonts);

    egui_ctx.style_mut(|style| {
        style.override_text_style = Some(TextStyle::Monospace);
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
    })
}

#[derive(Event)]
pub enum UiEvent {
    TeleportPlayer(usize),
}

fn update(
    mut contexts: EguiContexts,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<UiSettings>,
    mut ev: EventWriter<UiEvent>,
    inventory: Res<Inventory>,
) {
    settings.show_performance_overlay ^= keyboard_input.just_pressed(KeyCode::F3);
    settings.show_debug_settings ^= keyboard_input.just_pressed(KeyCode::F4);
    egui::SidePanel::left("side_panel").show(contexts.ctx_mut(), |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
            ui.label("Equipped");
            ui.add(Separator::default().horizontal());
        });
        ui.label("");
        let equipped = inventory.equipped;
        let GunState { ammo_loaded, .. } =
            inventory.guns.get(&equipped).cloned().unwrap_or_default();
        let GunInfo {
            max_load: max_loaded,
            ..
        } = equipped.get_info();
        ui.label(format!("{equipped:>7} [{ammo_loaded}/{max_loaded}]"));
        ui.label("");

        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
            ui.label("Inventory");
            ui.add(Separator::default().horizontal());
        });
        ui.label("");
        for (gun, state) in inventory.guns.iter() {
            if *gun != equipped && state.present {
                let ammo_loaded = state.ammo_loaded;
                let ammo_max = gun.get_info().max_load;
                ui.label(format!("{gun:>7} [{ammo_loaded}/{ammo_max}]"));
            }
            let extra_ammo = state.ammo_available;
            if extra_ammo > 0 {
                ui.label(format!("{extra_ammo} {gun:>7} ammo"));
            }
        }
        ui.label("");

        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
            ui.label("Controls");
            ui.add(Separator::default().horizontal());
        });
        ui.label("");
        ui.label("move: WASD");
        ui.label("shoot: click");
        ui.label("reload: R");
    });
    if settings.show_debug_settings {
        egui::TopBottomPanel::bottom("debug_panel").show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::RED, "DEBUG SETTINGS");
                ui.separator();
                ui.checkbox(&mut settings.debug_scroll, "allow scroll");
                ui.separator();
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
            .add_systems(Update, update);
    }
}
