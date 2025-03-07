use bevy::{
    asset::{Assets, RenderAssetUsages},
    ecs::{
        event::EventReader,
        system::{Commands, Query, ResMut, Single},
    },
    image::Image,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    utils::default,
    window::Window,
};

use super::EdgeTexture;

pub fn on_resize_edge_texture(
    mut resize_reader: EventReader<bevy::window::WindowResized>,
    mut edge_texture_query: Query<&mut EdgeTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(mut edge_texture) = edge_texture_query.get_single_mut() else {
        return;
    };
    for e in resize_reader.read() {
        if let Some(image) = images.get_mut(&mut edge_texture.0) {
            image.resize(Extent3d {
                width: e.width as u32,
                height: e.height as u32,
                ..default()
            });
        }
    }
}

pub fn prepare_edge_texture(
    mut commands: Commands,
    window: Single<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    let target_size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("edge_texture"),
            size: target_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        asset_usage: RenderAssetUsages::default(),
        ..default()
    };

    image.resize(target_size);
    commands.spawn(EdgeTexture(images.add(image)));
}
