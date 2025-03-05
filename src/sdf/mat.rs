use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(AsBindGroup, Clone, Default, Asset, TypePath)]
pub struct SdfMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub screen_texture: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    pub sdf: Option<Handle<Image>>,
    #[uniform(4)]
    pub iteration: i32,
}

impl Material2d for SdfMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf.wgsl".into()
    }
}
