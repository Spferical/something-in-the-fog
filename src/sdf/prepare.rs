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

use super::SdfTexture;

pub fn on_resize_sdf_texture(
    mut resize_reader: EventReader<bevy::window::WindowResized>,
    mut sdf_texture_query: Query<&mut SdfTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(mut sdf_texture) = sdf_texture_query.get_single_mut() else {
        return;
    };
    for e in resize_reader.read() {
        sdf_texture.iters.iter_mut().for_each(|x| {
            if let Some(image) = images.get_mut(x) {
                image.resize(Extent3d {
                    width: e.width as u32,
                    height: e.height as u32,
                    ..default()
                });
            }
        });
    }
}

pub fn create_sdf_texture(window: &Single<&Window>, name: &'static str) -> Image {
    let target_size = Extent3d {
        width: window.resolution.physical_width() / 1,
        height: window.resolution.physical_height() / 1,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some(name),
            size: target_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
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
