use std::collections::HashMap;

use bevy::prelude::*;

pub const TILE_SIZE: f32 = 48.0;

#[derive(Component)]
pub struct MapPos(pub IVec2);

impl MapPos {
    pub fn to_vec3(&self, z: f32) -> Vec3 {
        Vec3 {
            x: TILE_SIZE * self.0.x as f32,
            y: TILE_SIZE * self.0.y as f32,
            z,
        }
    }
}

#[derive(Component)]
pub struct BlocksMovement;

#[derive(Default, Resource)]
pub struct Map(pub HashMap<IVec2, Vec<Entity>>);

fn update_tilemap(mut tile_map: ResMut<Map>, query: Query<(Entity, &MapPos)>) {
    tile_map.0.clear();
    for (entity, MapPos(vec2)) in query.iter() {
        tile_map.0.entry(*vec2).or_default().push(entity);
    }
}

#[derive(Resource)]
pub struct WorldAssets {
    pub square: Handle<Mesh>,
    pub white: Handle<ColorMaterial>,
    pub green: Handle<ColorMaterial>,
    pub dark_green: Handle<ColorMaterial>,
    pub red: Handle<ColorMaterial>,
    pub purple: Handle<ColorMaterial>,
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(WorldAssets {
        square: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
        white: materials.add(Color::LinearRgba(LinearRgba::WHITE)),
        green: materials.add(Color::LinearRgba(LinearRgba::GREEN)),
        dark_green: materials.add(Color::LinearRgba(LinearRgba::rgb(0.0, 0.5, 0.0))),
        red: materials.add(Color::LinearRgba(LinearRgba::RED)),
        purple: materials.add(Color::LinearRgba(LinearRgba::rgb(1.0, 0.0, 1.0))),
    });
}

fn startup(mut ev_spawn: EventWriter<SpawnEvent>) {
    for (pos, spawn_list) in crate::mapgen::gen_map() {
        for spawn in spawn_list.into_iter() {
            ev_spawn.send(SpawnEvent(pos, spawn));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TileKind {
    Wall,
    Bush,
    Tree,
}

impl TileKind {
    pub fn blocks_movement(&self) -> bool {
        use TileKind::*;
        match self {
            Wall | Tree => true,
            Bush => false,
        }
    }
}

pub enum MobKind {
    Zombie,
}

pub enum Spawn {
    Tile(TileKind),
    Mob(MobKind),
}

impl Spawn {
    fn blocks_movement(&self) -> bool {
        match self {
            Self::Tile(tk) => tk.blocks_movement(),
            Self::Mob(_) => true,
        }
    }
}

#[derive(Component)]
#[allow(unused)]
struct Mob {
    saw_player_at: Option<IVec2>,
    move_timer: Timer,
}

#[derive(Event)]
struct SpawnEvent(IVec2, Spawn);

fn spawn(
    mut commands: Commands,
    world_assets: Res<WorldAssets>,
    mut ev_new_tile: EventReader<SpawnEvent>,
) {
    for SpawnEvent(pos, spawn) in ev_new_tile.read() {
        let color = match spawn {
            Spawn::Tile(TileKind::Wall) => world_assets.white.clone(),
            Spawn::Tile(TileKind::Bush) => world_assets.green.clone(),
            Spawn::Tile(TileKind::Tree) => world_assets.dark_green.clone(),
            Spawn::Mob(MobKind::Zombie) => world_assets.purple.clone(),
        };
        let mut entity_commands = commands.spawn((
            Mesh2d(world_assets.square.clone()),
            MeshMaterial2d(color),
            MapPos(*pos),
            Transform::from_translation(Vec3::new(
                TILE_SIZE * pos.x as f32,
                TILE_SIZE * pos.y as f32,
                0.0,
            )),
        ));
        if spawn.blocks_movement() {
            entity_commands.insert(BlocksMovement);
        }
    }
}

#[derive(Default)]
pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>();
        app.add_systems(PreStartup, init_assets);
        app.add_systems(Startup, startup);
        app.add_systems(PreUpdate, update_tilemap);
        app.add_systems(FixedUpdate, spawn);
        app.add_event::<SpawnEvent>();
    }
}
