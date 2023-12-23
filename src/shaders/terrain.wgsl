struct PushConstants {
  chunk_id: u32
}

var<push_constant> push: PushConstants;

const VOXELS_PER_CHUNK_DIM: u32 = 64u;

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
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
        f32(((push.chunk_id >> 0u) & 1u) * VOXELS_PER_CHUNK_DIM),
        f32(((push.chunk_id >> 1u) & 1u) * VOXELS_PER_CHUNK_DIM),
        f32(((push.chunk_id >> 2u) & 1u) * VOXELS_PER_CHUNK_DIM)
    );

    out.position = camera.view_proj * vec4((((in.position.xyz + chunk_offset) / f32(VOXELS_PER_CHUNK_DIM)) - 1.0) * 8.0, 1.0);
    out.color = mix(vec4<f32>(0.6, 0.3, 0.05, 1.0), vec4<f32>(0.9, 0.2, 0.4, 1.0), in.normal.w);
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
    out.color = vec4(in.color.xyz * min(max(-dot(normalize(in.normal), light_dir), 0.0) + AMBIENT, 1.0), 1.0);
    return out;
}