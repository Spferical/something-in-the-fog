struct RayTraceOutputs {
    intersection: vec3f,
    dist: f32,
    hit: bool
}

struct Light {
    color: vec4f,
    intensity: f32,
    center: vec4f,
    direction: vec4f,
    focus: f32,
    attenuation: f32,
    flicker: i32
    
    // _padding: f32
}

struct LightBundle {
    lights: array<Light, 8>
}

struct LightingSettings {
    toggle_2d: i32,
    num_lights: i32,
    world_origin: vec3f,
    light_trace_samples: u32,
    ray_trace_samples: u32,
    fog_trace_samples: u32,
    fog_density: f32,
    padding: vec3f,
    // _padding: None
}

struct SdfSettings {
    iteration: i32,
    probe_size: i32,
    
    _padding: vec2f
}
