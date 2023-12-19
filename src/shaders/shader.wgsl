struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_id: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) instance_location: vec3<f32>,
    @location(4) instance_scale: vec3<f32>,
    @location(5) instance_color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

fn rand(co: vec3<f32>) -> f32 {
    return fract(sin(dot(co, vec3(12.9898, 78.233, 17.828)) * 0.002) * 0.004);
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = in.color * (dot(in.normal, vec3(0.2, 0.3, 0.4)) + 0.1);
    out.clip_position = camera.view_proj * (vec4<f32>(in.position * in.instance_scale, 1.0) + vec4<f32>(in.instance_location, 1.0));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4(in.color, 1.0);
    return out;
}