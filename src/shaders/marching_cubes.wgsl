struct PushConstants {
    time: f32,
    chunk_id: u32
}

var<push_constant> push: PushConstants;

const EPSILON: f32 = 0.0000001;
const VOXELS_PER_CHUNK_DIM: u32 = 50u;

struct IndirectDrawCommand {
    vertex_count: atomic<u32>,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32
};

struct Vertex {
    position: vec4<f32>,
    normal: vec4<f32>
};

@group(0) @binding(0)
var density: texture_storage_3d<rgba16float, read>;

@group(0) @binding(1)
var<storage, read_write> draw_command: IndirectDrawCommand;

@group(0) @binding(2)
var<storage, read_write> vertices: array<Vertex>;

@group(0) @binding(3)
var<storage, read> edge_table: array<u32, 256>;

@group(0) @binding(4)
var<storage, read> tri_table: array<array<i32, 16>, 256>;



fn vertexInterp(iso_level: f32, p1: vec3<u32>, p2: vec3<u32>, n1: vec3<f32>, n2: vec3<f32>, v1: f32, v2: f32) -> Vertex {
    let _p1 = vec4<f32>(vec3<f32>(p1), 1.0);
    let _p2 = vec4<f32>(vec3<f32>(p2), 1.0);
    let mu = clamp(max(abs(iso_level - v1), EPSILON) / max(abs(v2 - v1), EPSILON), 0.0, 1.0);

    var vert = Vertex();
    vert.position = mix(_p1, _p2, mu);

    let chunk_offset = vec3<f32>(
        f32((push.chunk_id >> 0u) & 1u),
        f32((push.chunk_id >> 1u) & 1u),
        f32((push.chunk_id >> 2u) & 1u)
    );
    let global_xyz = vert.position.xyz / f32(VOXELS_PER_CHUNK_DIM) + chunk_offset;

    vert.normal = vec4(mix(n1, n2, mu), (global_xyz.y / 2.0));

    return vert;
}

@compute @workgroup_size(10, 10, 10)
fn main(@builtin(global_invocation_id) thread_id : vec3<u32>) {
    let iso_level = 0.5;
    let positions = array<vec3<u32>, 8>(
        thread_id + vec3<u32>(0u, 0u, 0u),
        thread_id + vec3<u32>(1u, 0u, 0u),
        thread_id + vec3<u32>(1u, 0u, 1u),
        thread_id + vec3<u32>(0u, 0u, 1u),
        thread_id + vec3<u32>(0u, 1u, 0u),
        thread_id + vec3<u32>(1u, 1u, 0u),
        thread_id + vec3<u32>(1u, 1u, 1u),
        thread_id + vec3<u32>(0u, 1u, 1u)
    );
    let data = array<vec4<f32>, 8>(
        textureLoad(density, positions[0u]),
        textureLoad(density, positions[1u]),
        textureLoad(density, positions[2u]),
        textureLoad(density, positions[3u]),
        textureLoad(density, positions[4u]),
        textureLoad(density, positions[5u]),
        textureLoad(density, positions[6u]),
        textureLoad(density, positions[7u]),
    );
    let cube_index =
        (u32(step(data[0u].x, iso_level)) << 0u) | 
        (u32(step(data[1u].x, iso_level)) << 1u) | 
        (u32(step(data[2u].x, iso_level)) << 2u) | 
        (u32(step(data[3u].x, iso_level)) << 3u) | 
        (u32(step(data[4u].x, iso_level)) << 4u) | 
        (u32(step(data[5u].x, iso_level)) << 5u) | 
        (u32(step(data[6u].x, iso_level)) << 6u) | 
        (u32(step(data[7u].x, iso_level)) << 7u);
    
    let tri_hash = edge_table[cube_index];

    var tri_verts = array<Vertex, 12>();

    if (((tri_hash >> 0u) & 1u) == 1u) {
        tri_verts[0] = vertexInterp(
            iso_level,
            positions[0u],
            positions[1u],
            data[0u].yzw,
            data[1u].yzw,
            data[0u].x,
            data[1u].x);
    }
    if (((tri_hash >> 1u) & 1u) == 1u) {
        tri_verts[1] = vertexInterp(
            iso_level,
            positions[1u],
            positions[2u],
            data[1u].yzw,
            data[2u].yzw,
            data[1u].x,
            data[2u].x);
    }
    if (((tri_hash >> 2u) & 1u) == 1u) {
        tri_verts[2] = vertexInterp(
            iso_level,
            positions[2u],
            positions[3u],
            data[2u].yzw,
            data[3u].yzw,
            data[2u].x,
            data[3u].x);
    }
    if (((tri_hash >> 3u) & 1u) == 1u) {
        tri_verts[3] = vertexInterp(
            iso_level,
            positions[3u],
            positions[0u],
            data[3u].yzw,
            data[0u].yzw,
            data[3u].x,
            data[0u].x);
    }
    if (((tri_hash >> 4u) & 1u) == 1u) {
        tri_verts[4] = vertexInterp(
            iso_level,
            positions[4u],
            positions[5u],
            data[4u].yzw,
            data[5u].yzw,
            data[4u].x,
            data[5u].x);
    }
    if (((tri_hash >> 5u) & 1u) == 1u) {
        tri_verts[5] = vertexInterp(
            iso_level,
            positions[5u],
            positions[6u],
            data[5u].yzw,
            data[6u].yzw,
            data[5u].x,
            data[6u].x);
    }
    if (((tri_hash >> 6u) & 1u) == 1u) {
        tri_verts[6] = vertexInterp(
            iso_level,
            positions[6u],
            positions[7u],
            data[6u].yzw,
            data[7u].yzw,
            data[6u].x,
            data[7u].x);
    }
    if (((tri_hash >> 7u) & 1u) == 1u) {
        tri_verts[7] = vertexInterp(
            iso_level,
            positions[7u],
            positions[4u],
            data[7u].yzw,
            data[4u].yzw,
            data[7u].x,
            data[4u].x);
    }
    if (((tri_hash >> 8u) & 1u) == 1u) {
        tri_verts[8] = vertexInterp(
            iso_level,
            positions[0u],
            positions[4u],
            data[0u].yzw,
            data[4u].yzw,
            data[0u].x,
            data[4u].x);
    }
    if (((tri_hash >> 9u) & 1u) == 1u) {
        tri_verts[9] = vertexInterp(
            iso_level,
            positions[1u],
            positions[5u],
            data[1u].yzw,
            data[5u].yzw,
            data[1u].x,
            data[5u].x);
    }
    if (((tri_hash >> 10u) & 1u) == 1u) {
        tri_verts[10] = vertexInterp(
            iso_level,
            positions[2u],
            positions[6u],
            data[2u].yzw,
            data[6u].yzw,
            data[2u].x,
            data[6u].x);
    }
    if (((tri_hash >> 11u) & 1u) == 1u) {
        tri_verts[11] = vertexInterp(
            iso_level,
            positions[3u],
            positions[7u],
            data[3u].yzw,
            data[7u].yzw,
            data[3u].x,
            data[7u].x);
    }
    var out_vert_count = 0u;
    var tri_vert_ids = tri_table[cube_index];
    for (; tri_vert_ids[out_vert_count] != -1; out_vert_count += 3u) {}
    let vertex_idx = atomicAdd(&draw_command.vertex_count, out_vert_count);
    for (var i = 0u; tri_vert_ids[i] != -1; i += 3u) {
        vertices[vertex_idx + i + 0u] = tri_verts[tri_vert_ids[i + 0u]];
        vertices[vertex_idx + i + 1u] = tri_verts[tri_vert_ids[i + 1u]];
        vertices[vertex_idx + i + 2u] = tri_verts[tri_vert_ids[i + 2u]];
    }
}