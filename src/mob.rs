use std::time::Duration;

use bevy::prelude::*;
use line_drawing::Bresenham;

use crate::{
    Player,
    animation::MoveAnimation,
    map::{MapPos, SightBlockedMap, WalkBlockedMap},
};
#[derive(Clone, Copy)]
pub enum MobKind {
    Zombie,
    Sculpture,
    Hider,
}

impl MobKind {
    pub fn get_move_delay(&self) -> Duration {
        use MobKind::*;
        match self {
            Zombie => Duration::from_secs(1),
            Sculpture => Duration::from_millis(16),
            Hider => Duration::from_millis(200),
        }
    }

    pub fn max_damage(&self) -> i32 {
        use MobKind::*;
        match self {
            Zombie => 3,
            Sculpture => 99,
            Hider => 3,
        }
    }

    pub fn get_ease_function_for_movement(&self) -> EaseFunction {
        use MobKind::*;
        match self {
            Zombie => EaseFunction::BounceIn,
            Sculpture => EaseFunction::Linear,
            Hider => EaseFunction::CubicIn,
        }
    }
}

#[derive(Component)]
pub struct Mob {
    pub kind: MobKind,
    pub saw_player_at: Option<IVec2>,
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
                let path = walk_blocked_map.path(pos.0, player_pos, 100);
                if let Some(path) = path {
                    if let Some(move_pos) = path.get(1) {
                        if *move_pos != player_pos {
                            walk_blocked_map.0.insert(*move_pos);
                            pos.0 = *move_pos;
                            commands.entity(entity).insert(MoveAnimation {
                                from: transform.translation.truncate(),
                                to: pos.to_vec2(),
                                timer: Timer::new(mob.kind.get_move_delay() / 2, TimerMode::Once),
                                ease: mob.kind.get_ease_function_for_movement(),
                            });
                            mob.move_timer.reset();
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
        app.add_systems(Update, (damage_mobs, move_mobs));
        app.add_event::<MobDamageEvent>();
    }
}
