use std::time::Duration;

use bevy::prelude::*;

use crate::{
    Player,
    animation::MoveAnimation,
    map::{
        MapPos, PlayerVisibilityMap, SightBlockedMap, WalkBlockedMap, path, update_visibility,
        update_walkability,
    },
};

const MAX_PATH: i32 = 100;
const HIDER_CHASE_DISTANCE: i32 = 5;

#[derive(Clone, Copy)]
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
            Hider => 3,
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

fn path_to(
    source: IVec2,
    target: IVec2,
    walk_blocked_map: &WalkBlockedMap,
    player_visibility_map: &PlayerVisibilityMap,
    avoid_player_sight: bool,
) -> Option<Vec<IVec2>> {
    path(
        source,
        target,
        MAX_PATH,
        |p| walk_blocked_map.0.contains(&p),
        |p| {
            if avoid_player_sight && player_visibility_map.0.contains(&p) {
                99
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
pub struct SawPlayer(pub IVec2);

fn update_mobs_seeing_player(
    mut commands: Commands,
    mobs: Query<(Entity, &MapPos, Option<&SawPlayer>), With<SeesPlayer>>,
    player_visibility_map: Res<PlayerVisibilityMap>,
    player: Query<&MapPos, (With<Player>, Without<Mob>)>,
) {
    let player_pos = player.single();
    for (entity, pos, saw_player) in mobs.iter() {
        if player_visibility_map.0.contains(&pos.0) {
            commands.entity(entity).insert(SawPlayer(player_pos.0));
        } else if saw_player.is_some_and(|p| p.0 == player_pos.0) {
            // Mob is standing on last seen player position.
            commands.entity(entity).remove::<SawPlayer>();
        }
    }
}

fn move_mobs(
    mut commands: Commands,
    mut mobs: Query<(
        Entity,
        &mut Mob,
        &mut MapPos,
        &mut Transform,
        Option<&SawPlayer>,
    )>,
    player: Query<&MapPos, (With<Player>, Without<Mob>)>,
    mut walk_blocked_map: ResMut<WalkBlockedMap>,
    sight_blocked_map: Res<SightBlockedMap>,
    player_visibility_map: Res<PlayerVisibilityMap>,
    time: Res<Time>,
) {
    let player_pos = player.single();
    for (entity, mut mob, mut pos, transform, saw_player) in mobs.iter_mut() {
        mob.move_timer.tick(time.delta());
        if mob.move_timer.finished() {
            let last_seen_player_pos = saw_player.map(|saw| saw.0);
            let target_pos = match mob.kind {
                MobKind::Hider => last_seen_player_pos
                    .filter(|p| {
                        p.distance_squared(pos.0) <= HIDER_CHASE_DISTANCE * HIDER_CHASE_DISTANCE
                    })
                    .or_else(|| find_hiding_spot(pos.0, &walk_blocked_map, &sight_blocked_map)),
                _ => last_seen_player_pos,
            };
            if let Some(target_pos) = target_pos {
                let avoid_player_sight = matches!(mob.kind, MobKind::Sculpture);
                if let Some(path) = path_to(
                    pos.0,
                    target_pos,
                    &walk_blocked_map,
                    &player_visibility_map,
                    avoid_player_sight,
                ) {
                    if let Some(move_pos) = path.get(1) {
                        if *move_pos != player_pos.0 {
                            if !(matches!(mob.kind, MobKind::Sculpture)
                                && player_visibility_map.0.contains(move_pos))
                            {
                                walk_blocked_map.0.insert(*move_pos);
                                pos.0 = *move_pos;
                                commands.entity(entity).insert(MoveAnimation {
                                    from: transform.translation.truncate(),
                                    to: pos.to_vec2(),
                                    timer: Timer::new(
                                        mob.kind.get_move_delay() / 2,
                                        TimerMode::Once,
                                    ),
                                    ease: mob.kind.get_ease_function_for_movement(),
                                });
                                mob.move_timer.reset();
                            }
                        } else {
                            // TODO: monster found the player by pathing through him
                        }
                    }
                }
            }
        }
    }
}

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_mobs_seeing_player, damage_mobs, move_mobs)
                .chain()
                .after(update_visibility)
                .after(update_walkability),
        );
        app.add_event::<MobDamageEvent>();
    }
}
