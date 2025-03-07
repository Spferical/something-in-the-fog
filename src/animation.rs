use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{assets::GameAssets, despawn_after::DespawnAfter, map::TILE_SIZE};

#[derive(Component)]
pub struct MoveAnimation {
    pub from: Vec2,
    pub to: Vec2,
    pub timer: Timer,
    pub ease: EaseFunction,
}

fn animate(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut MoveAnimation)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut animation) in query.iter_mut() {
        animation.timer.tick(time.delta());
        let Vec2 { x, y } = EasingCurve::new(animation.from, animation.to, animation.ease)
            .sample_clamped(animation.timer.fraction());
        transform.translation.x = x;
        transform.translation.y = y;
        if animation.timer.finished() {
            commands.entity(entity).remove::<MoveAnimation>();
        }
    }
}

#[derive(Event)]
pub struct TextEvent {
    pub text: String,
    pub position: Vec2,
}

fn spawn_text(
    mut commands: Commands,
    mut ev_text: EventReader<TextEvent>,
    assets: Res<GameAssets>,
) {
    for TextEvent { text, position } in ev_text.read() {
        let timer = Timer::new(Duration::from_secs(1), TimerMode::Once);
        commands.spawn((
            Transform::from_translation(position.extend(3.0)),
            Text2d::new(text),
            TextFont::from_font(assets.font.clone()),
            TextFade(timer.clone()),
            DespawnAfter(timer.clone()),
            RenderLayers::layer(1),
            MoveAnimation {
                from: *position,
                to: position + Vec2::new(0.0, TILE_SIZE),
                timer,
                ease: EaseFunction::QuadraticIn,
            },
        ));
    }
}

#[derive(Component)]
pub struct TextFade(Timer);

fn fade_text(mut query: Query<(&mut TextColor, &mut TextFade)>, time: Res<Time>) {
    for (mut color, mut fade) in query.iter_mut() {
        fade.0.tick(time.delta());
        color.0 = Color::Srgba(color.0.to_srgba().with_alpha(fade.0.fraction_remaining()));
    }
}

pub struct AnimatePlugin;

impl Plugin for AnimatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (animate, spawn_text, fade_text))
            .add_event::<TextEvent>();
    }
}
