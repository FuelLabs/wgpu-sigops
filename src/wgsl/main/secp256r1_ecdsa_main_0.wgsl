{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "secp256r1_curve.wgsl" %}
{% include "signature.wgsl" %}
{% include "secp256r1_ecdsa.wgsl" %}
{% include "constants.wgsl" %}
{% include "secp_constants.wgsl" %}
{% include "secp_curve_utils.wgsl" %}
{% include "secp256r1_curve_generators.wgsl" %}
{% include "bytes_be_to_limbs_le.wgsl" %}
{% include "limbs_le_to_u32s_be.wgsl" %}

@group(0) @binding(0) var<storage, read_write> sig: array<u32>;
@group(0) @binding(1) var<storage, read_write> msg: array<u32>;
@group(0) @binding(2) var<storage, read_write> u1: array<BigInt>;
@group(0) @binding(3) var<storage, read_write> u2: array<BigInt>;
@group(0) @binding(4) var<storage, read_write> recovered_r: array<Point>;
@group(0) @binding(5) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn secp256r1_recover_0(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gidx = global_id.x; 
    let gidy = global_id.y; 
    let gidz = global_id.z; 
    let num_x_workgroups = params[0];
    let num_y_workgroups = params[1];
    let num_z_workgroups = params[2];
    let id = (gidx * num_y_workgroups + gidy) * num_z_workgroups + gidz;

    // Copy sig_r to the stack
    var sig_r_u32s: array<u32, 16>;
    for (var i = 0u; i < 16u; i ++) {
        sig_r_u32s[i] = sig[id * 16u + i];
    }

    // Copy msg to the stack
    var msg_u32s: array<u32, 8>;
    for (var i = 0u; i < 8u; i ++) {
        msg_u32s[i] = msg[id * 8u + i];
    }

    // Convert r, s, and msg to bytes
    var r_bytes_be: array<u32, 32>;
    var s_bytes_be: array<u32, 32>;
    var msg_bytes_be: array<u32, 32>;
    for (var i = 0u; i < 8u; i++) {
        let r = sig_r_u32s[i];
        let s = sig_r_u32s[8u + i];
        let m = msg_u32s[i];
        for (var j = 0u; j < 4u; j ++) {
            var idx = i * 4u + j;
            var j8 = j * 8u;
            r_bytes_be[idx] = (r >> j8) & 255u;
            s_bytes_be[idx] = (s >> j8) & 255u;
            msg_bytes_be[idx] = (m >> j8) & 255u;
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

    // Perform the first step of ECDSA recovery to produce u1, u2, and recovered_r
    var intermediate = secp256r1_ecrecover_0(&r_bytes_be, &s_bytes_be, &msg_bytes_be, &p_bigint, &p_wide, &scalar_p, &scalar_p_wide, &r, &rinv, &mu_fp, &mu_fr);

    u1[id] = intermediate.u1;
    u2[id] = intermediate.u2;
    recovered_r[id] = intermediate.recovered_r;
}
