use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::{player::GunType, spawn::SpawnEvent};

pub const TILE_SIZE: f32 = 48.0;

#[derive(Component, Clone)]
pub struct MapPos(pub IVec2);

impl MapPos {
    pub fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: TILE_SIZE * self.0.x as f32,
            y: TILE_SIZE * self.0.y as f32,
        }
    }
    pub fn from_vec3(vec3: Vec3) -> Self {
        Self(IVec2 {
            x: (vec3.x / TILE_SIZE) as i32,
            y: (vec3.y / TILE_SIZE) as i32,
        })
    }
}

#[derive(Component)]
pub struct BlocksMovement;

#[derive(Component)]
pub struct BlocksSight;

#[derive(Component)]
pub struct Tile(pub TileKind);

#[derive(Clone, Copy)]
pub enum ItemKind {
    Ammo(GunType, usize),
    Gun(GunType, usize),
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemKind::Ammo(gun_type, ammo) => write!(f, "{ammo} {gun_type} ammo"),
            ItemKind::Gun(gun_type, _ammo) => write!(f, "{gun_type}"),
        }
    }
}

#[derive(Component)]
pub struct Pickup(pub ItemKind);

#[derive(Default, Resource)]
pub struct Map(pub HashMap<IVec2, Vec<Entity>>);

impl Map {
    pub fn get_nearby(&self, center: IVec2, radius: i32) -> impl Iterator<Item = &Entity> {
        (center.x - radius..center.x + radius)
            .flat_map(move |x| (center.y - radius..center.y + radius).map(move |y| (x, y)))
            .filter_map(|(x, y)| self.0.get(&IVec2 { x, y }))
            .flatten()
    }
}

fn update_tilemap(mut tile_map: ResMut<Map>, query: Query<(Entity, &MapPos)>) {
    tile_map.0.clear();
    for (entity, MapPos(vec2)) in query.iter() {
        tile_map.0.entry(*vec2).or_default().push(entity);
    }
}

#[derive(Resource)]
pub struct Zones(pub Vec<IRect>);

fn startup(mut commands: Commands, mut ev_spawn: EventWriter<SpawnEvent>) {
    let crate::mapgen::MapgenResult { spawns, zones } = crate::mapgen::gen_map();
    for (pos, spawn_list) in spawns {
        for spawn in spawn_list.into_iter() {
            ev_spawn.send(SpawnEvent(pos, spawn));
        }
    }
    commands.insert_resource(Zones(zones));
}

#[derive(Debug, Clone, Copy)]
pub enum TileKind {
    Wall,
    Bush,
    Tree,
    Crate,
    ShippingContainer,
}

impl TileKind {
    pub fn blocks_movement(&self) -> bool {
        use TileKind::*;
        match self {
            Wall | Tree | ShippingContainer => true,
            Bush | Crate => false,
        }
    }
    pub fn blocks_sight(&self) -> bool {
        use TileKind::*;
        match self {
            Wall | Tree | Bush | Crate | ShippingContainer => true,
        }
    }
}

#[derive(Default, Resource)]
pub struct SightBlockedMap(pub HashSet<IVec2>);

fn update_visibility(
    query: Query<&MapPos, With<BlocksSight>>,
    mut vis_map: ResMut<SightBlockedMap>,
) {
    vis_map.0.clear();
    for pos in query.iter() {
        vis_map.0.insert(pos.0);
    }
}

#[derive(Default, Resource)]
pub struct WalkBlockedMap(pub HashSet<IVec2>);

impl WalkBlockedMap {
    pub fn path(&self, start: IVec2, dest: IVec2, max_distance: i32) -> Option<Vec<IVec2>> {
        pathfinding::directed::astar::astar(
            &start,
            |&p| {
                rogue_algebra::CARDINALS
                    .iter()
                    .copied()
                    .map(move |c| p + IVec2::from(c))
                    .filter(|p| *p == dest || !self.0.contains(p))
                    .filter(|p| p.distance_squared(dest) <= max_distance * max_distance)
                    .map(|p| (p, 1))
            },
            |p| p.distance_squared(dest),
            |p| *p == dest,
        )
        .map(|(path, _cost)| path)
    }
}

fn update_walkability(
    query: Query<&MapPos, With<BlocksMovement>>,
    mut walk_map: ResMut<WalkBlockedMap>,
) {
    walk_map.0.clear();
    for pos in query.iter() {
        walk_map.0.insert(pos.0);
    }
}

#[derive(Default)]
pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>();
        app.init_resource::<SightBlockedMap>();
        app.init_resource::<WalkBlockedMap>();
        app.add_systems(Startup, startup);
        app.add_systems(PreUpdate, update_tilemap);
        app.add_systems(Update, (update_visibility, update_walkability).chain());
        app.add_event::<SpawnEvent>();
    }
}
