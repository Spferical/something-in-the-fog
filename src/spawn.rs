use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    Z_ITEMS, Z_MOBS, Z_TILES,
    assets::{GameAssets, SpriteKind},
    map::{
        BlocksMovement, BlocksSight, ItemKind, LightsUp, MapPos, Pickup, TILE_HEIGHT, TILE_WIDTH,
        Tile, TileKind,
    },
    mob::{HearsPlayer, KoolAidMovement, Mob, MobKind, SeesPlayer},
};

#[derive(Debug, Clone)]
pub enum Spawn {
    Tile(TileKind),
    Mob(MobKind),
    Item(ItemKind),
}

impl Spawn {
    fn blocks_movement(&self) -> bool {
        match self {
            Self::Tile(tk) => tk.blocks_movement(),
            Self::Mob(_) => true,
            Self::Item(_) => false,
        }
    }
}

#[derive(Event)]
pub struct SpawnEvent(pub IVec2, pub Spawn);

pub fn spawn(
    mut commands: Commands,
    world_assets: Res<GameAssets>,
    mut ev_spawn: EventReader<SpawnEvent>,
) {
    for SpawnEvent(pos, spawn) in ev_spawn.read() {
        let sprite = world_assets.get_sprite(SpriteKind::Spawn(spawn.clone()));
        let z = match spawn {
            Spawn::Tile(..) => Z_TILES,
            Spawn::Mob(..) => Z_MOBS,
            Spawn::Item(..) => Z_ITEMS,
        };
        let mut entity_commands = commands.spawn((
            sprite,
            MapPos(*pos),
            Transform::from_translation(Vec3::new(
                TILE_WIDTH * pos.x as f32,
                TILE_HEIGHT * pos.y as f32,
                z,
            )),
            RenderLayers::layer(1),
        ));
        if spawn.blocks_movement() {
            entity_commands.insert(BlocksMovement);
        }
        match spawn {
            Spawn::Tile(t) => {
                if t.blocks_sight() {
                    entity_commands.insert(BlocksSight);
                }
                entity_commands.insert(Tile(*t));
            }
            Spawn::Mob(kind) => {
                entity_commands.insert((
                    SeesPlayer,
                    HearsPlayer,
                    Mob {
                        move_timer: Timer::new(kind.get_move_delay(), TimerMode::Once),
                        damage: 0,
                        kind: *kind,
                    },
                    LightsUp::default(),
                ));
                if let MobKind::KoolAidMan = kind {
                    entity_commands.insert(KoolAidMovement::default());
                }
            }
            Spawn::Item(kind) => {
                entity_commands.insert(Pickup(*kind));
            }
        }
    }
}

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn);
    }
}
