use bevy::prelude::*;

use crate::{
    assets::GameAssets,
    map::{Map, MapPos},
    mob::{HeardPlayer, Mob, MobKind, SawPlayer},
    player::Player,
};

#[derive(Component)]
struct BaseTrack;

#[derive(Component)]
struct ActiveTrack;

#[derive(Component)]
struct MonkTrack;

#[derive(Component)]
struct BossTrack;

#[derive(Component)]
struct RadioStaticTrack;

#[derive(Component)]
struct FadeIn {
    max_volume: f32,
}

#[derive(Component)]
struct FadeOut;

const FADE_IN_TIME: f32 = 2.0;
const FADE_OUT_TIME: f32 = 10.0;

impl Default for FadeIn {
    fn default() -> FadeIn {
        FadeIn { max_volume: 1.0 }
    }
}

pub fn setup_background_music(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        AudioPlayer(game_assets.sfx.base_track.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::ZERO,
            ..default()
        },
        BaseTrack,
        FadeIn::default(),
    ));

    commands.spawn((
        AudioPlayer(game_assets.sfx.active_track.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::ZERO,
            ..default()
        },
        ActiveTrack,
    ));

    commands.spawn((
        AudioPlayer(game_assets.sfx.monk_track.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::ZERO,
            ..default()
        },
        MonkTrack,
    ));

    commands.spawn((
        AudioPlayer(game_assets.sfx.boss_track.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.0),
            ..default()
        },
        BossTrack,
    ));

    commands.spawn((
        AudioPlayer(game_assets.sfx.radio_static_track.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.0),
            ..default()
        },
        RadioStaticTrack,
    ));
}

fn adjust_radio_static(
    mut commands: Commands,
    player: Query<&MapPos, (With<Player>, Without<Mob>)>,
    heard_mobs: Query<(Entity, &MapPos, &Mob), With<crate::mob::HearsPlayer>>,
    seen_mobs: Query<(Entity, &MapPos, &Mob), With<crate::mob::SeesPlayer>>,
    query_radio_track: Query<Entity, With<RadioStaticTrack>>,
    map: Res<Map>,
) {
    let Ok(player_pos) = player.get_single() else {
        return;
    };

    let Ok(radio_track) = query_radio_track.get_single() else {
        return;
    };

    const HEARING_RADIUS: i32 = 10;
    let mut closest_enemy_dist: f32 = 150.0;
    for (_, pos, _) in heard_mobs
        .iter_many(map.get_nearby(player_pos.0, HEARING_RADIUS))
        .filter(|(_, _, mob)| matches!(mob.kind, MobKind::Zombie))
    {
        closest_enemy_dist =
            closest_enemy_dist.min((player_pos.to_vec2() - pos.to_vec2()).length());
    }

    for (_, pos, _) in seen_mobs
        .iter_many(map.get_nearby(player_pos.0, HEARING_RADIUS))
        .filter(|(_, _, mob)| matches!(mob.kind, MobKind::Zombie))
    {
        closest_enemy_dist =
            closest_enemy_dist.min((player_pos.to_vec2() - pos.to_vec2()).length());
    }

    let volume = (150.0 - closest_enemy_dist) / 150.0;
    if volume <= 0.0 {
        commands
            .entity(radio_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
    } else {
        commands
            .entity(radio_track)
            .insert(FadeIn { max_volume: 0.6 })
            .remove::<FadeOut>();
    }
}

#[allow(clippy::type_complexity)]
fn update_mob_audio(
    mut commands: Commands,
    q_saw_player: Query<&Mob, Or<(With<SawPlayer>, With<HeardPlayer>)>>,
    query_base_track: Query<Entity, With<BaseTrack>>,
    query_active_track: Query<(Entity, Option<&FadeIn>, Option<&FadeOut>), With<ActiveTrack>>,
    query_monk_track: Query<(Entity, Option<&FadeIn>, Option<&FadeOut>), With<MonkTrack>>,
    query_boss_track: Query<(Entity, Option<&FadeIn>), With<BossTrack>>,
) {
    let Ok(base_track) = query_base_track.get_single() else {
        return;
    };
    let Ok((active_track, active_fading_in, active_fading_out)) = query_active_track.get_single()
    else {
        return;
    };
    let Ok((monk_track, monk_fading_in, monk_fading_out)) = query_monk_track.get_single() else {
        return;
    };
    let Ok((boss_track, boss_fading_in)) = query_boss_track.get_single() else {
        return;
    };

    let should_play_active = q_saw_player
        .iter()
        .any(|mob| mob.kind == MobKind::KoolAidMan);
    let should_play_monk = q_saw_player
        .iter()
        .any(|mob| (mob.kind == MobKind::Sculpture || mob.kind == MobKind::Ghost));
    let should_play_boss = q_saw_player
        .iter()
        .any(|mob| (mob.kind == MobKind::Eyeball));

    if should_play_active && active_fading_in.is_none() && !should_play_boss {
        commands
            .entity(active_track)
            .insert(FadeIn::default())
            .remove::<FadeOut>();
    }
    if !should_play_active && active_fading_out.is_none() {
        commands
            .entity(active_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
    }
    if should_play_monk && monk_fading_in.is_none() && !should_play_boss {
        commands
            .entity(monk_track)
            .insert(FadeIn::default())
            .remove::<FadeOut>();
    }
    if !should_play_monk && monk_fading_out.is_none() {
        commands
            .entity(monk_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
    }
    if should_play_boss && boss_fading_in.is_none() {
        println!("senpai noticed me!");
        commands
            .entity(boss_track)
            .insert(FadeIn::default())
            .remove::<FadeOut>();
        commands
            .entity(base_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
        commands
            .entity(active_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
        commands
            .entity(monk_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
    }
}

fn fade_in(mut audio_sink: Query<(&mut AudioSink, &FadeIn)>, time: Res<Time>) {
    for (audio, fade_in) in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() + time.delta_secs() / FADE_IN_TIME);
        if audio.volume() >= fade_in.max_volume {
            audio.set_volume(1.0);
        }
    }
}

fn fade_out(mut audio_sink: Query<&mut AudioSink, With<FadeOut>>, time: Res<Time>) {
    for audio in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() - time.delta_secs() / FADE_OUT_TIME);
        if audio.volume() <= 0.0 {
            audio.set_volume(0.0);
        }
    }
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_background_music)
            .add_systems(
                Update,
                (fade_in, fade_out, update_mob_audio, adjust_radio_static),
            );
    }
}
