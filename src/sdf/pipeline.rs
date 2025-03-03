use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::{sampler, uniform_buffer};
use bevy::render::render_resource::{
    binding_types::texture_2d, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
    RenderPipelineDescriptor, ShaderStages, TextureFormat,
};
use bevy::render::render_resource::{
    Sampler, SamplerBindingType, SamplerDescriptor, TextureSampleType,
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

impl FromWorld for SdfPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let layout = render_device.create_bind_group_layout(
            SDF_BIND_GROUP_LAYOUT,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<ViewUniform>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let shader = world.load_asset(SDF_SHADER_ASSET_PATH);

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
