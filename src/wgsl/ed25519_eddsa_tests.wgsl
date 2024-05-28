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
@group(0) @binding(2) var<storage, read_write> result: ETEPoint;

@compute
@workgroup_size(1)
fn test_verify(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var s_u32s: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        s_u32s[i] = s[i];
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

    // Rearrange the bytes
    var ay_u32s: array<u32, 16>;
    for (var i = 0u; i < 8u; i ++) {
        let v = k[i + 8u];
        let a = v >> 24u;
        let b = (v >> 16u) & 0xff;
        let c = (v >> 8u) & 0xff;
        let d = v & 0xff;
        ay_u32s[i] = a + (b << 8u) + (c << 16u) + (d << 24u);
    }
    var ay_bytes_be = u32s_to_bytes_be(&ay_u32s);
    var compressed_sign_bit = ay_bytes_be[31] >> 7u;
    ay_bytes_be[31] &= 0x7fu;
    var ay_bytes_le: array<u32, 32>;
    for (var i = 0u; i < 32u; i ++) {
        ay_bytes_le[i] = ay_bytes_be[31u - i];
    }
    var ay_val = bytes_be_to_limbs_le(&ay_bytes_le);
    var p = get_p();
    var r = get_r();
    var p_wide = get_p_wide();
    var mu_fp = get_mu_fp();

    // Reduce ay_val
    if (bigint_gte(&ay_val, &p)) {
        ay_val = bigint_sub(&ay_val, &p);
    }
    var ayr_val = ff_mul(&ay_val, &r, &p, &p_wide, &mu_fp);

    var compressed = compressed_sign_bit == 1u;

    result = ed25519_verify(&s_val, &k_val, &ayr_val, compressed, &p);
}
