struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var t_terrain: texture_3d<f32>;

@group(0) @binding(2)
var s_terrain: sampler;

struct VertexInput {
    @builtin(instance_index) index: u32,
    @location(0) pos: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec3<u32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var x = (in.index % 65u);
    var y = (in.index / 65u) % 65u;
    var z = in.index / 65u / 65u;
    out.tex_coords = vec3<u32>(x, y, z);
    var voxel_pos = vec3<f32>(f32(x), f32(y), f32(z)) / 65.0;
    var density = textureLoad(t_terrain, out.tex_coords, 0).r;
    out.position = camera.view_proj * vec4<f32>((((in.pos * density * 4.0) / 65.0 + voxel_pos) - 0.5) * 4.0, 1.0);
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(mix(vec3<f32>(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0), textureLoad(t_terrain, in.tex_coords, 0).r), 1.0);
    return out;

}