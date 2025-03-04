use bevy::{
    ecs::system::{Res, ResMut},
    render::{
        render_resource::{TextureDescriptor, TextureDimension, TextureFormat, TextureUsages},
        renderer::{RenderDevice},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
    },
};

const EDGE_TEXTURE: &str = "edge_texture";

pub fn prepare_edge_texture(
    render_device: &Res<RenderDevice>,
    texture_cache: &mut ResMut<TextureCache>,
    view_target: &ViewTarget,
) -> CachedTexture {
    texture_cache.get(
        &render_device,
        TextureDescriptor {
            label: Some(EDGE_TEXTURE),
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
