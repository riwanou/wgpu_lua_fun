struct Globals {
    clip_view: mat4x4<f32>,
    view_world: mat4x4<f32>,
    elapsed: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

struct InstanceInput {
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
    @location(7) normal_matrix_0: vec3<f32>,
    @location(8) normal_matrix_1: vec3<f32>,
    @location(9) normal_matrix_2: vec3<f32>,
}


fn map(val: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    return min2 + (val - min1) * (max2 - min2) / (max1 - min1);
}

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let world_local = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_rotation = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    var out: VertexOutput;

    out.tex_coords = model.tex_coord;
    out.world_normal = normal_rotation * model.normal;

    let world_position = world_local * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = globals.clip_view * globals.view_world * world_position;

    return out;
}

struct PointLight {
    position: vec3<f32>,
    radius: f32,
}

struct PointLightData {
    len: u32,
    data: array<PointLight>,
}

@group(1) @binding(0)
var<storage, read> point_lights: PointLightData;

struct SimpleMaterial {
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> uniform: SimpleMaterial;
@group(2) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(2)
var s_diffuse: sampler;

// Radius based attenuation
// https://lisyarus.github.io/blog/posts/point-light-attenuation.html
fn attenuate(distance: f32, radius: f32) -> f32 {
    let s = saturate(distance / radius);
    let s2 = s * s;
    let inv_s2 = 1.0 - s2;
    return inv_s2 * inv_s2 / (1.0 + s);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let val = map(cos(globals.elapsed * 3), -1.0, 1.0, 0.0, 1.0);
    var ease = 0.0;
    if val < 0.5 {
        ease = 4 * val * val * val;
    } else {
        let a = -2 * val + 2;
        ease = a * a * a / 2;
    }

    let diffuse_sample = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let color = mix(diffuse_sample.xyz, val * uniform.color, ease);

    return vec4<f32>(color, 1.0);
}
