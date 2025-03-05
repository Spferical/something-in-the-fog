// #import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var screen_texture: texture_2d<f32>;
@group(2) @binding(1) var screen_texture_sampler: sampler;
@group(2) @binding(2) var sdf: texture_2d<f32>;
@group(2) @binding(3) var sdf_sampler: sampler;
@group(2) @binding(4) var<uniform> iteration: i32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0);
    // return vec4(textureSample(screen_texture, screen_texture_sampler, in.uv));
    /*if (iteration == 0) {
        return vec4(0.5);
    } else {
        return vec4(textureSample(sdf, sdf_sampler, in.uv));
    }*/
}