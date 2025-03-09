use std::{collections::HashMap, path::Path};

use bevy::prelude::*;

use crate::{
    map::{ItemKind, TILE_HEIGHT, TILE_WIDTH, TileKind},
    mob::MobKind,
    player::GunType,
    spawn::Spawn,
};

pub static PRESS_START_2P_BYTES: &[u8] =
    include_bytes!("../assets/PressStart2P/PressStart2P-Regular.ttf");

#[derive(Resource, Default)]
pub struct Sfx {
    pub base_track: Handle<AudioSource>,
    pub active_track: Handle<AudioSource>,
    pub monk_track: Handle<AudioSource>,

    pub reload_pistol: Vec<Handle<AudioSource>>,
    pub reload_shotgun: Vec<Handle<AudioSource>>,
    pub fire_pistol: Vec<Handle<AudioSource>>,
    pub fire_shotgun: Vec<Handle<AudioSource>>,
    pub empty_pistol: Vec<Handle<AudioSource>>,
    pub empty_shotgun: Vec<Handle<AudioSource>>,
}

#[derive(Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub pixel: Handle<Mesh>,
    pub white: Handle<ColorMaterial>,
    pub sight_line: Handle<ColorMaterial>,
    pub reload_indicator_mesh: Handle<Mesh>,
    pub reload_indicator_material: Handle<ColorMaterial>,
    pub sheets: HashMap<SpriteSheet, (Handle<Image>, Handle<TextureAtlasLayout>)>,
    pub sfx: Sfx,
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
                self.get_sprite_by_index(SpriteSheet::OryxTerrainObjects, 20 + 10)
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
            SpriteKind::Spawn(Spawn::Mob(MobKind::Ghost)) => {
                self.get_sprite_by_index(SpriteSheet::OryxMonsters, 16 * 19 + 2)
            }
            SpriteKind::Spawn(Spawn::Mob(MobKind::KoolAidMan)) => {
                self.get_sprite_by_index(SpriteSheet::OryxMonsters, 14 * 19 + 15)
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Ammo(GunType::Pistol, ..))) => {
                self.get_sprite_by_index(SpriteSheet::Urizen, 103 * 22 + 68)
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Ammo(GunType::Shotgun, ..))) => {
                self.get_sprite_by_index(SpriteSheet::Urizen, 103 * 22 + 71)
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Gun(GunType::Pistol, ..))) => {
                self.get_sprite_by_index(SpriteSheet::Urizen, 103 * 22 + 52)
            }
            SpriteKind::Spawn(Spawn::Item(ItemKind::Gun(GunType::Shotgun, ..))) => {
                self.get_sprite_by_index(SpriteSheet::Urizen, 103 * 22 + 57)
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
            SpriteKind::Spawn(Spawn::Mob(MobKind::Ghost)) => Color::LinearRgba(LinearRgba::WHITE),
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

fn add_audio_sources(
    base_dir: &Path,
    track_names: &[&'static str],
    asset_server: &Res<AssetServer>,
) -> Vec<Handle<AudioSource>> {
    track_names
        .iter()
        .map(|track| asset_server.load::<AudioSource>(base_dir.join(track)))
        .collect()
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fonts: ResMut<Assets<Font>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let base_track = asset_server.load::<AudioSource>("sfx/music/base_layer.ogg");
    let active_track = asset_server.load::<AudioSource>("sfx/music/active_layer.ogg");
    let monk_track = asset_server.load::<AudioSource>("sfx/music/scp_layer.ogg");

    let empty_pistol = add_audio_sources(
        Path::new("sfx/gun_sounds/dry/pistol"),
        &["Dry Fire 2-1.ogg", "Dry Fire 2-2.ogg"],
        &asset_server,
    );
    let empty_shotgun = add_audio_sources(
        Path::new("sfx/gun_sounds/dry/shotgun"),
        &["Dry Fire 3-1.ogg", "Dry Fire 3-2.ogg"],
        &asset_server,
    );

    let fire_pistol = add_audio_sources(
        Path::new("sfx/gun_sounds/fire/pistol"),
        &[
            "Gunshot 1-1.ogg",
            "Gunshot 1-2.ogg",
            "Gunshot 1-3.ogg",
            "Gunshot 1-4.ogg",
            "Gunshot 1-5.ogg",
        ],
        &asset_server,
    );
    let fire_shotgun = add_audio_sources(
        Path::new("sfx/gun_sounds/fire/shotgun"),
        &[
            "Gunshot 5-1.ogg",
            "Gunshot 5-2.ogg",
            "Gunshot 5-3.ogg",
            "Gunshot 5-4.ogg",
            "Gunshot 5-5.ogg",
            "Gunshot 5-6.ogg",
        ],
        &asset_server,
    );

    let reload_pistol = add_audio_sources(
        Path::new("sfx/gun_sounds/reload/pistol"),
        &[
            "Unload 1-1.ogg",
            "Unload 1-5.ogg",
            "Unload 1-10.ogg",
            "Unload 1-13.ogg",
            "Unload 1-15.ogg",
            "Unload 1-17.ogg",
        ],
        &asset_server,
    );
    let reload_shotgun = add_audio_sources(
        Path::new("sfx/gun_sounds/reload/shotgun"),
        &["Pumping 2-1.ogg", "Pumping 2-7.ogg", "Pumping 2-10.ogg"],
        &asset_server,
    );

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
        reload_indicator_mesh: meshes.add(CircularSector::from_degrees(
            TILE_WIDTH.min(TILE_HEIGHT),
            360.0,
        )),
        pixel: meshes.add(Rectangle::new(1.0, 1.0)),
        white: materials.add(Color::LinearRgba(LinearRgba::WHITE)),
        sight_line: materials.add(Color::Srgba(
            bevy::color::palettes::basic::WHITE.with_alpha(0.5),
        )),
        reload_indicator_material: materials.add(Color::Srgba(
            bevy::color::palettes::basic::WHITE.with_alpha(0.25),
        )),
        sfx: Sfx {
            base_track,
            active_track,
            monk_track,

            reload_pistol,
            reload_shotgun,
            fire_pistol,
            fire_shotgun,
            empty_pistol,
            empty_shotgun,
            ..default()
        },
    });
}

#[derive(Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_assets);
    }
}
