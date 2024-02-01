struct CameraUniform {
    view_proj: mat4x4<f32>,
    fog_color: vec3<f32>,
    _padding: f32,
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

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) time: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) dist: f32,
}

const HEAD_X: f32 = 0.0;
const HEAD_AMP: f32 = 0.01;

const SPEED_Z: f32 = 1.5;
const FREQ_Z: f32 = 2.0;
const AMP_Z: f32 = 0.02;

const AMP_X: f32 = 0.04;
const AMP_Y: f32 = 0.075;

const SPEED_X_Y: f32 = 4.0;
const FREQ_X_Y: f32 = 2.0;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let wave_x_y = sin((model.position.x + instance.time * SPEED_X_Y) * FREQ_X_Y);
    let wave_z =   sin((model.position.x + instance.time * SPEED_Z)   * FREQ_Z  );

    let amp_x = f32(model.position.x > HEAD_X) * HEAD_AMP + f32(model.position.x <= HEAD_X) * AMP_X;
    let amp_y = f32(model.position.x > HEAD_X) * HEAD_AMP + f32(model.position.x <= HEAD_X) * AMP_Y;

    let x = model.position.x + wave_x_y * amp_x;
    let y = model.position.y + wave_x_y * amp_y;
    let z = model.position.z + wave_z * AMP_Z;
    let pos = vec4<f32>(x, y, z, 1.0);

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * pos;
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
    let in_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let fog_color = vec4<f32>(camera.fog_color, 1.0);

    let dist_value = smoothstep(0.0, 40.0, in.dist);
    let dist = vec4<f32>(dist_value, dist_value, dist_value, dist_value);

    let output = mix(in_color, fog_color, dist);
    return output;
}