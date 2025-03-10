use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    assets::GameAssets, despawn_after::DespawnAfter, lighting::UI_LAYER, map::TILE_HEIGHT,
    player::GunInfo, FadeOutEndScreen, Z_TEXT,
};

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
    pub teletype: Duration,
    pub movement: bool,
    pub font_size: f32,
}

impl Default for TextEvent {
    fn default() -> TextEvent {
        TextEvent {
            text: "".into(),
            position: Vec2::ZERO,
            duration: Duration::ZERO,
            teletype: Duration::ZERO,
            movement: true,
            font_size: 10.0,
        }
    }
}

#[derive(Component)]
pub struct TeleType {
    timer: Timer,
    text: String,
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
        teletype,
        movement,
        font_size,
    } in ev_text.read()
    {
        let timer = Timer::new(*duration, TimerMode::Once);
        let initial_text = if teletype.as_millis() > 0 {
            String::new()
        } else {
            text.clone()
        };
        let mut entity = commands.spawn((
            Transform::from_translation(position.extend(Z_TEXT)),
            Text2d::new(initial_text),
            TextLayout::new(JustifyText::Center, LineBreak::WordBoundary),
            TextFont::from_font(assets.font.clone()).with_font_size(*font_size),
            TextFade(timer.clone()),
            DespawnAfter(timer.clone()),
            RenderLayers::layer(UI_LAYER),
        ));
        if teletype.as_millis() > 0 {
            entity.insert(TeleType {
                timer: Timer::new(*teletype, TimerMode::Once),
                text: text.clone(),
            });
        }

        if *movement {
            entity.insert(MoveAnimation {
                from: *position,
                to: position + Vec2::new(0.0, TILE_HEIGHT),
                timer: timer.clone(),
                ease: EaseFunction::QuadraticIn,
            });
        }
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

fn handle_fade_out(
    mut query: Query<(&mut FadeOutEndScreen, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let Ok((mut fade_out, material_handle)) = query.get_single_mut() else {
        return;
    };
    let material = materials.get_mut(material_handle).unwrap();

    fade_out.timer.tick(time.delta());
    material.base_color = Color::Srgba(
        fade_out
            .color
            .to_srgba()
            .with_alpha(fade_out.timer.fraction()),
    );
}

fn teletype_text(mut query: Query<(&mut Text2d, &mut TeleType)>, time: Res<Time>) {
    for (mut text, mut teletype) in query.iter_mut() {
        teletype.timer.tick(time.delta());
        let alpha = teletype.timer.fraction();
        let num_chars = (alpha * (teletype.text.len() as f32)) as usize;
        text.0 = teletype.text[0..num_chars].to_owned();
    }
}

#[derive(Clone)]
pub struct WobbleEffect {
    pub timer: Timer,
    pub ease: EasingCurve<f32>,
}

#[derive(Component, Default)]
pub struct WobbleEffects {
    pub effects: Vec<WobbleEffect>,
}

fn wobble_effect(mut mobs: Query<(&mut Transform, &mut WobbleEffects)>, time: Res<Time>) {
    for (mut transform, mut wobble) in mobs.iter_mut() {
        let mut total_rotation = 0.0;
        for effect in wobble.effects.iter_mut() {
            effect.timer.tick(time.delta());
            let t = effect
                .ease
                .sample_clamped((effect.timer.fraction() * 2.0 - 1.0).abs());
            total_rotation += t;
        }
        transform.rotation = Quat::from_rotation_z(total_rotation);
        wobble.effects.retain(|w| !w.timer.finished());
    }
}

#[derive(Component)]
pub struct MuzzleFlash {
    pub timer: Timer,
    pub ease: EasingCurve<f32>,
    pub info: GunInfo,
}

pub struct AnimatePlugin;

impl Plugin for AnimatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate,
                spawn_text,
                teletype_text,
                fade_text,
                handle_fade_out,
                wobble_effect,
            ),
        )
        .add_event::<TextEvent>();
    }
}
