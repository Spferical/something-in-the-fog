use std::collections::HashMap;

use bevy::prelude::*;

pub const TILE_SIZE: f32 = 24.0;

#[derive(Component)]
#[allow(unused)]
pub struct WorldPos(IVec2);

#[allow(unused)]
#[derive(Default, Resource)]
pub struct TileMap(HashMap<IVec2, Vec<Entity>>);

#[allow(unused)]
fn update_tilemap(mut tile_map: ResMut<TileMap>, query: Query<(Entity, &WorldPos)>) {
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
    ev_new_tile.send(NewTileEvent(IVec2::new(3, 3), TileKind::Wall));
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
        commands.spawn((
            Mesh2d(world_assets.square.clone()),
            MeshMaterial2d(color),
            WorldPos(*pos),
            Transform::from_translation(Vec3::new(
                TILE_SIZE * pos.x as f32,
                TILE_SIZE * pos.y as f32,
                0.0,
            )),
        ));
    }
}

#[derive(Default)]
pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMap>();
        app.add_systems(Startup, startup);
        app.add_systems(FixedUpdate, make_tiles);
        app.add_event::<NewTileEvent>();
    }
}
