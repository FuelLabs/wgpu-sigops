{% include "bigint.wgsl" %}
{% include "bytes_be_to_limbs_le.wgsl" %}

@group(0) @binding(0) var<storage, read_write> input: array<u32, 16>;
@group(0) @binding(1) var<storage, read_write> result: BigInt;

@compute
@workgroup_size(1)
fn test_bytes_be_to_limbs_le(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var input_u32s: array<u32, 8>;
    for (var i = 0u; i < 8u; i ++) {
        input_u32s[i] = input[i];
    }

    var bytes_be: array<u32, 32>;
    for (var i = 0u; i < 8u; i++) {
        let r = input_u32s[i];
        for (var j = 0u; j < 4u; j ++) {
            bytes_be[(i * 4 + j)] = (r >> (j * 8u)) & 255u;
        }
    }

    var r: BigInt;
    r = bytes_be_to_limbs_le(&bytes_be);
    result = r;
}
