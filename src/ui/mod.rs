use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, Align, Color32, FontData, FontFamily, Separator, TextStyle},
};

use crate::{
    assets::PRESS_START_2P_BYTES,
    mob::MobKind,
    player::{
        FLASHLIGHT_MAX_BATTERY, FlashlightInfo, GunInfo, GunState, Inventory, PLAYER_MAX_DAMAGE,
        Player,
    },
    spawn::Spawn,
};

pub mod performance;

#[derive(Default)]
pub(crate) struct UiPlugin;

#[derive(Resource)]
pub struct UiSettings {
    pub show_performance_overlay: bool,
    pub show_debug_settings: bool,
    pub debug_scroll: bool,
    pub show_visibility: bool,
    pub nohurt: bool,
    pub inf_ammo: bool,
    pub toggle_2d: bool,
    pub show_fov: bool,
    pub show_flashlight: bool,
    pub low_graphics: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_performance_overlay: false,
            show_debug_settings: true,
            debug_scroll: false,
            show_visibility: false,
            show_fov: false,
            show_flashlight: false,
            nohurt: true,
            inf_ammo: false,
            toggle_2d: false,
            low_graphics: false,
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
    Spawn(Spawn),
}

fn update(
    mut contexts: EguiContexts,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<UiSettings>,
    mut ev: EventWriter<UiEvent>,
    inventory: Res<Inventory>,
    player: Query<&Player>,
    flashlight: Res<FlashlightInfo>,
) {
    settings.show_performance_overlay ^= keyboard_input.just_pressed(KeyCode::F3);
    settings.show_debug_settings ^= keyboard_input.just_pressed(KeyCode::F4);
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
            ui.label("Status");
            ui.add(Separator::default().horizontal());
        });
        ui.horizontal(|ui| {
            ui.label("Health: ");
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let player = player.single();
            ui.colored_label(
                Color32::RED,
                "x".repeat(0.max(PLAYER_MAX_DAMAGE - player.damage) as usize),
            );
            ui.colored_label(
                Color32::GRAY,
                "x".repeat(player.damage.min(PLAYER_MAX_DAMAGE) as usize),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Battery: ");
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            let quantized_juice =
                (7.0 * (flashlight.battery / FLASHLIGHT_MAX_BATTERY)).round() as usize;
            ui.colored_label(Color32::YELLOW, "x".repeat(quantized_juice));
            ui.colored_label(Color32::GRAY, "x".repeat(7 - quantized_juice));
        });

        ui.label("");

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
        ui.label("scroll: swap gun");
        ui.label("right click: focus light");
        ui.label("hold still: focus gun");
        ui.label("");

        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
            ui.label("Settings");
            ui.add(Separator::default().horizontal());
        });
        ui.checkbox(&mut settings.low_graphics, "low graphics");
        ui.label("");

        if settings.show_debug_settings {
            ui.label("");
            ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                ui.colored_label(Color32::RED, "DEBUG SETTINGS");
                ui.add(Separator::default().horizontal());
            });
            ui.label("");
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut settings.debug_scroll, "scroll zoom");
                ui.separator();
                ui.checkbox(&mut settings.show_visibility, "viz");
                ui.separator();
                ui.checkbox(&mut settings.show_fov, "fov");
                ui.separator();
                ui.checkbox(&mut settings.show_flashlight, "flash");
                ui.separator();
                ui.checkbox(&mut settings.nohurt, "nohurt");
                ui.separator();
                ui.checkbox(&mut settings.inf_ammo, "inf ammo");
                ui.separator();
                ui.checkbox(&mut settings.toggle_2d, "toggle_2d");
                ui.separator();
                ui.label("spawn");
                if ui.button("m").clicked() {
                    ev.send(UiEvent::Spawn(Spawn::Mob(MobKind::Sculpture)));
                }
                if ui.button("k").clicked() {
                    ev.send(UiEvent::Spawn(Spawn::Mob(MobKind::KoolAidMan)));
                }
                if ui.button("z").clicked() {
                    ev.send(UiEvent::Spawn(Spawn::Mob(MobKind::Zombie)));
                }
                ui.label("Teleport to... ");
                for i in 0..=5 {
                    if ui.button(format!("{i}")).clicked() {
                        ev.send(UiEvent::TeleportPlayer(i));
                    }
                }
            });
        }
    });
}

// from https://github.com/vladbat00/bevy_egui/issues/47
fn absorb_egui_inputs(
    mut contexts: bevy_egui::EguiContexts,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_wheel: ResMut<Events<MouseWheel>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    let ctx = contexts.ctx_mut();
    if !(ctx.wants_pointer_input() || ctx.is_pointer_over_area()) {
        return;
    }
    let modifiers = [
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ];

    let pressed = modifiers.map(|key| keyboard.pressed(key).then_some(key));

    mouse.reset_all();
    mouse_wheel.clear();
    keyboard.reset_all();

    for key in pressed.into_iter().flatten() {
        keyboard.press(key);
    }
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<UiSettings>()
            .add_event::<UiEvent>()
            .add_systems(Startup, startup)
            .add_systems(Update, update)
            .add_systems(
                PreUpdate,
                absorb_egui_inputs
                    .after(bevy_egui::input::write_egui_input_system)
                    .before(bevy_egui::begin_pass_system),
            );
    }
}
