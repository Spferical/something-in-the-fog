use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res, ResMut},
    },
    render::{
        render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
    },
};

use super::{AllSdfSettings, SdfSettings, SdfTexture};

const SDF_TEXTURE: &str = "sdf_texture";

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
        // commands.entity(entity).insert(AllSdfSettings::default());
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
}
