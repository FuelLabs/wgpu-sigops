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

@group(0) @binding(0) var<storage, read_write> s: array<u32, 16>;
@group(0) @binding(1) var<storage, read_write> k: array<u32, 24>;
@group(0) @binding(2) var<storage, read_write> ayr: array<u32, 16>;
@group(0) @binding(3) var<storage, read_write> x_sign: u32;
@group(0) @binding(4) var<storage, read_write> result: ETEPoint;

@compute
@workgroup_size(1)
fn test_verify(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var s_u32s: array<u32, 16>;
    var ayr_u32s: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        s_u32s[i] = s[i];
        ayr_u32s[i] = ayr[i];
    }

    var k_u32s: array<u32, 24>;
    for (var i = 0u; i < 24u; i ++) {
        k_u32s[i] = k[i];
    }

    // Compute the hash
    var hash_u32s: array<u32, 16> = sha512_96(&k_u32s);

    // Rearrange the bytes
    var l_limbs: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        let a = hash_u32s[i] >> 24u;
        let b = (hash_u32s[i] >> 16u) & 0xff;
        let c = (hash_u32s[i] >> 8u) & 0xff;
        let d = hash_u32s[i] & 0xff;
        l_limbs[15u - i] = a + (b << 8u) + (c << 16u) + (d << 24u);
    }

    // 32 x 16-bit limbs
    var l_reduced_bytes_le: array<u32, 32> = ed25519_reduce_fr(&l_limbs);
    var l_reduced_bytes_be: array<u32, 32>;
    for (var i = 0u; i < 32u; i ++) {
        l_reduced_bytes_be[i] = l_reduced_bytes_le[31u - i];
    }

    var k_val = bytes_be_to_limbs_le(&l_reduced_bytes_be);

    var s_bytes_be = u32s_to_bytes_be(&s_u32s);
    var s_val = bytes_be_to_limbs_le(&s_bytes_be);

    var ayr_bytes_be = u32s_to_bytes_be(&ayr_u32s);
    var ayr_val = bytes_be_to_limbs_le(&ayr_bytes_be);

    var compressed = x_sign == 1u;
    var p = get_p();
    result = ed25519_verify(&s_val, &k_val, &ayr_val, compressed, &p);
}
