struct Uniforms {
    vp: mat4x4<f32>,
    light_dir: vec3<f32>,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// --- Voxel instanced cube pipeline ---

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @location(2) offset: vec3<f32>,
    @location(3) color: vec4<f32>,
};

struct VsOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VsOutput {
    let world_pos = vert.position + inst.offset;
    var out: VsOutput;
    out.clip_position = uniforms.vp * vec4<f32>(world_pos, 1.0);
    out.world_pos = world_pos;
    out.normal = vert.normal;
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VsOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);

    // Primary sun light
    let sun = normalize(uniforms.light_dir);
    let sun_dot = max(dot(n, sun), 0.0);

    // Fill light from below-left for softer look
    let fill = normalize(vec3<f32>(-0.4, -0.3, 0.6));
    let fill_dot = max(dot(n, fill), 0.0);

    // Sky light from above
    let sky_dot = max(dot(n, vec3<f32>(0.0, 1.0, 0.0)), 0.0);

    let ambient = 0.18;
    let sun_contrib = sun_dot * 0.55;
    let fill_contrib = fill_dot * 0.12;
    let sky_contrib = sky_dot * 0.15;
    let lighting = ambient + sun_contrib + fill_contrib + sky_contrib;

    // Height-based darkening for grounded feel
    let height_fade = clamp(in.world_pos.y / 128.0, 0.0, 1.0);
    let height_boost = 0.85 + height_fade * 0.15;

    // Subtle distance fog
    let dist = length(in.clip_position.xyz);
    let fog_factor = clamp(1.0 - dist / 600.0, 0.6, 1.0);

    let lit_color = in.color.rgb * lighting * height_boost * fog_factor;
    return vec4<f32>(lit_color, in.color.a);
}

// --- Grid line pipeline ---

struct LineVsOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn grid_vs(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> LineVsOutput {
    var out: LineVsOutput;
    out.clip_position = uniforms.vp * vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

@fragment
fn grid_fs(in: LineVsOutput) -> @location(0) vec4<f32> {
    return in.color;
}
