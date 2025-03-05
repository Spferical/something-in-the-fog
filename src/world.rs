use std::collections::HashMap;

use bevy::{prelude::*, render::view::RenderLayers};

pub const TILE_SIZE: f32 = 24.0;

#[derive(Component)]
#[allow(unused)]
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

#[allow(unused)]
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
    square: Handle<Mesh>,
    white: Handle<ColorMaterial>,
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ev_new_tile: EventWriter<NewTileEvent>,
) {
    commands.insert_resource(WorldAssets {
        square: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
        white: materials.add(Color::LinearRgba(LinearRgba::WHITE)),
    });
    for (x, y) in [(3, 3), (3, 4), (3, 5), (3, 6), (2, 6), (1, 6), (0, 6)] {
        ev_new_tile.send(NewTileEvent(IVec2::new(x, y), TileKind::Wall));
    }
}

enum TileKind {
    Wall,
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
            RenderLayers::layer(1),
        ));
        if matches!(kind, TileKind::Wall) {
            entity_commands.insert(BlocksMovement);
        }
    }
}

#[derive(Default)]
pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMap>();
        app.add_systems(Startup, startup);
        app.add_systems(PreUpdate, update_tilemap);
        app.add_systems(Update, make_tiles);
        app.add_event::<NewTileEvent>();
    }
}
