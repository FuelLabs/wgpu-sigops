{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "ed25519_curve.wgsl" %}
{% include "constants.wgsl" %}
{% include "ed25519_constants.wgsl" %}
{% include "ed25519_utils.wgsl" %}
{% include "ed25519_eddsa.wgsl" %}
{% include "bytes_be_to_limbs_le.wgsl" %}
{% include "sha512.wgsl" %}
{% include "ed25519_reduce_fr.wgsl" %}

@group(0) @binding(0) var<storage, read_write> signature: array<u32>;
@group(0) @binding(1) var<storage, read_write> result: BigInt;

@compute
@workgroup_size(1)
fn test_verify(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var s_u32s: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        s_u32s[i] = signature[16u + i];
    }
    var s_bytes_be = u32s_to_bytes_be(&s_u32s);
    var s_val = bytes_be_to_limbs_le(&s_bytes_be);

    result = s_val;
}
