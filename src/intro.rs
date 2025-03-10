use rand::prelude::*;
use std::time::Duration;

use bevy::prelude::*;

use crate::{
    animation::TextEvent,
    map::{self, MapPos},
    player::Player,
};

pub struct IntroPlugin;

const INTRO_TEXT: [&'static str; 9] = [
    "I left this town\nlong ago",
    "I was cold\nand hungry,\nfilled with regret",
    "There is nothing\nleft here for you",
    "Go back to\nyour car",
    "start the engine",
    "and drive as far\nfrom here as you can",
    "There is something\nin these woods",
    "It knows you\nare here",
    "Something in the fog",
];

#[derive(Component)]
pub struct IntroText(f32);

fn intro_system_update(
    mut ev_text: EventWriter<TextEvent>,
    player_query: Query<(&Transform, &map::MapPos), With<Player>>,
    mut intro_text: Query<&mut IntroText>,
) {
    let mut rng = rand::thread_rng();

    let Ok((player, player_pos)) = player_query.get_single() else {
        return;
    };
    let Ok(mut intro_text) = intro_text.get_single_mut() else {
        return;
    };

    let intro_text_x = (intro_text.0 / 300.0) as usize;
    let player_x = (player.translation.x / 300.0) as usize;

    if intro_text_x < player_x && player_x <= INTRO_TEXT.len() {
        ev_text.send(TextEvent {
            text: INTRO_TEXT[player_x - 1].into(),
            position: MapPos(player_pos.0 + IVec2::new(rng.gen_range(-5..5), rng.gen_range(-5..5)))
                .to_vec2(),
            duration: Duration::from_secs(10),
            teletype: Duration::from_secs(3.5),
            font_size: 15.0,
            ..default()
        });

        intro_text.0 = player.translation.x;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(IntroText(0.0));
}

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, intro_system_update);
    }
}
