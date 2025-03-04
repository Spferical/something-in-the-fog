use std::collections::HashMap;

use bevy::prelude::*;

pub const TILE_SIZE: f32 = 48.0;

#[derive(Component)]
pub struct WorldPos(pub IVec2);

impl WorldPos {
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
pub struct TileMap(pub HashMap<IVec2, Vec<Entity>>);

fn update_tilemap(mut tile_map: ResMut<TileMap>, query: Query<(Entity, &WorldPos)>) {
    tile_map.0.clear();
    for (entity, WorldPos(vec2)) in query.iter() {
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
    });
}

fn startup(mut ev_new_tile: EventWriter<NewTileEvent>) {
    for (pos, tile) in crate::mapgen::gen_map() {
        ev_new_tile.send(NewTileEvent(pos, tile));
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum TileKind {
    Wall,
    Bush,
    Tree,
}

impl TileKind {
    fn is_opaque(&self) -> bool {
        use TileKind::*;
        match self {
            Wall | Tree => true,
            Bush => false,
        }
    }
}

#[derive(Event)]
struct NewTileEvent(IVec2, TileKind);

fn make_tiles(
    mut commands: Commands,
    world_assets: Res<WorldAssets>,
    mut ev_new_tile: EventReader<NewTileEvent>,
) {
    for NewTileEvent(pos, kind) in ev_new_tile.read() {
        let color = match kind {
            TileKind::Wall => world_assets.white.clone(),
            TileKind::Bush => world_assets.green.clone(),
            TileKind::Tree => world_assets.dark_green.clone(),
        };
        let mut entity_commands = commands.spawn((
            Mesh2d(world_assets.square.clone()),
            MeshMaterial2d(color),
            WorldPos(*pos),
            Transform::from_translation(Vec3::new(
                TILE_SIZE * pos.x as f32,
                TILE_SIZE * pos.y as f32,
                0.0,
            )),
        ));
        if kind.is_opaque() {
            entity_commands.insert(BlocksMovement);
        }
    }
}

#[derive(Default)]
pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMap>();
        app.add_systems(PreStartup, init_assets);
        app.add_systems(Startup, startup);
        app.add_systems(PreUpdate, update_tilemap);
        app.add_systems(FixedUpdate, make_tiles);
        app.add_event::<NewTileEvent>();
    }
}
