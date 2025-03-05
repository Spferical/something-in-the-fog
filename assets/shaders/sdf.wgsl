// #import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
//#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var screen_texture: texture_2d<f32>;
@group(2) @binding(1) var edge_texture: texture_2d<f32>;
@group(2) @binding(2) var seed_texture: texture_2d<f32>;
@group(2) @binding(3) var<uniform> iteration: i32;
@group(2) @binding(4) var<uniform> probe_size: i32;
@group(2) @binding(5) var<uniform> screen_size: vec2i;

fn query_seeds(position: vec2i) -> vec2f {
    if (iteration == 0) {
        if (textureLoad(edge_texture, vec2i(position.xy), 0).r > 0.5) {
            return vec2f(position);
        } else {
            return vec2f(-1e4, -1e4);
        }
    } else {
        return textureLoad(seed_texture, vec2i(position.xy), 0).xy *
            vec2f(screen_size);
    }
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4<f32> {
    let pos = vec2i(position.xy);

    var nearest_seed = vec2f(-screen_size);
    var nearest_dist: f32 = 999999.9;

    for (var i_r: i32 = -1; i_r < 2; i_r++) {
        for (var j_r: i32 = -1; j_r < 2; j_r++) {
            let i = i_r * probe_size;
            let j = j_r * probe_size;

            if (
                pos.x + i >= screen_size.x ||
                pos.x + i < 0 ||
                pos.y + j >= screen_size.y ||
                pos.y + j < 0
            ) {
                continue;
            }

            let s_q = query_seeds(pos + vec2i(i, j));
            if (s_q.x >= 0 && s_q.y >= 0) {
                let dist_s_q = length(position.xy - s_q);
                if (dist_s_q < nearest_dist) {
                    nearest_dist = dist_s_q;
                    nearest_seed = s_q;
                }
            }
        }
    }

    if (probe_size == 1) {
        return textureLoad(screen_texture, vec2i(position.xy), 0);
        // Uncomment me to get sdf vis!
        // return vec4(nearest_dist / vec2f(screen_size), 0.0, 1.0);
    } else {
        return vec4(nearest_seed / vec2f(screen_size), 0.0, 1.0);
    }
}
