use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use bevy::prelude::*;

use crate::{
    player::{FlashlightInfo, GunType, MouseWorldCoords, Player},
    spawn::SpawnEvent,
    ui::UiSettings,
};

pub const TILE_WIDTH: f32 = 32.0;
pub const TILE_HEIGHT: f32 = 48.0;

#[derive(Component, Debug, Clone)]
pub struct MapPos(pub IVec2);

impl MapPos {
    pub fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: TILE_WIDTH * self.0.x as f32,
            y: TILE_HEIGHT * self.0.y as f32,
        }
    }
    pub fn from_vec3(vec3: Vec3) -> Self {
        Self::from_vec2(vec3.xy())
    }
    pub fn from_vec2(vec2: Vec2) -> Self {
        Self(IVec2 {
            x: (vec2.x / TILE_WIDTH) as i32,
            y: (vec2.y / TILE_HEIGHT) as i32,
        })
    }
    pub fn corners(&self) -> [Vec2; 4] {
        [
            self.to_vec2(),
            self.to_vec2() + Vec2::new(0.0, 1.0),
            self.to_vec2() + Vec2::new(1.0, 0.0),
            self.to_vec2() + Vec2::new(1.0, 1.0),
        ]
    }
}

#[derive(Component)]
pub struct BlocksMovement;

#[derive(Component)]
pub struct BlocksSight;

#[derive(Component)]
pub struct Tile(pub TileKind);

#[derive(Debug, Clone, Copy)]
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
    pub fn get(&self, pos: IVec2) -> impl Iterator<Item = &Entity> {
        self.0.get(&pos).into_iter().flatten()
    }
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
    Door,
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
            Bush | Crate | Door => false,
        }
    }
    pub fn blocks_sight(&self) -> bool {
        use TileKind::*;
        match self {
            Wall | Tree | Bush | Crate | Door | ShippingContainer => true,
        }
    }
}

#[derive(Default, Resource)]
pub struct SightBlockedMap(pub HashSet<IVec2>);

pub fn update_visibility(
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

pub fn path(
    start: IVec2,
    dest: IVec2,
    max_distance: i32,
    mut blocked: impl FnMut(IVec2) -> bool,
    mut extra_heuristic: impl FnMut(IVec2) -> i32,
) -> Option<Vec<IVec2>> {
    pathfinding::directed::astar::astar(
        &start,
        move |&p| {
            rogue_algebra::Pos::from(p)
                .adjacent_cardinal()
                .map(IVec2::from)
                .into_iter()
                .filter(|&p| p == dest || !blocked(p))
                .filter(|p| p.distance_squared(dest) <= max_distance * max_distance)
                .map(|p| (p, 1))
                .collect::<Vec<_>>()
        },
        move |p| p.distance_squared(dest) + extra_heuristic(*p),
        |p| *p == dest,
    )
    .map(|(path, _cost)| path)
}

pub fn update_walkability(
    query: Query<&MapPos, With<BlocksMovement>>,
    mut walk_map: ResMut<WalkBlockedMap>,
) {
    walk_map.0.clear();
    for pos in query.iter() {
        walk_map.0.insert(pos.0);
    }
}

#[derive(Default, Resource)]
pub struct PlayerVisibilityMap(pub HashSet<IVec2>);

pub fn update_player_visibility(
    mut player_vis_map: ResMut<PlayerVisibilityMap>,
    q_player: Query<&MapPos, With<Player>>,
    sight_blocked_map: Res<SightBlockedMap>,
) {
    player_vis_map.0.clear();
    let player_pos = q_player.single();
    for pos in rogue_algebra::fov::calculate_fov(player_pos.0.into(), 99, |pos| {
        sight_blocked_map.0.contains(&pos.into())
    }) {
        player_vis_map.0.insert(pos.into());
    }
}

#[derive(Default, Resource)]
pub struct FlashlightMap(pub HashSet<IVec2>);

pub fn update_flashlight_map(
    player_vis_map: Res<PlayerVisibilityMap>,
    mut flashlight_map: ResMut<FlashlightMap>,
    mouse_world_coords: Res<MouseWorldCoords>,
    q_player: Query<&Transform, With<Player>>,
    flashlight_info: Res<FlashlightInfo>,
) {
    flashlight_map.0.clear();
    let player_pos = q_player.single().translation.xy();
    let flashlight_dir = mouse_world_coords.0 - player_pos;
    let allowed_angle_radians = flashlight_info.cone_width_degrees * (PI / 180.0);
    for &p in player_vis_map.0.iter() {
        for world_pos in MapPos(p).corners() {
            if (world_pos - player_pos).angle_to(flashlight_dir).abs() <= allowed_angle_radians {
                flashlight_map.0.insert(p);
            }
        }
    }
}

#[derive(Default, Component)]
pub struct LightsUp {
    pub is_lit: bool,
    pub is_brightly_lit: bool,
    // increases when focused and lit, never goes down
    pub lit_factor: f32,
}

pub fn update_lit(
    flashlight_map: Res<FlashlightMap>,
    flashlight_info: Res<FlashlightInfo>,
    mut q_lights_up: Query<(&MapPos, &mut LightsUp)>,
    time: Res<Time>,
) {
    for (pos, mut lit) in q_lights_up.iter_mut() {
        lit.is_lit = flashlight_map.0.contains(&pos.0);
        lit.is_brightly_lit = lit.is_lit && flashlight_info.focused;
        lit.lit_factor += match (lit.is_lit, lit.is_brightly_lit) {
            (true, true) => time.delta_secs(), // * 2.0,
            // (true, false) => time.delta_secs(),
            _ => 0.0,
        };
    }
}

// like the flashlight map but more forgiving
#[derive(Default, Resource)]
pub struct FovMap(pub HashSet<IVec2>);

pub fn update_fov_map(
    mut fov_map: ResMut<FovMap>,
    vis_map: Res<PlayerVisibilityMap>,
    mouse_world_coords: Res<MouseWorldCoords>,
    q_player: Query<&Transform, With<Player>>,
) {
    pub const FOV_CONE_DEGREES: f32 = 70.0;
    fov_map.0.clear();
    let player_pos = q_player.single().translation.xy();
    let flashlight_dir = mouse_world_coords.0 - player_pos;
    let allowed_angle_radians = FOV_CONE_DEGREES * (PI / 180.0);
    for &p in vis_map.0.iter() {
        for world_pos in MapPos(p).corners() {
            if (world_pos - player_pos).angle_to(flashlight_dir).abs() <= allowed_angle_radians {
                fov_map.0.insert(p);
            }
        }
    }
}

pub fn apply_visibility(
    player_vis_map: Res<PlayerVisibilityMap>,
    flashlight_map: Res<FlashlightMap>,
    fov_map: Res<FovMap>,
    mut query: Query<(&MapPos, &mut Visibility)>,
    settings: Res<UiSettings>,
) {
    for (map_pos, mut visibility) in query.iter_mut() {
        *visibility = if settings.show_visibility && !player_vis_map.0.contains(&map_pos.0)
            || settings.show_flashlight && !flashlight_map.0.contains(&map_pos.0)
            || settings.show_fov && !fov_map.0.contains(&map_pos.0)
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

#[derive(Default)]
pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>();
        app.init_resource::<SightBlockedMap>();
        app.init_resource::<WalkBlockedMap>();
        app.init_resource::<PlayerVisibilityMap>();
        app.init_resource::<FlashlightMap>();
        app.init_resource::<FovMap>();
        app.add_systems(Startup, startup);
        app.add_systems(
            Update,
            (
                update_tilemap,
                update_visibility,
                update_walkability,
                update_player_visibility,
                update_flashlight_map,
                update_fov_map,
                apply_visibility,
                update_lit,
            )
                .chain()
                .after(crate::spawn::spawn),
        );
        app.add_event::<SpawnEvent>();
    }
}
