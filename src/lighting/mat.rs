use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

#[derive(Clone, Copy, Default, ShaderType)]
pub struct Light {
    pub color: Vec4,
    pub intensity: f32,
    pub center: Vec4,
    pub direction: Vec4,
    pub focus: f32,
    pub attenuation: f32,
    pub _padding: f32
}

#[derive(Clone, Default, ShaderType)]
pub struct LightBundle {
    pub lights: [Light; 8],
}

#[derive(Clone, Default, ShaderType)]
pub struct LightingSettings {
    pub toggle_2d: i32,
    pub num_lights: i32,
    pub _padding: Vec2
}

#[derive(AsBindGroup, Clone, Default, Asset, TypePath)]
pub struct LightingMaterial {
    #[texture(0)]
    pub screen_texture: Option<Handle<Image>>,
    #[texture(1)]
    pub edge_texture: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    pub seed_texture: Option<Handle<Image>>,
    #[uniform(4)]
    pub lighting_settings: LightingSettings,
    #[uniform(5)]
    pub lights: LightBundle,
}

impl Material for LightingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lighting.wgsl".into()
    }
}
