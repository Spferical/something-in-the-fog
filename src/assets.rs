use bevy::prelude::*;

use crate::map::TILE_SIZE;

#[derive(Resource)]
pub struct GameAssets {
    pub circle: Handle<Mesh>,
    pub square: Handle<Mesh>,
    pub pixel: Handle<Mesh>,
    pub white: Handle<ColorMaterial>,
    pub green: Handle<ColorMaterial>,
    pub dark_green: Handle<ColorMaterial>,
    pub red: Handle<ColorMaterial>,
    pub purple: Handle<ColorMaterial>,
    pub sight_line: Handle<ColorMaterial>,
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(GameAssets {
        square: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
        circle: meshes.add(Circle::new(TILE_SIZE / 2.0)),
        pixel: meshes.add(Rectangle::new(1.0, 1.0)),
        white: materials.add(Color::LinearRgba(LinearRgba::WHITE)),
        green: materials.add(Color::LinearRgba(LinearRgba::GREEN)),
        dark_green: materials.add(Color::LinearRgba(LinearRgba::rgb(0.0, 0.5, 0.0))),
        red: materials.add(Color::LinearRgba(LinearRgba::RED)),
        purple: materials.add(Color::LinearRgba(LinearRgba::rgb(1.0, 0.0, 1.0))),
        sight_line: materials.add(Color::Srgba(
            bevy::color::palettes::basic::YELLOW.with_alpha(0.5),
        )),
    });
}

#[derive(Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_assets);
    }
}
