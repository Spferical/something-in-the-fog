use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(AsBindGroup, Clone, Default, Asset, TypePath)]
pub struct EdgeMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub screen_texture: Option<Handle<Image>>,
}

impl Material2d for EdgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/edge.wgsl".into()
    }
}
