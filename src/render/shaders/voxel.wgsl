struct Uniforms {
    vp: mat4x4<f32>,
    light_dir: vec3<f32>,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

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
    let col = in.color;
    let sun = normalize(uniforms.light_dir);

    // Detect material type from color heuristics
    let is_fire = col.r > 0.5 && col.g < col.r * 0.9 && col.b < 0.25;
    let is_water = col.b > 0.5 && col.r < 0.2 && col.a < 0.85;
    let is_acid = col.g > 0.5 && col.r < 0.25 && col.b < 0.2;

    // Sun light with hemisphere wrap for softer shadows
    let sun_wrap = (dot(n, sun) + 0.3) / 1.3;
    let sun_dot = max(sun_wrap, 0.0);

    // Fill light from opposite side
    let fill = normalize(vec3<f32>(-0.5, 0.2, -0.4));
    let fill_dot = max(dot(n, fill), 0.0);

    // Sky light (hemisphere)
    let sky = 0.5 + 0.5 * n.y;

    // Combine lighting
    var ambient = 0.12;
    var sun_str = 0.52;
    var fill_str = 0.14;
    var sky_str = 0.18;

    // Fire is emissive -- minimal lighting influence
    if is_fire {
        ambient = 0.7;
        sun_str = 0.15;
        fill_str = 0.05;
        sky_str = 0.1;
    }

    let lighting = ambient + sun_dot * sun_str + fill_dot * fill_str + sky * sky_str;

    // Specular highlight for water/acid surfaces
    var spec = 0.0;
    if is_water || is_acid {
        let view_dir = normalize(-in.world_pos);
        let half_vec = normalize(sun + view_dir);
        let spec_dot = max(dot(n, half_vec), 0.0);
        spec = pow(spec_dot, 32.0) * 0.35;
    }

    // Height-based subtle darkening
    let h_frac = clamp(in.world_pos.y / 128.0, 0.0, 1.0);
    let h_boost = 0.88 + h_frac * 0.12;

    // Depth fade
    let depth = in.clip_position.z / in.clip_position.w;
    let fog = clamp(1.0 - depth * 0.6, 0.55, 1.0);

    var final_color = col.rgb * lighting * h_boost * fog + vec3<f32>(spec);

    // Fire glow boost
    if is_fire {
        final_color = final_color * 1.3;
    }

    return vec4<f32>(clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0)), col.a);
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
