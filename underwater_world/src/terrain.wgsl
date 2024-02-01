struct CameraUniform {
    view_proj: mat4x4<f32>,
    fog_color: vec3<f32>,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) dist: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.dist = length(out.clip_position.xyz);
    return out;
}

//----------------------------------------------------------------------------//

// srgb_color = ((rgb_color / 255 + 0.055) / 1.055) ^ 2.4
fn color_convert_srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    let linear = (srgb + 0.055) / 1.055;
    return pow(linear, vec3<f32>(2.4, 2.4, 2.4));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let in_color = color_convert_srgb_to_linear(in.color);

    let dist_value = smoothstep(0.0, 40.0, in.dist);
    let dist = vec3<f32>(dist_value, dist_value, dist_value);

    let output = mix(in_color, camera.fog_color, dist);
    return vec4<f32>(output, 1.0);
}