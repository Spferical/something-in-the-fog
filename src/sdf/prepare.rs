use bevy::{
    asset::{Assets, RenderAssetUsages},
    ecs::system::{Commands, ResMut, Single},
    image::Image,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    utils::default,
    window::Window,
};

use super::SdfTexture;

pub fn create_sdf_texture(window: &Single<&Window>, name: &'static str) -> Image {
    let target_size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some(name),
            size: target_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        asset_usage: RenderAssetUsages::default(),
        ..default()
    };

    image.resize(target_size);
    image
}

pub fn prepare_sdf_texture(
    mut commands: Commands,
    window: Single<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(SdfTexture {
        iters: (0..32)
            .map(|_| images.add(create_sdf_texture(&window, "sdf_texture")))
            .collect(),
    });
}
