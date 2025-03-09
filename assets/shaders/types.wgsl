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
    attenuation: f32
}

struct LightBundle {
    lights: array<Light, 8>
}

struct LightingSettings {
    tile_size: i32,
    toggle_2d: i32
}