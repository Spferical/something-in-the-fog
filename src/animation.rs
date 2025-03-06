use bevy::prelude::*;

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

pub struct AnimatePlugin;

impl Plugin for AnimatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate);
    }
}
