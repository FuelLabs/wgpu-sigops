{% include "ed25519_reduce_fr.wgsl" %}

@group(0) @binding(0) var<storage, read_write> input: array<u32, 16>;
@group(0) @binding(1) var<storage, read_write> result: array<u32, 32>;

@compute
@workgroup_size(1)
fn test_ed25519_reduce_fr(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        x[i] = input[i];
    }

    var reduced = ed25519_reduce_fr(&x);
    result = reduced;
}

@compute
@workgroup_size(1)
fn test_convert_512_be_to_le(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var x: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        x[i] = input[i];
    }

    result = convert_512_be_to_le(&x);
}
