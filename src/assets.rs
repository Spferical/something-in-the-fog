use std::collections::HashMap;

use bevy::{prelude::*, sprite::AlphaMode2d};

use crate::{
    map::{ItemKind, TILE_HEIGHT, TILE_WIDTH, TileKind},
    mob::MobKind,
    spawn::Spawn,
};

pub static PRESS_START_2P_BYTES: &[u8] =
    include_bytes!("../assets/PressStart2P/PressStart2P-Regular.ttf");

#[derive(Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub square: Handle<Mesh>,
    pub pixel: Handle<Mesh>,
    pub white: Handle<ColorMaterial>,
    pub sight_line: Handle<ColorMaterial>,
    pub reload_indicator_mesh: Handle<Mesh>,
    pub reload_indicator_material: Handle<ColorMaterial>,
    pub hurt_effect_material: Handle<ColorMaterial>,
    pub fade_out_material: Handle<ColorMaterial>,
    pub sheets: HashMap<SpriteSheet, (Handle<Image>, Handle<TextureAtlasLayout>)>,
}

#[derive(PartialEq, Eq, Hash)]
pub enum SpriteSheet {
    Urizen,
    OryxAvatar,
    OryxTerrain,
    OryxTerrainObjects,
    OryxMonsters,
}

#[derive(Debug)]
pub enum SpriteKind {
    Player,
    Spawn(Spawn),
}

impl GameAssets {
    pub fn get_sprite_by_index(&self, sheet: SpriteSheet, index: usize) -> Sprite {
        let (texture, layout) = self.sheets.get(&sheet).unwrap();
        let mut sprite = Sprite::from_atlas_image(
            texture.clone(),
            TextureAtlas {
                layout: layout.clone(),
                index,
            },
        );
        sprite.custom_size = match sheet {
            SpriteSheet::Urizen => Some(Vec2::new(24.0, 24.0)),
            _ => Some(Vec2::new(32.0, 48.0)),
        };
        sprite
    }
    pub fn get_sprite(&self, kind: SpriteKind) -> Sprite {
        let mut sprite = match kind {
            SpriteKind::Player => self.get_sprite_by_index(SpriteSheet::OryxAvatar, 1),
            SpriteKind::Spawn(Spawn::Tile(TileKind::Wall)) => {
                self.get_sprite_by_index(SpriteSheet::OryxTerrain, 0)
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::ShippingContainer)) => {
                self.get_sprite_by_index(SpriteSheet::OryxTerrainObjects, 7)
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::Crate)) => {
                self.get_sprite_by_index(SpriteSheet::OryxTerrainObjects, 20 * 3 + 5)
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::Bush)) => {
                self.get_sprite_by_index(SpriteSheet::OryxTerrainObjects, 20 * 10 + 3)
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::Tree)) => {
                self.get_sprite_by_index(SpriteSheet::OryxTerrainObjects, 20 * 6 + 6)
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::Door)) => {
                self.get_sprite_by_index(SpriteSheet::OryxTerrainObjects, 20 + 3)
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::Zombie)) => {
                self.get_sprite_by_index(SpriteSheet::OryxAvatar, 1)
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::Sculpture)) => {
                self.get_sprite_by_index(SpriteSheet::OryxTerrainObjects, 20 * 3)
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::Hider)) => {
                self.get_sprite_by_index(SpriteSheet::OryxMonsters, 4 * 19 + 2)
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::KoolAidMan)) => {
                self.get_sprite_by_index(SpriteSheet::OryxMonsters, 14 * 19 + 15)
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Ammo(..))) => {
                self.get_sprite_by_index(SpriteSheet::Urizen, 103 * 22 + 52)
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Gun(..))) => {
                self.get_sprite_by_index(SpriteSheet::Urizen, 103 * 22 + 52)
            }
        };
        sprite.color = match kind {
            SpriteKind::Player => Color::LinearRgba(LinearRgba::WHITE),
            SpriteKind::Spawn(Spawn::Tile(TileKind::Wall)) => Color::LinearRgba(LinearRgba::WHITE),
            SpriteKind::Spawn(Spawn::Tile(TileKind::ShippingContainer)) => {
                Color::srgba_u8(0xad, 0x4e, 0x37, 0xff)
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::Crate)) => {
                Color::LinearRgba(bevy::color::palettes::basic::GRAY.into())
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::Bush)) => Color::LinearRgba(LinearRgba::GREEN),
            SpriteKind::Spawn(Spawn::Tile(TileKind::Tree)) => {
                Color::LinearRgba(LinearRgba::rgb(0.0, 0.5, 0.0))
            }
            SpriteKind::Spawn(Spawn::Tile(TileKind::Door)) => {
                Color::srgba_u8(0xad, 0x4e, 0x37, 0xff)
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::Zombie)) => {
                Color::LinearRgba(LinearRgba::rgb(1.0, 0.0, 1.0))
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::Sculpture)) => {
                Color::srgba_u8(0xad, 0x4e, 0x37, 0xff)
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::Hider)) => {
                Color::LinearRgba(bevy::color::palettes::basic::AQUA.into())
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::KoolAidMan)) => {
                Color::LinearRgba(LinearRgba::RED)
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Ammo(..))) => {
                Color::LinearRgba(bevy::color::palettes::basic::GRAY.into())
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Gun(..))) => {
                Color::LinearRgba(bevy::color::palettes::basic::GRAY.into())
            }
        };
        sprite
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
    let mut sheets = HashMap::new();
    sheets.insert(
        SpriteSheet::Urizen,
        (
            asset_server.load("urizen_onebit_tileset__v1d1.png"),
            texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(12),
                103,
                50,
                Some(UVec2::splat(1)),
                Some(UVec2::splat(1)),
            )),
        ),
    );
    sheets.insert(
        SpriteSheet::OryxAvatar,
        (
            asset_server.load("oryx_roguelike_2.0/Avatar.png"),
            texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(16, 24),
                6,
                2,
                None,
                None,
            )),
        ),
    );
    sheets.insert(
        SpriteSheet::OryxTerrain,
        (
            asset_server.load("oryx_roguelike_2.0/Terrain.png"),
            texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(16, 24),
                16,
                11,
                None,
                None,
            )),
        ),
    );
    sheets.insert(
        SpriteSheet::OryxTerrainObjects,
        (
            asset_server.load("oryx_roguelike_2.0/Terrain_Objects.png"),
            texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(16, 24),
                20,
                12,
                None,
                None,
            )),
        ),
    );
    sheets.insert(
        SpriteSheet::OryxMonsters,
        (
            asset_server.load("oryx_roguelike_2.0/Monsters.png"),
            texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(16, 24),
                19,
                26,
                None,
                None,
            )),
        ),
    );
    commands.insert_resource(GameAssets {
        font: fonts.add(Font::try_from_bytes(PRESS_START_2P_BYTES.into()).unwrap()),
        sheets,
        square: meshes.add(Rectangle::new(
            TILE_WIDTH.min(TILE_HEIGHT),
            TILE_WIDTH.min(TILE_HEIGHT),
        )),
        reload_indicator_mesh: meshes.add(CircularSector::from_degrees(
            TILE_WIDTH.min(TILE_HEIGHT),
            360.0,
        )),
        pixel: meshes.add(Rectangle::new(1.0, 1.0)),
        white: materials.add(Color::LinearRgba(LinearRgba::WHITE)),
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
