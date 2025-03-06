use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use bevy::{prelude::*, render::view::RenderLayers};
use line_drawing::Bresenham;

use crate::{Player, animation::MoveAnimation, assets::GameAssets};

pub const TILE_SIZE: f32 = 48.0;
pub const ZOMBIE_MOVE_DELAY: Duration = Duration::from_secs(1);

#[derive(Component)]
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
}

impl TileKind {
    pub fn blocks_movement(&self) -> bool {
        use TileKind::*;
        match self {
            Wall | Tree => true,
            Bush | Crate => false,
        }
    }
    pub fn blocks_sight(&self) -> bool {
        use TileKind::*;
        match self {
            Wall | Tree | Bush | Crate => true,
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
pub struct Mob {
    saw_player_at: Option<IVec2>,
    #[allow(unused)]
    move_timer: Timer,
    damage: i32,
}

#[derive(Event)]
pub struct MobDamageEvent {
    pub damage: i32,
    pub entity: Entity,
}

fn damage_mobs(
    mut commands: Commands,
    mut q_mob: Query<(Entity, &mut Mob)>,
    mut ev_mob_damage: EventReader<MobDamageEvent>,
) {
    for MobDamageEvent { damage, entity } in ev_mob_damage.read() {
        if let Ok((entity, mut mob)) = q_mob.get_mut(*entity) {
            mob.damage += damage;
            if mob.damage >= 3 {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Event)]
struct SpawnEvent(IVec2, Spawn);

fn spawn(
    mut commands: Commands,
    world_assets: Res<GameAssets>,
    mut ev_new_tile: EventReader<SpawnEvent>,
) {
    for SpawnEvent(pos, spawn) in ev_new_tile.read() {
        let color = match spawn {
            Spawn::Tile(TileKind::Wall) => world_assets.white.clone(),
            Spawn::Tile(TileKind::Crate) => world_assets.gray.clone(),
            Spawn::Tile(TileKind::Bush) => world_assets.green.clone(),
            Spawn::Tile(TileKind::Tree) => world_assets.dark_green.clone(),
            Spawn::Mob(MobKind::Zombie) => world_assets.purple.clone(),
        };
        let mesh = match spawn {
            Spawn::Tile(_) => world_assets.square.clone(),
            Spawn::Mob(_) => world_assets.circle.clone(),
        };
        let mut entity_commands = commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(color),
            MapPos(*pos),
            Transform::from_translation(Vec3::new(
                TILE_SIZE * pos.x as f32,
                TILE_SIZE * pos.y as f32,
                0.0,
            )),
            RenderLayers::layer(1),
        ));
        if spawn.blocks_movement() {
            entity_commands.insert(BlocksMovement);
        }
        if let Spawn::Tile(t) = spawn {
            if t.blocks_sight() {
                entity_commands.insert(BlocksSight);
            }
            entity_commands.insert(Tile(*t));
        }
        if let Spawn::Mob(_) = spawn {
            entity_commands.insert(Mob {
                saw_player_at: None,
                move_timer: Timer::new(ZOMBIE_MOVE_DELAY, TimerMode::Once),
                damage: 0,
            });
        }
    }
}

#[derive(Default, Resource)]
struct SightBlockedMap(HashSet<IVec2>);

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

fn update_walkability(
    query: Query<&MapPos, With<BlocksMovement>>,
    mut walk_map: ResMut<WalkBlockedMap>,
) {
    walk_map.0.clear();
    for pos in query.iter() {
        walk_map.0.insert(pos.0);
    }
}

fn move_mobs(
    mut commands: Commands,
    mut mobs: Query<(Entity, &mut Mob, &mut MapPos, &mut Transform)>,
    player: Query<&MapPos, (With<Player>, Without<Mob>)>,
    sight_blocked_map: Res<SightBlockedMap>,
    mut walk_blocked_map: ResMut<WalkBlockedMap>,
    time: Res<Time>,
) {
    let player_pos = player.single();
    for (entity, mut mob, mut pos, transform) in mobs.iter_mut() {
        let player_visible = Bresenham::new((pos.0.x, pos.0.y), (player_pos.0.x, player_pos.0.y))
            .skip(1)
            .all(|(x, y)| !sight_blocked_map.0.contains(&IVec2::new(x, y)));
        if player_visible {
            mob.saw_player_at = Some(player_pos.0);
        }
        mob.move_timer.tick(time.delta());
        if mob.move_timer.finished() {
            if let Some(player_pos) = mob.saw_player_at {
                let path = rogue_algebra::path::bfs_paths(
                    &[rogue_algebra::Pos::new(pos.0.x, pos.0.y)],
                    10,
                    |pos| {
                        rogue_algebra::CARDINALS
                            .iter()
                            .copied()
                            .map(|c| pos + c)
                            .filter(|&rogue_algebra::Pos { x, y }| {
                                !walk_blocked_map.0.contains(&IVec2 { x, y })
                            })
                            .collect()
                    },
                )
                .find(|path| {
                    (*path.last().unwrap() - rogue_algebra::Pos::from(player_pos)).mhn_dist() == 1
                });
                if let Some(path) = path {
                    if let Some(move_pos) = path.get(1) {
                        let move_pos = IVec2::from(*move_pos);
                        walk_blocked_map.0.insert(move_pos);
                        pos.0 = move_pos;
                        commands.entity(entity).insert(MoveAnimation {
                            from: transform.translation.truncate(),
                            to: pos.to_vec2(),
                            timer: Timer::new(Duration::from_millis(300), TimerMode::Once),
                            ease: EaseFunction::BounceIn,
                        });
                        mob.move_timer.reset();
                    }
                }
            }
        }
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
        app.add_systems(
            Update,
            (
                spawn,
                update_visibility,
                update_walkability,
                damage_mobs,
                move_mobs,
            )
                .chain(),
        );
        app.add_event::<SpawnEvent>();
        app.add_event::<MobDamageEvent>();
    }
}
