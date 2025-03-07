use bevy::prelude::*;

#[derive(Component)]
pub struct DespawnAfter(pub Timer);

fn despawn_after(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnAfter)>,
    time: Res<Time>,
) {
    for (entity, mut clear_after) in query.iter_mut() {
        clear_after.0.tick(time.delta());
        if clear_after.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct DespawnAfterPlugin;
impl Plugin for DespawnAfterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_after);
    }
}
