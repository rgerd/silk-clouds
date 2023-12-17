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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = in.color;
    out.clip_position = camera.view_proj * (vec4<f32>(in.position, 1.0) + vec4<f32>(f32(in.instance_id) * 0.1, 0.0, 0.0, 0.0));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4(in.color, 1.0);
    return out;
}