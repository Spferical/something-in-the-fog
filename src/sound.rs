use bevy::prelude::*;

use crate::assets::GameAssets;

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
        AudioPlayer(game_assets.sfx.base_track.clone().into()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::ZERO,
            ..default()
        },
        BaseTrack,
        FadeIn,
    ));

    commands.spawn((
        AudioPlayer(game_assets.sfx.active_track.clone().into()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::ZERO,
            ..default()
        },
        ActiveTrack,
    ));

    commands.spawn((
        AudioPlayer(game_assets.sfx.monk_track.clone().into()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::ZERO,
            ..default()
        },
        MonkTrack,
    ));
}

fn fade_in(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeIn>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() + time.delta_secs() / FADE_TIME);
        if audio.volume() >= 1.0 {
            audio.set_volume(1.0);
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

fn fade_out(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeOut>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() - time.delta_secs() / FADE_TIME);
        if audio.volume() <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_background_music)
            .add_systems(Update, (fade_in, fade_out));
    }
}
