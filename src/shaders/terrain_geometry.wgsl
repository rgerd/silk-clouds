struct IndirectDrawCommand {
    vertex_count: atomic<u32>,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32
};

struct Vertex {
    position: vec4<f32>,
    color: vec4<f32>,
    normal: vec4<f32>
};

@group(0) @binding(0)
var terrain: texture_storage_3d<r32float, read>;

@group(0) @binding(1)
var<storage, read_write> draw_command: IndirectDrawCommand;

@group(0) @binding(2)
var<storage, read_write> vertices: array<Vertex>;

@compute @workgroup_size(65, 1, 1)
fn main(@builtin(global_invocation_id) thread_id : vec3<u32>) {
    let vertex_idx = atomicAdd(&draw_command.vertex_count, 3u);
    let size = 0.4;
    let offs = vec4<f32>(f32(thread_id.x), f32(thread_id.y), f32(thread_id.z), 0.0) * size;
    vertices[vertex_idx + 0u] = Vertex(
        vec4<f32>(0.0, 0.0, 0.0, 1.0) + offs,
        vec4<f32>(0.0, 0.0, 1.0, 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0)
    );
    vertices[vertex_idx + 1u] = Vertex(
        vec4<f32>(size, 0.0, 0.0, 1.0) + offs,
        vec4<f32>(1.0, 0.0, 0.0, 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0)
    );
    vertices[vertex_idx + 2u] = Vertex(
        vec4<f32>(0.0, size, 0.0, 1.0) + offs,
        vec4<f32>(0.0, 1.0, 0.0, 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0)
    );
}