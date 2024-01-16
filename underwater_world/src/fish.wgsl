struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) dist: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.dist = length(out.clip_position.xyz);
    return out;
}

//----------------------------------------------------------------------------//

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let dist_value = clamp(in.dist, 0.0, 30.0) / 30.0;
    let dist_value = smoothstep(0.0, 40.0, in.dist);
    let dist = vec4<f32>(dist_value, dist_value, dist_value, dist_value);
    let fog_color = vec4<f32>(0.0, 0.1, 0.2, 1.0);
    let output = mix(textureSample(t_diffuse, s_diffuse, in.tex_coords), fog_color, dist);
    return output;

    // return vec4<f32>(color_convert_srgb_to_linear(in.color), 1.0);   
}