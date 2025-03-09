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
    
    _padding: f32
}

struct LightBundle {
    lights: array<Light, 8>
}

struct LightingSettings {
    toggle_2d: i32,
    num_lights: i32,
    
    _padding: vec2f
}

struct SdfSettings {
    iteration: i32,
    probe_size: i32,
    
    _padding: vec2f
}
