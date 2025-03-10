#import bevy_pbr::mesh_view_bindings::globals

const COLOR: vec3<f32> = vec3<f32>(0.42, 0.40, 0.47);
const BG: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const ZOOM: f32 = 3.0;
const OCTAVES: i32 = 4;

// const INTENSITY: f32 = 130.0;

fn random(st: vec2<f32>) -> f32 {
    return fract(sin(dot(st, vec2<f32>(12.9818, 79.279))) * 43758.5453123);
}

fn random2(st: vec2<f32>) -> vec2<f32> {
    var st_tmp = vec2<f32>(dot(st, vec2<f32>(127.1, 311.7)), dot(st, vec2<f32>(269.5, 183.3)));
    return -1.0 + 2.0 * fract(sin(st_tmp) * 7.0);
}

fn noise(st: vec2<f32>) -> f32 {
    let i = floor(st);
    let f = fract(st);

    // smoothstep
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(
            dot(random2(i + vec2<f32>(0.0, 0.0)), f - vec2<f32>(0.0, 0.0)),
            dot(random2(i + vec2<f32>(1.0, 0.0)), f - vec2<f32>(1.0, 0.0)),
            u.x,
        ),
        mix(
            dot(random2(i + vec2<f32>(0.0, 1.0)), f - vec2<f32>(0.0, 1.0)),
            dot(random2(i + vec2<f32>(1.0, 1.0)), f - vec2<f32>(1.0, 1.0)),
            u.x,
        ),
        u.y,
    );
}

fn fbm(coord: vec2<f32>) -> f32 {
    var value = 0.0;
    var scale = 0.2;
    var coord_tmp = coord;
    for (var i = 0; i < OCTAVES; i++) {
        value += noise(coord_tmp) * scale;
        coord_tmp *= 2.0;
        scale *= 0.5;
    }
    return value + 0.2;
}

fn get_fog_density(uv: vec2<f32>) -> f32 {
    let motion = vec2f(globals.time * -0.1, 0.0);
    return fbm((uv + motion) * ZOOM);
}
