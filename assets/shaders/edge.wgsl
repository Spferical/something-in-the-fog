#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::{
    view_transformations::frag_coord_to_uv
}

@group(2) @binding(0) var screen_texture: texture_2d<f32>;
@group(2) @binding(1) var screen_texture_sampler: sampler;

fn load_texture(position: vec2i) -> f32 {
    return textureLoad(screen_texture, position, 0).a;
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4<f32> {
    let screen_size = vec2f(textureDimensions(screen_texture));

    let scale_factor = screen_size / view_bindings::view.viewport.zw;
    var position_scaled = vec2i(position.xy * scale_factor);
    
    let center_x = i32(position_scaled.x);
    let center_y = i32(position_scaled.y);

    var count: i32 = 0;
    var center_filled: bool = sign(load_texture(vec2i(center_x, center_y)) - 0.05) == 1;
    for (var i = center_x - 1; i <= center_x + 1; i++) {
        for (var j = center_y - 1; j <= center_y + 1; j++) {
            var val = i32(sign(load_texture(vec2i(i, j)) - 0.05));
            count += val;
        }
    }

    if (count > 0 && count < 9 && center_filled) {
        return vec4(1.0);
    } else {
        return vec4(0.0);
    }
}
