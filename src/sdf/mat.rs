use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(AsBindGroup, Clone, Default, Asset, TypePath)]
pub struct SdfMaterial {
    #[texture(0)]
    pub screen_texture: Option<Handle<Image>>,
    #[texture(1)]
    pub edge_texture: Option<Handle<Image>>,
    #[texture(2)]
    pub seed_texture: Option<Handle<Image>>,
    #[uniform(3)]
    pub iteration: i32,
    #[uniform(4)]
    pub probe_size: i32,
    #[uniform(5)]
    pub screen_size: IVec2,
}

impl Material2d for SdfMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf.wgsl".into()
    }
}
