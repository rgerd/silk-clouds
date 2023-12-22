struct PushConstants {
  chunk_id: u32
}

var<push_constant> push: PushConstants;

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Align to (0, 0, 0)
    let chunk_offset = vec3<f32>(
        f32((push.chunk_id >> 0u) & 1u) - 1.0,
        f32((push.chunk_id >> 1u) & 1u) - 1.0,
        f32((push.chunk_id >> 2u) & 1u) - 1.0
    );

    out.position = camera.view_proj * vec4(((in.position.xyz / 64.0) + chunk_offset) * 5.0, 1.0);
    out.color = in.color;
    out.normal = in.normal.xyz;
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

const AMBIENT: f32 = 0.1;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    let light_dir: vec3<f32> = normalize(vec3(-0.5, -1.0, 0.0));
    out.color = in.color * min(max(-dot(in.normal, light_dir), 0.0) + AMBIENT, 1.0);
    return out;
}