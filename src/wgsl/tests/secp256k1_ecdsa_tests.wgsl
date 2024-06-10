{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "secp256k1_curve.wgsl" %}
{% include "signature.wgsl" %}
{% include "secp256k1_ecdsa.wgsl" %}
{% include "constants.wgsl" %}
{% include "secp_constants.wgsl" %}
{% include "secp256k1_curve_generators.wgsl" %}
{% include "bytes_be_to_limbs_le.wgsl" %}

@group(0) @binding(0) var<storage, read_write> sig: array<u32, 16>;
@group(0) @binding(1) var<storage, read_write> msg: array<u32, 8>;
@group(0) @binding(2) var<storage, read_write> result: Point;

@compute
@workgroup_size(1)
fn test_secp256k1_recover(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sig_r_u32s: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        sig_r_u32s[i] = sig[i];
    }

    var r_bytes_be: array<u32, 32>;
    var s_bytes_be: array<u32, 32>;
    for (var i = 0u; i < 8u; i++) {
        let r = sig_r_u32s[i];
        let s = sig_r_u32s[8u + i];
        for (var j = 0u; j < 4u; j ++) {
            r_bytes_be[(i * 4 + j)] = (r >> (j * 8u)) & 255u;
            s_bytes_be[(i * 4 + j)] = (s >> (j * 8u)) & 255u;
        }
    }

    var msg_u32s: array<u32, 8>;
    for (var i = 0u; i < 8u; i ++) {
        msg_u32s[i] = msg[i];
    }
    var msg_bytes_be: array<u32, 32>;
    for (var i = 0u; i < 8u; i++) {
        let m = msg_u32s[i];
        for (var j = 0u; j < 4u; j ++) {
            msg_bytes_be[(i * 4 + j)] = (m >> (j * 8u)) & 255u;
        }
    }

    var p_bigint = get_p();
    var p_wide = get_p_wide();
    var scalar_p = get_scalar_p();
    var scalar_p_wide = get_scalar_p_wide();
    var mu_fp = get_mu_fp();
    var mu_fr = get_mu_fr();
    var r = get_r();
    var rinv = get_rinv();

    result = secp256k1_ecrecover(&r_bytes_be, &s_bytes_be, &msg_bytes_be, &p_bigint, &p_wide, &scalar_p, &scalar_p_wide, &r, &rinv, &mu_fp, &mu_fr);
}