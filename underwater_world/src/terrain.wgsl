struct CameraUniform {
    view_proj: mat4x4<f32>,
    fog_color: vec3<f32>,
    _padding1: f32,
    sub_pos: vec3<f32>,
    _padding2: f32,
    sub_forward: vec3<f32>,
    _padding3: f32,
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
    @location(2) light: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let half_pi = 3.14159 / 2.0;

    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);

    let dist_vec = model.position - camera.sub_pos;
    out.dist = length(dist_vec);

    let dot = dot(dist_vec, camera.sub_forward);
    let cos_angle = dot / (length(dist_vec) * length(camera.sub_forward)); 
    let angle = acos(cos_angle);

    let squished = angle * 3.0;
    let light = cos(squished) * f32(angle < half_pi);
    out.light = smoothstep(0.0, 1.0, light);

    return out;
}

//----------------------------------------------------------------------------//

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let min_dist_value = smoothstep(0.0, 20.0, clamp(in.dist, 0.0, 20.0));
    let max_dist_value = smoothstep(0.0, 40.0, clamp(in.dist, 0.0, 40.0));
    let dark_value = clamp(1.0 - in.light, max_dist_value, min_dist_value);

    let output = mix(in.color, camera.fog_color, dark_value);
    return vec4<f32>(output, 1.0);
}