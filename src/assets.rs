use bevy::{prelude::*, sprite::AlphaMode2d};

use crate::map::TILE_SIZE;

pub static PRESS_START_2P_BYTES: &[u8] =
    include_bytes!("../assets/PressStart2P/PressStart2P-Regular.ttf");

#[derive(Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub circle: Handle<Mesh>,
    pub square: Handle<Mesh>,
    pub pixel: Handle<Mesh>,
    pub white: Handle<ColorMaterial>,
    pub gray: Handle<ColorMaterial>,
    pub green: Handle<ColorMaterial>,
    pub dark_green: Handle<ColorMaterial>,
    pub red: Handle<ColorMaterial>,
    pub purple: Handle<ColorMaterial>,
    pub sight_line: Handle<ColorMaterial>,
    pub brown: Handle<ColorMaterial>,
    pub aqua: Handle<ColorMaterial>,
    pub small_square: Handle<Mesh>,
    pub reload_indicator_mesh: Handle<Mesh>,
    pub reload_indicator_material: Handle<ColorMaterial>,
    pub hurt_effect_material: Handle<ColorMaterial>,
    pub fade_out_material: Handle<ColorMaterial>,
    pub urizen_texture: Handle<Image>,
    pub urizen_layout: Handle<TextureAtlasLayout>,
}

impl GameAssets {
    pub fn get_urizen_sprite(&self, index: usize) -> Sprite {
        Sprite::from_atlas_image(
            self.urizen_texture.clone(),
            TextureAtlas {
                layout: self.urizen_layout.clone(),
                index,
            },
        )
    }
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fonts: ResMut<Assets<Font>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let urizen_texture: Handle<Image> = asset_server.load("urizen_onebit_tileset__v1d1.png");
    let urizen_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(12),
        103,
        50,
        Some(UVec2::splat(1)),
        Some(UVec2::splat(1)),
    ));
    commands.insert_resource(GameAssets {
        font: fonts.add(Font::try_from_bytes(PRESS_START_2P_BYTES.into()).unwrap()),
        urizen_texture,
        urizen_layout,
        square: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
        circle: meshes.add(Circle::new(TILE_SIZE / 2.0)),
        reload_indicator_mesh: meshes.add(CircularSector::from_degrees(TILE_SIZE, 360.0)),
        pixel: meshes.add(Rectangle::new(1.0, 1.0)),
        small_square: meshes.add(Rectangle::new(10.0, 10.0)),
        white: materials.add(Color::LinearRgba(LinearRgba::WHITE)),
        gray: materials.add(Color::LinearRgba(bevy::color::palettes::basic::GRAY.into())),
        brown: materials.add(Color::srgba_u8(0xad, 0x4e, 0x37, 0xff)),
        green: materials.add(Color::LinearRgba(LinearRgba::GREEN)),
        dark_green: materials.add(Color::LinearRgba(LinearRgba::rgb(0.0, 0.5, 0.0))),
        red: materials.add(Color::LinearRgba(LinearRgba::RED)),
        purple: materials.add(Color::LinearRgba(LinearRgba::rgb(1.0, 0.0, 1.0))),
        aqua: materials.add(Color::LinearRgba(bevy::color::palettes::basic::AQUA.into())),
        sight_line: materials.add(ColorMaterial {
            color: Color::Srgba(bevy::color::palettes::basic::YELLOW.with_alpha(0.5)),
            alpha_mode: AlphaMode2d::Opaque,
            ..default()
        }),
        reload_indicator_material: materials.add(Color::Srgba(
            bevy::color::palettes::basic::YELLOW.with_alpha(0.25),
        )),
        hurt_effect_material: materials.add(Color::Srgba(Srgba::RED.with_alpha(0.0))),
        fade_out_material: materials.add(Color::Srgba(Srgba::BLACK.with_alpha(0.0))),
    });
}

#[derive(Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_assets);
    }
}
