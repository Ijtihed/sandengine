@group(0) @binding(0) var grid_tex: texture_2d<f32>;
@group(0) @binding(1) var grid_sampler: sampler;

struct VsOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOutput {
    // Fullscreen triangle: 3 vertices covering entire viewport
    let x = f32(i32(vi % 2u)) * 4.0 - 1.0;
    let y = f32(i32(vi / 2u)) * 4.0 - 1.0;

    var out: VsOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOutput) -> @location(0) vec4<f32> {
    return textureSample(grid_tex, grid_sampler, in.uv);
}
