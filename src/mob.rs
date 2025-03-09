use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use line_drawing::WalkGrid;
use rand::seq::SliceRandom;

use crate::{
    Player,
    animation::MoveAnimation,
    map::{
        FlashlightMap, FovMap, Map, MapPos, PlayerVisibilityMap, SightBlockedMap, Tile,
        WalkBlockedMap, Zones, path, update_flashlight_map, update_fov_map, update_visibility,
        update_walkability,
    },
    player::{PlayerDamageEvent, PlayerMoveEvent, ShootEvent},
    spawn::{Spawn, SpawnEvent},
};

const MAX_PATH: i32 = 100;
const HIDER_CHASE_DISTANCE: i32 = 5;
const KOOL_AID_OVERSHOOT: usize = 4;

#[derive(Clone, Copy, Debug)]
pub enum MobKind {
    Zombie,
    Sculpture,
    Hider,
    KoolAidMan,
}

impl MobKind {
    pub fn get_move_delay(&self) -> Duration {
        use MobKind::*;
        match self {
            Zombie => Duration::from_secs(1),
            Sculpture => Duration::from_millis(16),
            Hider => Duration::from_millis(200),
            KoolAidMan => Duration::from_millis(100),
        }
    }

    pub fn max_damage(&self) -> i32 {
        use MobKind::*;
        match self {
            Zombie => 3,
            Sculpture => 99,
            Hider => 2,
            KoolAidMan => 5,
        }
    }

    pub fn get_ease_function_for_movement(&self) -> EaseFunction {
        use MobKind::*;
        match self {
            Zombie => EaseFunction::BounceIn,
            Sculpture => EaseFunction::Linear,
            Hider => EaseFunction::CubicIn,
            KoolAidMan => EaseFunction::BounceOut,
        }
    }
}

#[derive(Component)]
pub struct Mob {
    pub kind: MobKind,
    pub move_timer: Timer,
    pub damage: i32,
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
            if mob.damage >= mob.kind.max_damage() {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn path_to(
    source: IVec2,
    target: IVec2,
    walk_blocked_map: &WalkBlockedMap,
    fov_map: &FovMap,
    flashlight_map: &FlashlightMap,
    avoid_player_sight: bool,
    avoid_player_flashlight: bool,
    path_through_walls: bool,
) -> Option<Vec<IVec2>> {
    path(
        source,
        target,
        MAX_PATH,
        |p| !path_through_walls && walk_blocked_map.0.contains(&p),
        |p| {
            if avoid_player_sight && fov_map.0.contains(&p) {
                99
            } else if avoid_player_flashlight && flashlight_map.0.contains(&p) {
                1
            } else {
                0
            }
        },
    )
}

fn find_hiding_spot(
    source: IVec2,
    walk_blocked_map: &WalkBlockedMap,
    sight_blocked_map: &SightBlockedMap,
) -> Option<IVec2> {
    pathfinding::directed::bfs::bfs_reach(source, |&p| {
        rogue_algebra::Pos::from(p)
            .adjacent_cardinal()
            .map(IVec2::from)
            .into_iter()
            .filter(|p| !walk_blocked_map.0.contains(p))
    })
    .find(|&p| sight_blocked_map.0.contains(&p))
}

#[derive(Component)]
pub struct SeesPlayer;

#[derive(Component)]
pub struct SawPlayer {
    pub pos: IVec2,
    pub time_since: Stopwatch,
}

impl SawPlayer {
    pub fn new(pos: IVec2) -> Self {
        Self {
            pos,
            time_since: Stopwatch::new(),
        }
    }
}

fn update_mobs_seeing_player(
    mut commands: Commands,
    mobs: Query<(Entity, &MapPos, Option<&SawPlayer>), With<SeesPlayer>>,
    player_visibility_map: Res<PlayerVisibilityMap>,
    sight_blocked_map: Res<SightBlockedMap>,
    mut ev_player_move: EventReader<PlayerMoveEvent>,
    player: Query<&MapPos, (With<Player>, Without<Mob>)>,
) {
    let player_pos = player.single();
    let last_player_move = ev_player_move.read().last();
    for (entity, mob_pos, saw_player) in mobs.iter() {
        let player_sees_mob = player_visibility_map.0.contains(&mob_pos.0);
        let player_is_hidden = sight_blocked_map.0.contains(&player_pos.0);
        if player_sees_mob && !player_is_hidden {
            commands.entity(entity).insert(SawPlayer::new(player_pos.0));
        } else if let Some(PlayerMoveEvent { source, dest }) = last_player_move {
            if let Some(SawPlayer {
                pos: last_seen_player_pos,
                ..
            }) = saw_player
            {
                if *last_seen_player_pos == source.0 && player_sees_mob {
                    // Mob saw player move into a hiding spot.
                    commands.entity(entity).insert(SawPlayer::new(dest.0));
                }
            }
        }
    }
}

#[derive(Component)]
pub struct HearsPlayer;

#[derive(Component)]
pub struct HeardPlayer {
    pub pos: IVec2,
    pub time_since: Stopwatch,
}

impl HeardPlayer {
    pub fn new(pos: IVec2) -> Self {
        Self {
            pos,
            time_since: Stopwatch::new(),
        }
    }
}

fn update_hearing_player(
    mut commands: Commands,
    mobs: Query<Entity, With<HearsPlayer>>,
    mut ev_shoot: EventReader<ShootEvent>,
    map: Res<Map>,
) {
    const HEARING_RADIUS: i32 = 20;
    for ShootEvent { start, .. } in ev_shoot.read() {
        let map_pos = MapPos::from_vec2(*start);
        for entity in mobs.iter_many(map.get_nearby(map_pos.0, HEARING_RADIUS)) {
            commands.entity(entity).insert(HeardPlayer::new(map_pos.0));
        }
    }
}

#[allow(clippy::type_complexity)]
fn forget_player(
    mut commands: Commands,
    mut set: ParamSet<(
        Query<(Entity, &MapPos, &mut HeardPlayer)>,
        Query<(Entity, &MapPos, &mut SawPlayer)>,
    )>,
    time: Res<Time>,
) {
    const FORGET_DURATION: Duration = Duration::from_secs(30);
    for (entity, pos, mut heard_player) in set.p0().iter_mut() {
        heard_player.time_since.tick(time.delta());
        if pos.0 == heard_player.pos || heard_player.time_since.elapsed() > FORGET_DURATION {
            // Mob is standing on last seen player position.
            commands.entity(entity).remove::<HeardPlayer>();
        }
    }
    for (entity, pos, mut saw_player) in set.p1().iter_mut() {
        saw_player.time_since.tick(time.delta());
        if pos.0 == saw_player.pos || saw_player.time_since.elapsed() > FORGET_DURATION {
            // Mob is standing on last seen player position.
            commands.entity(entity).remove::<SawPlayer>();
        }
    }
}

#[derive(Component, Debug)]
pub enum KoolAidMovement {
    Moving(Vec<IVec2>),
    Resting(Timer),
}

impl Default for KoolAidMovement {
    fn default() -> Self {
        Self::Resting(Timer::new(Duration::from_secs(0), TimerMode::Once))
    }
}

#[allow(clippy::complexity)]
fn move_mobs(
    mut commands: Commands,
    mut mobs: Query<(
        Entity,
        &mut Mob,
        &mut MapPos,
        &mut Transform,
        Option<&SawPlayer>,
        Option<&HeardPlayer>,
        Option<&mut KoolAidMovement>,
    )>,
    player: Query<&MapPos, (With<Player>, Without<Mob>)>,
    mut walk_blocked_map: ResMut<WalkBlockedMap>,
    sight_blocked_map: Res<SightBlockedMap>,
    fov_map: Res<FovMap>,
    flashlight_map: Res<FlashlightMap>,
    time: Res<Time>,
    mut ev_bust: EventWriter<BustThroughWallEvent>,
    mut ev_player_damage: EventWriter<PlayerDamageEvent>,
) {
    let player_pos = player.single();
    for (entity, mut mob, mut mob_pos, transform, saw_player, heard_player, mut kool_aid) in
        mobs.iter_mut()
    {
        mob.move_timer.tick(time.delta());
        if mob.move_timer.finished() {
            let last_known_player_pos = saw_player
                .map(|saw| saw.pos)
                .or(heard_player.map(|heard| heard.pos));
            let mut target_pos = match mob.kind {
                MobKind::Hider => last_known_player_pos
                    .filter(|p| {
                        p.distance_squared(mob_pos.0) <= HIDER_CHASE_DISTANCE * HIDER_CHASE_DISTANCE
                    })
                    .or_else(|| find_hiding_spot(mob_pos.0, &walk_blocked_map, &sight_blocked_map)),
                _ => last_known_player_pos,
            };
            if let Some(kool_aid) = kool_aid.as_deref_mut() {
                target_pos = match kool_aid {
                    KoolAidMovement::Moving(path) => {
                        if path.is_empty() {
                            *kool_aid = KoolAidMovement::Resting(Timer::new(
                                Duration::from_secs(1),
                                TimerMode::Once,
                            ));
                            None
                        } else {
                            path.first().copied()
                        }
                    }
                    KoolAidMovement::Resting(timer) => {
                        timer.tick(time.delta());
                        if let Some(pos) = target_pos {
                            if timer.finished() {
                                let mut path: Vec<IVec2> =
                                    WalkGrid::new(mob_pos.0.into(), pos.into())
                                        .map(From::from)
                                        .collect();
                                // overshoot a bit
                                let overshot = pos + (pos - mob_pos.0);
                                path.extend(
                                    WalkGrid::new(pos.into(), overshot.into())
                                        .map(IVec2::from)
                                        .take(KOOL_AID_OVERSHOOT),
                                );
                                *kool_aid = KoolAidMovement::Moving(path);
                            }
                            None
                        } else {
                            None
                        }
                    }
                }
            }
            if let Some(target_pos) = target_pos {
                let avoid_player_sight = matches!(mob.kind, MobKind::Sculpture);
                let avoid_player_flashlight = true;
                let bust_through_walls = kool_aid.is_some();
                let move_pos = if bust_through_walls {
                    Some(target_pos)
                } else {
                    path_to(
                        mob_pos.0,
                        target_pos,
                        &walk_blocked_map,
                        &fov_map,
                        &flashlight_map,
                        avoid_player_sight,
                        avoid_player_flashlight,
                        false,
                    )
                    .and_then(|path| path.get(1).copied())
                };
                if let Some(move_pos) = move_pos {
                    if move_pos != player_pos.0 {
                        if !(matches!(mob.kind, MobKind::Sculpture)
                            && (fov_map.0.contains(&move_pos) || fov_map.0.contains(&mob_pos.0)))
                        {
                            walk_blocked_map.0.insert(move_pos);
                            mob_pos.0 = move_pos;
                            commands.entity(entity).insert(MoveAnimation {
                                from: transform.translation.truncate(),
                                to: mob_pos.to_vec2(),
                                timer: Timer::new(mob.kind.get_move_delay() / 2, TimerMode::Once),
                                ease: mob.kind.get_ease_function_for_movement(),
                            });
                            if bust_through_walls {
                                ev_bust.send(BustThroughWallEvent(mob_pos.0));
                            }
                            if let Some(KoolAidMovement::Moving(path)) = kool_aid.as_deref_mut() {
                                if !path.is_empty() {
                                    path.remove(0);
                                }
                            }
                            mob.move_timer.reset();
                        }
                    } else if !(matches!(mob.kind, MobKind::Sculpture)
                        && fov_map.0.contains(&mob_pos.0))
                    {
                        info!("{:?} damaging player", mob.kind);
                        ev_player_damage.send(PlayerDamageEvent { damage: 1 });
                        mob.move_timer.reset();
                    }
                }
            }
        }
    }
}

fn spawn_kool_aid_man(
    mut ev_shoot: EventReader<ShootEvent>,
    mut ev_spawn: EventWriter<SpawnEvent>,
    mut spawned: Local<bool>,
    player_visibility_map: Res<PlayerVisibilityMap>,
    zones: Res<Zones>,
) {
    for shoot_event in ev_shoot.read() {
        if !*spawned {
            if let Some(rect) = zones.0.get(2) {
                let map_pos = MapPos::from_vec2(shoot_event.start);
                if rect.contains(map_pos.0) {
                    let mut rng = rand::thread_rng();
                    let spawn_rect = rogue_algebra::Rect::new_centered(map_pos.0.into(), 10, 10);
                    let legal = spawn_rect
                        .into_iter()
                        .map(IVec2::from)
                        .filter(|pos| !player_visibility_map.0.contains(pos))
                        .collect::<Vec<_>>();
                    if let Some(spawn_pos) = legal.choose(&mut rng) {
                        ev_spawn.send(SpawnEvent(*spawn_pos, Spawn::Mob(MobKind::KoolAidMan)));
                        *spawned = true;
                    }
                }
            }
        }
    }
}

#[derive(Event)]
struct BustThroughWallEvent(IVec2);

fn bust_through_walls(
    mut commands: Commands,
    mut ev_bust: EventReader<BustThroughWallEvent>,
    map: Res<Map>,
    query: Query<Entity, With<Tile>>,
) {
    for BustThroughWallEvent(pos) in ev_bust.read() {
        for entity in query.iter_many(map.get(*pos)) {
            commands.entity(entity).despawn();
        }
    }
}

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_kool_aid_man,
                update_mobs_seeing_player,
                update_hearing_player,
                forget_player,
                damage_mobs,
                move_mobs,
                bust_through_walls,
            )
                .chain()
                .after(update_visibility)
                .after(update_walkability)
                .after(update_fov_map)
                .after(update_flashlight_map),
        )
        .add_event::<MobDamageEvent>()
        .add_event::<BustThroughWallEvent>();
    }
}
