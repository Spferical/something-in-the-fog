use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{Z_TEXT, assets::GameAssets, despawn_after::DespawnAfter, map::TILE_HEIGHT};

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
    pub duration: Duration,
}

fn spawn_text(
    mut commands: Commands,
    mut ev_text: EventReader<TextEvent>,
    assets: Res<GameAssets>,
) {
    for TextEvent {
        text,
        position,
        duration,
    } in ev_text.read()
    {
        let timer = Timer::new(*duration, TimerMode::Once);
        commands.spawn((
            Transform::from_translation(position.extend(Z_TEXT)),
            Text2d::new(text),
            TextFont::from_font(assets.font.clone()),
            TextFade(timer.clone()),
            DespawnAfter(timer.clone()),
            RenderLayers::layer(1),
            MoveAnimation {
                from: *position,
                to: position + Vec2::new(0.0, TILE_HEIGHT),
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

#[derive(Component)]
pub struct FadeColorMaterial {
    pub timer: Timer,
    pub ease: EasingCurve<f32>,
}

fn fade_color_material(
    mut query: Query<(&MeshMaterial2d<ColorMaterial>, &mut FadeColorMaterial)>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (material, mut fade) in query.iter_mut() {
        fade.timer.tick(time.delta());
        let color = &mut materials.get_mut(material.0.id()).unwrap().color;
        *color = color.with_alpha(fade.ease.sample_clamped(fade.timer.fraction()));
    }
}

pub struct AnimatePlugin;

impl Plugin for AnimatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate, spawn_text, fade_text, fade_color_material),
        )
        .add_event::<TextEvent>();
    }
}
