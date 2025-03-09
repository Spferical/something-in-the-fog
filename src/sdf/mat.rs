use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::Material2d,
};

#[derive(Clone, Default, ShaderType)]
pub struct SdfSettings {
    pub iteration: i32,
    pub probe_size: i32,
    pub _padding: Vec2,
}

#[derive(AsBindGroup, Clone, Default, Asset, TypePath)]
pub struct SdfMaterial {
    #[texture(0)]
    pub screen_texture: Option<Handle<Image>>,
    #[texture(1)]
    pub edge_texture: Option<Handle<Image>>,
    #[texture(2)]
    pub seed_texture: Option<Handle<Image>>,
    #[uniform(3)]
    pub settings: SdfSettings,
}

impl Material2d for SdfMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf.wgsl".into()
    }
}
