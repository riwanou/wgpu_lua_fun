struct Globals {
    elasped: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

fn map(val: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    return min2 + (val - min1) * (max2 - min2) / (max1 - min1);
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    let val = map(cos(globals.elasped), -1.0, 1.0, 0.1, 0.4);
    return vec4<f32>(0.1, val, 0.2, 1.0);
}
