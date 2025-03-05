use bevy::{
    asset::{Assets, Handle, RenderAssetUsages},
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res, ResMut, Single},
    },
    image::Image,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
    },
    utils::default,
    window::Window,
};

// use super::{AllSdfSettings, SdfSettings, SdfTexture};
use super::SdfTexture;

const SDF_TEXTURE: &str = "sdf_texture";

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
        asset_usage: RenderAssetUsages::RENDER_WORLD,
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
    println!("Prepared sdf texture!");
    commands.spawn(SdfTexture {
        ping: images.add(create_sdf_texture(&window, "sdf_texture_0")),
        pong: images.add(create_sdf_texture(&window, "sdf_texture_1")),
    });
}

/*const SDF_TEXTURE: &str = "sdf_texture";

pub fn create_single_sdf_texture(
    render_device: &Res<RenderDevice>,
    texture_cache: &mut ResMut<TextureCache>,
    view_target: &ViewTarget,
) -> CachedTexture {
    texture_cache.get(
        &render_device,
        TextureDescriptor {
            label: Some(SDF_TEXTURE),
            size: view_target.main_texture().size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
    )
}

pub fn prepare_sdf_texture(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    view_targets: Query<(Entity, &ViewTarget)>,
) {
    for (entity, view_target) in &view_targets {
        let ping = create_single_sdf_texture(&render_device, &mut texture_cache, view_target);
        let pong = create_single_sdf_texture(&render_device, &mut texture_cache, view_target);

        commands.entity(entity).insert(SdfTexture { ping, pong });
    }
}

pub fn prepare_sdf_settings(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut all_sdf_settings: ResMut<AllSdfSettings>,
) {
    all_sdf_settings
        .all
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| x.set(SdfSettings::new(i as i32)));

    all_sdf_settings
        .all
        .iter_mut()
        .for_each(|x| x.write_buffer(&render_device, &render_queue));
}*/
