#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::{
    forward_io::VertexOutput,
    view_transformations,
}
#import bevy_pbr::utils::coords_to_viewport_uv
#import "shaders/types.wgsl"::{
    RayTraceOutputs, Light, LightBundle, LightingSettings
}
#import "shaders/fog.wgsl"::{
    get_fog_density, fbm
}

@group(2) @binding(0) var screen_texture: texture_2d<f32>;
@group(2) @binding(1) var ui_texture: texture_2d<f32>;
@group(2) @binding(2) var edge_texture: texture_2d<f32>;
@group(2) @binding(3) var seed_texture: texture_2d<f32>;
@group(2) @binding(4) var seed_sampler: sampler;
@group(2) @binding(5) var<uniform> settings: LightingSettings;
@group(2) @binding(6) var<uniform> lights: LightBundle;

fn sample_2d_seed(uv: vec2f) -> vec2f {
    let screen_size = vec2f(textureDimensions(seed_texture));

    return textureSample(
        seed_texture,
        seed_sampler,
        uv
    ).xy / screen_size;
}

fn sdf_2d(uv: vec3f) -> f32 {
    let seed = sample_2d_seed(uv.xy);
    let inside_texture = textureSample(screen_texture, seed_sampler, uv.xy).a > 0.1;
    let sdf = length(uv.xy - seed.xy) * select(1., -1., inside_texture);
    return sdf;
}

fn sdf_extruded(p_: vec3<f32>) -> f32 {
    let p = p_ - vec3f(0.0, 0.0, 0.0);
    let d = sdf_2d(p);
    let h = 0.1;
    let w = vec2f(d, abs(p.z) - h);
    return min(max(w.x, w.y), 0.0) + length(max(w, vec2f(0.0, 0.0)));
}

fn sd_plane(p: vec3f) -> f32 {
    return dot(p, vec3f(0.0, 0.0, 1.0));
}

fn sdf(p: vec3f) -> f32 {
    return min(sd_plane(p), sdf_extruded(p));
    // return sdf_extruded(p);
    // return sd_plane(p);
    // return sdf_2d(p);
}

fn trace_ray(
    ray_origin: vec3f,
    ray_direction: vec3f,
    trace_iters: u32,
    tmin: f32,
    max_dist: f32,
    eps: f32
) -> RayTraceOutputs {
    var t: f32 = tmin;
    var h: f32 = 0.0;

    var p = ray_origin;
    for (var k: u32 = 0; k < trace_iters && t < max_dist; k++) {
        p = ray_origin + ray_direction * t;
        h = sdf(p);
        t += h;
    }

    return RayTraceOutputs (p, t, h <= eps);
}

fn visibility(ro: vec3f, end_pt: vec3f, trace_iters: u32, eps: f32, w: f32) -> f32 {
    let rd = normalize(end_pt - ro);
    var res: f32 = 1.0;
    var ph: f32 = 1e8;

    let maxt = length(end_pt - ro);
    // var t: f32 = 0.05;
    var t: f32 = 0.01;
    for (var i = 0u; i < trace_iters && t < maxt; i++) {
        let p = ro + rd * t;
        let h = sdf(p);
        if (h < eps) {
            return 0.0;
        }
        let y = h * h / (2.0 * ph);
        let d = sqrt(h * h - y * y);
        res = min(res, d / (w * max(0.0, t - y)));
        ph = h;
        t = t + h;
    }
    return clamp(res, 0.0, 1.0);
}

/*fn visibility(ro: vec3f, end_pt: vec3f, trace_iters: u32, eps: f32, k: f32) -> f32 {
  let rd = normalize(end_pt - ro);
  var res = 1.0;
  var t = 0.0;
  let maxt = length(end_pt - ro);
  for (var i = 0u; i < trace_iters && t < maxt; i = i + 1u) {
  let p = ro + rd * t;
  let h = sdf(p);
  if (h < eps) {
  return 0.0;
  }
  res = min(res, k * h / t);
  t = t + h;
  }
  return res;
  }*/

fn h(p: vec3f, index: i32) -> f32 {
    var forward = p;
    var backward = p;
    forward[index] += 1e-4;
    backward[index] -= 1e-4;
    return dot(vec3f(sdf(backward), sdf(p), sdf(forward)),
               vec3(1.0, 2.0, 1.0));
}

fn h_p(p: vec3f, index: i32) -> f32 {
    var forward = p;
    var backward = p;
    forward[index] += 1e-4;
    backward[index] -= 1e-4;
    return dot(vec2f(sdf(backward), sdf(forward)), vec2f(1.0, -1.0));
}

fn sobel_gradient_estimate(p: vec3f) -> vec3f {
    let h_x = h_p(p, 0) * h(p, 1) * h(p, 2);
    let h_y = h_p(p, 1) * h(p, 2) * h(p, 0);
    let h_z = h_p(p, 2) * h(p, 0) * h(p, 1);

    return normalize(-vec3f(h_x, h_y, h_z));
}

fn apply_fog(col: vec3f,
             t: f32,
             ro: vec3f,
             rd: vec3f) -> vec3f {
    let a: f32 = 0.5;
    let b: f32 = 0.05;

    var fogAmount = (a/b) * exp(-ro.y*b) * (1.0-exp(-t*rd.y*b))/rd.y;
    var fogColor = vec3f(0.5, 0.6, 0.7);
    return mix(col, fogColor, fogAmount);
}

fn fog_trace(
    color: vec3f,
    ro: vec3f,
    light: Light,
    endpoint: vec3f,
    trace_iters: u32,
) -> vec3f {
    let tmax = length(ro - endpoint);
    let rd = normalize(ro - endpoint);

    var accum = color;

    var fog_color = vec3f(0.5, 0.6, 0.7) * 5e-4;

    var t = 0.0;
    let step_size = tmax / f32(trace_iters);
    for (var i: u32 = 0; i < trace_iters; i++) {
        let p = t * rd + ro;

        let f = get_fog_density(p.xy) * settings.fog_density;
        let T = exp(-f * step_size);
        accum = accum * T;
        accum += fog_color * f;
        
        t += step_size;
    }

    return accum;
}

fn narrow_beam(a: vec3f, b: vec3f, limit: f32) -> f32 {
    let angle = acos(dot(a, b));
    return max(-angle + limit, 0.0);
}

fn filtered_squares(p: vec2<f32>, dpdx_p: vec2<f32>, dpdy_p: vec2<f32>) -> f32 {
    const N: f32 = 3.0;
    let w = max(abs(dpdx_p), abs(dpdy_p));
    let a = p + 0.5 * w;
    let b = p - 0.5 * w;
    let fract_a = fract(a) * N;
    let fract_b = fract(b) * N;
    let min_fract_a = min(fract_a, vec2<f32>(1.0, 1.0));
    let min_fract_b = min(fract_b, vec2<f32>(1.0, 1.0));
    let i = (floor(a) + min_fract_a - floor(b) - min_fract_b) / (N * w);
    return 1.0 - i.x * i.y;
}

fn lighting_simple(
    pos: vec3f,
    light: Light,
    camera_origin: vec3f,
    normal: vec3f
) -> vec3<f32> {
    let pi = radians(180.0);
    let shadow = visibility(
        pos,
        camera_origin,
        u32(settings.light_trace_samples),
        1e-6,
        0.1
    );
    let l = normalize(light.center.xyz - pos);
    
    let t = length(pos - camera_origin);
    let rd = normalize(pos - camera_origin);

    var intensity = light.intensity;
    if (light.focus > 0.0) {
        intensity = intensity * narrow_beam(l.xyz, light.direction.xyz, light.focus);
        // intensity = intensity * narrow_beam(l, light.direction.xyz, light.focus);
    }
    if (light.attenuation > 0.0) {
        intensity = intensity * (
            1 / pow(length(pos - light.center.xyz) * light.attenuation, 2.0)
        );
    }

    let color = light.color.xyz * intensity / pi * max(dot(normal, l), 0.0) * shadow;
    var ground_p = (pos.xy + settings.world_origin.xy);
    ground_p *= 200.0;
    let ground_color = color * select(
        1.0,
        filtered_squares(
            ground_p.xy,
            dpdx(ground_p.xy),
            dpdy(ground_p.xy)
        ),
        pos.z < 0.05
    );
    return fog_trace(ground_color, pos, light, camera_origin, settings.fog_trace_samples);
    // return apply_fog(color, t, camera_origin, rd);
}

@fragment fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let screen_size = vec2f(textureDimensions(seed_texture));
    let uv = vec2f(mesh.uv.x, mesh.uv.y);

    if (settings.toggle_2d > 0) {
        return textureSample(screen_texture, seed_sampler, uv.xy);
    }

    let inside_texture = textureSample(screen_texture, seed_sampler, uv.xy).a > 0.5;
    let height = select(0.0, 0.5, inside_texture);

    let ro = vec3(0.5, 0.5, 1.0);
    let ro_lighting = vec3(0.5, 0.5, 0.3);

    let rd = normalize(vec3(uv, 0.0) - ro);
    let ray_outputs = trace_ray(
        ro,
        rd,
        u32(settings.ray_trace_samples),
        0.01,
        1000.0,
        1e-4
    );
    let endpoints = ray_outputs.intersection;
    let normal_sample_pt = endpoints - rd * 1e-4;
    let normal = sobel_gradient_estimate(normal_sample_pt);

    var total_light = vec3(0.0);
    for (var light_i = 0; light_i < settings.num_lights; light_i++) {
        var light = lights.lights[light_i];
        if (light.flicker > 0) {
            light.intensity *= (fbm(vec2f(view_bindings::globals.time * 3.0, 0.0)));
        }
        if length(light.center.xyz - endpoints) > 1.5 {
            continue;
        }
        total_light += lighting_simple(endpoints, light, ro_lighting, normal);
    }
    let ui_elem = textureSample(
        ui_texture,
        seed_sampler,
        (ro + rd * 1.4).xy  // roughly trying to look at the ground here
        // endpoints.xy
    );
    let color = mix(ui_elem.xyz, total_light, 1 - ui_elem.a);
    return vec4(color, 1.0);
}
