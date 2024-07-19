{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "ed25519_curve.wgsl" %}
{% include "constants.wgsl" %}
{% include "ed25519_constants.wgsl" %}
{% include "ed25519_utils.wgsl" %}
{% include "ed25519_eddsa.wgsl" %}
{% include "bytes_be_to_limbs_le.wgsl" %}
{% include "limbs_le_to_u32s_be.wgsl" %}
{% include "sha512.wgsl" %}
{% include "ed25519_reduce_fr.wgsl" %}

@group(0) @binding(0) var<storage, read_write> signature: array<u32>;
@group(0) @binding(1) var<storage, read_write> pk: array<u32>;
@group(0) @binding(2) var<storage, read_write> msg: array<u32>;
@group(0) @binding(3) var<storage, read_write> is_valid: array<u32>;
@group(0) @binding(4) var<storage, read_write> success: u32;
@group(0) @binding(5) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn ed25519_verify_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gidx = global_id.x; 
    let gidy = global_id.y; 
    let gidz = global_id.z; 
    let num_x_workgroups = params[0];
    let num_y_workgroups = params[1];
    let num_z_workgroups = params[2];
    let id = (gidx * num_y_workgroups + gidy) * num_z_workgroups + gidz;

    var p = get_p();
    var r = get_r();
    var p_wide = get_p_wide();
    var rinv = get_rinv();
    var mu_fp = get_mu_fp();

    var compressed_r_u32s: array<u32, 16>;
    var s_u32s: array<u32, 16>;
    var pk_u32s: array<u32, 16>;
    var msg_u32s: array<u32, 16>;

    for (var i = 0u; i < 8u; i ++) {
        compressed_r_u32s[i] = signature[id * 16u + i];
        s_u32s[7u - i] = u32_be_to_le(signature[id * 16u + 8u + i]);
        pk_u32s[i] = pk[id * 8u + i];
        msg_u32s[i] = msg[id * 8u + i];
    }

    // TODO: optimise these into one function
    var s_bytes_be = u32s_to_bytes_be(&s_u32s);
    var s_val = bytes_be_to_limbs_le(&s_bytes_be);

    var ay_bytes_be = u32s_to_bytes_be(&pk_u32s);
    var compressed_sign_bit = ay_bytes_be[31] >> 7u;
    ay_bytes_be[31] &= 0x7fu;
    var ay_bytes_le: array<u32, 32>;
    for (var i = 0u; i < 32u; i ++) {
        ay_bytes_le[i] = ay_bytes_be[31u - i];
    }
    var ay_val = bytes_be_to_limbs_le(&ay_bytes_le);

    // Reduce ay_val
    if (bigint_gte(&ay_val, &p)) {
        ay_val = bigint_sub(&ay_val, &p);
    }
    var ayr_val = ff_mul(&ay_val, &r, &p, &p_wide, &mu_fp);

    // Prepare the preimage
    var k_u32s: array<u32, 24>;
    for (var i = 0u; i < 8u; i ++) {
        k_u32s[i] = u32_be_to_le(compressed_r_u32s[i]);
        k_u32s[i + 8u] = u32_be_to_le(pk_u32s[i]);
        k_u32s[i + 16u] = u32_be_to_le(msg_u32s[i]);
    }

    // Compute the hash
    var hash_u32s: array<u32, 16> = sha512_96(&k_u32s);

    // Rearrange the bytes
    var l_limbs: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        l_limbs[15u - i] = u32_be_to_le(hash_u32s[i]);
    }

    // 32 x 16-bit limbs
    var l_reduced_bytes_le: array<u32, 32> = ed25519_reduce_fr(&l_limbs);
    var l_reduced_bytes_be: array<u32, 32>;
    for (var i = 0u; i < 32u; i ++) {
        l_reduced_bytes_be[i] = l_reduced_bytes_le[31u - i];
    }

    var k_val = bytes_be_to_limbs_le(&l_reduced_bytes_be);

    var compressed = compressed_sign_bit == 1u;

    var result_ete_pt = ed25519_verify(&s_val, &k_val, &ayr_val, compressed, &p, &p_wide, &rinv, &mu_fp);
    var result_affine = ete_to_affine_non_mont(&result_ete_pt, &p, &p_wide, &r, &rinv, &mu_fp);

    var compressed_y_u32s = compress_eteaffine(&result_affine, {{ log_limb_size }}u);

    var v = 1u;
    for (var i = 0u; i < 8u; i ++) {
        if (compressed_y_u32s[7u - i] != u32_be_to_le(signature[id * 16u + i])) {
            v = 0u;
            break;
        }
    }

    is_valid[id] = v;

    success = 1u;
}
