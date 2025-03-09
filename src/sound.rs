use bevy::prelude::*;

use crate::{
    assets::GameAssets,
    mob::{HeardPlayer, Mob, MobKind, SawPlayer},
};

#[derive(Component)]
struct BaseTrack;

#[derive(Component)]
struct ActiveTrack;

#[derive(Component)]
struct MonkTrack;

#[derive(Component)]
struct FadeIn;

#[derive(Component)]
struct FadeOut;

const FADE_TIME: f32 = 2.0;

pub fn setup_background_music(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        AudioPlayer(game_assets.sfx.base_track.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::ZERO,
            ..default()
        },
        BaseTrack,
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
}

#[allow(clippy::type_complexity)]
fn update_mob_audio(
    mut commands: Commands,
    q_saw_player: Query<&Mob, Or<(With<SawPlayer>, With<HeardPlayer>)>>,
    query_active_track: Query<(Entity, Option<&FadeIn>, Option<&FadeOut>), With<ActiveTrack>>,
    query_monk_track: Query<(Entity, Option<&FadeIn>, Option<&FadeOut>), With<MonkTrack>>,
) {
    let Ok((active_track, active_fading_in, active_fading_out)) = query_active_track.get_single()
    else {
        return;
    };
    let Ok((monk_track, monk_fading_in, monk_fading_out)) = query_monk_track.get_single() else {
        return;
    };

    let should_play_active = q_saw_player
        .iter()
        .any(|mob| mob.kind == MobKind::KoolAidMan);
    let should_play_monk = q_saw_player
        .iter()
        .any(|mob| mob.kind == MobKind::Sculpture);

    if should_play_active && active_fading_in.is_none() {
        commands
            .entity(active_track)
            .insert(FadeIn)
            .remove::<FadeOut>();
    }
    if !should_play_active && active_fading_out.is_none() {
        commands
            .entity(active_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
    }
    if should_play_monk && monk_fading_in.is_none() {
        commands
            .entity(monk_track)
            .insert(FadeIn)
            .remove::<FadeOut>();
    }
    if !should_play_monk && monk_fading_out.is_none() {
        commands
            .entity(monk_track)
            .insert(FadeOut)
            .remove::<FadeIn>();
    }
}

fn fade_in(mut audio_sink: Query<&mut AudioSink, With<FadeIn>>, time: Res<Time>) {
    for audio in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() + time.delta_secs() / FADE_TIME);
        if audio.volume() >= 1.0 {
            audio.set_volume(1.0);
        }
    }
}

fn fade_out(mut audio_sink: Query<&mut AudioSink, With<FadeOut>>, time: Res<Time>) {
    for audio in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() - time.delta_secs() / FADE_TIME);
        if audio.volume() <= 0.0 {
            audio.set_volume(0.0);
        }
    }
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_background_music)
            .add_systems(Update, (fade_in, fade_out, update_mob_audio));
    }
}
