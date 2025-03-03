use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::{sampler, uniform_buffer};
use bevy::render::render_resource::{
    binding_types::texture_2d, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
    RenderPipelineDescriptor, ShaderStages, ShaderType, TextureFormat,
};
use bevy::render::render_resource::{
    Sampler, SamplerBindingType, SamplerDescriptor, TextureSampleType, UniformBuffer,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ViewUniform;

const SDF_SHADER_ASSET_PATH: &str = "shaders/sdf.wgsl";
const SDF_PIPELINE: &str = "sdf_pipeline";
const SDF_BIND_GROUP_LAYOUT: &str = "sdf_bind_group_layout";

#[derive(Resource)]
pub struct SdfPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

#[derive(Resource, ShaderType)]
pub struct SdfSettings {
    pub iteration: i32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}

impl SdfSettings {
    pub fn new(iteration: i32) -> Self {
        Self {
            iteration,
            #[cfg(feature = "webgl2")]
            _webgl2_padding: Vec3::ZERO,
        }
    }
}

impl Default for SdfSettings {
    fn default() -> SdfSettings {
        SdfSettings::new(0)
    }
}

#[derive(Resource)]
pub struct AllSdfSettings {
    pub all: Vec<UniformBuffer<SdfSettings>>,
}

impl Default for AllSdfSettings {
    fn default() -> AllSdfSettings {
        let mut settings = Vec::<UniformBuffer<SdfSettings>>::new();
        for i in 0..16 {
            settings.push(UniformBuffer::from(SdfSettings::new(i as i32)));
        }

        AllSdfSettings { all: settings }
    }
}

impl FromWorld for SdfPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            SDF_BIND_GROUP_LAYOUT,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<SdfSettings>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let shader = world.load_asset(SDF_SHADER_ASSET_PATH);

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(SDF_PIPELINE.into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
