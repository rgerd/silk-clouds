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
var<storage, read_write> vertices: array<Vertex, 15>;

@compute @workgroup_size(5, 1, 1)
fn main(@builtin(local_invocation_id) thread_id : vec3<u32>) {
    if (thread_id.x == 0u) {
        draw_command.vertex_count = 0u;
        draw_command.instance_count = 1u;
        draw_command.first_vertex = 0u;
        draw_command.first_instance = 0u;
    }
    atomicAdd(&draw_command.vertex_count, 3u);
    vertices[(thread_id.x * 3u) + 0u] = Vertex(
        vec4<f32>(0.0, 0.0, f32(thread_id.x), 1.0),
        vec4<f32>(0.0, 0.0, 1.0, 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0)
    );
    vertices[(thread_id.x * 3u) + 1u] = Vertex(
        vec4<f32>(1.0, 0.0, f32(thread_id.x), 1.0),
        vec4<f32>(1.0, 0.0, 0.0, 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0)
    );
    vertices[(thread_id.x * 3u) + 2u] = Vertex(
        vec4<f32>(0.0, 1.0, f32(thread_id.x), 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0),
        vec4<f32>(0.0, 1.0, 0.0, 1.0)
    );
}