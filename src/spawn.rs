use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    Z_ITEMS, Z_MOBS, Z_TILES,
    assets::GameAssets,
    map::{BlocksMovement, BlocksSight, ItemKind, MapPos, Pickup, TILE_SIZE, Tile, TileKind},
    mob::{Mob, MobKind},
};

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

fn spawn(
    mut commands: Commands,
    world_assets: Res<GameAssets>,
    mut ev_spawn: EventReader<SpawnEvent>,
) {
    for SpawnEvent(pos, spawn) in ev_spawn.read() {
        let color = match spawn {
            Spawn::Tile(TileKind::Wall) => world_assets.white.clone(),
            Spawn::Tile(TileKind::ShippingContainer) => world_assets.brown.clone(),
            Spawn::Tile(TileKind::Crate) => world_assets.gray.clone(),
            Spawn::Tile(TileKind::Bush) => world_assets.green.clone(),
            Spawn::Tile(TileKind::Tree) => world_assets.dark_green.clone(),
            Spawn::Mob(MobKind::Zombie) => world_assets.purple.clone(),
            Spawn::Mob(MobKind::Sculpture) => world_assets.brown.clone(),
            Spawn::Mob(MobKind::Hider) => world_assets.aqua.clone(),
            Spawn::Item(ItemKind::Ammo(..)) => world_assets.gray.clone(),
            Spawn::Item(ItemKind::Gun(..)) => world_assets.gray.clone(),
        };
        let mesh = match spawn {
            Spawn::Tile(_) => world_assets.square.clone(),
            Spawn::Mob(_) => world_assets.circle.clone(),
            Spawn::Item(ItemKind::Ammo(..)) => world_assets.small_square.clone(),
            Spawn::Item(ItemKind::Gun(..)) => world_assets.small_square.clone(),
        };
        let z = match spawn {
            Spawn::Tile(..) => Z_TILES,
            Spawn::Mob(..) => Z_MOBS,
            Spawn::Item(..) => Z_ITEMS,
        };
        let mut entity_commands = commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(color),
            MapPos(*pos),
            Transform::from_translation(Vec3::new(
                TILE_SIZE * pos.x as f32,
                TILE_SIZE * pos.y as f32,
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
                entity_commands.insert(Mob {
                    saw_player_at: None,
                    move_timer: Timer::new(kind.get_move_delay(), TimerMode::Once),
                    damage: 0,
                    kind: *kind,
                });
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
