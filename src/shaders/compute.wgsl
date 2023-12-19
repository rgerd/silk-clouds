@group(0) @binding(0)
var<storage, read_write> output: array<u32, 12>;

@compute @workgroup_size(12, 1, 1) fn main(
    @builtin(local_invocation_id) local_thread_idx: vec3<u32>
) {
    output[local_thread_idx.x] = local_thread_idx.x;
    output[5u] = 26u;
    output[6u] = 7u;
    output[7u] = 7u;
    output[8u] = 7u;
    output[9u] = 7u;
    output[10u] = 7u;
}