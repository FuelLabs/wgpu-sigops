{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "secp256k1_curve.wgsl" %}
{% include "signature.wgsl" %}
{% include "secp256k1_ecdsa.wgsl" %}
{% include "constants.wgsl" %}
{% include "secp_constants.wgsl" %}
{% include "secp_curve_utils.wgsl" %}
{% include "secp256k1_curve_generators.wgsl" %}
{% include "bytes_be_to_limbs_le.wgsl" %}
{% include "limbs_le_to_u32s_be.wgsl" %}

@group(0) @binding(0) var<storage, read_write> u1g: array<Point>;
@group(0) @binding(1) var<storage, read_write> u2r: array<Point>;
@group(0) @binding(2) var<storage, read_write> sum: array<Point>;
@group(0) @binding(3) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn secp256k1_recover_3(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gidx = global_id.x; 
    let gidy = global_id.y; 
    let gidz = global_id.z; 
    let num_x_workgroups = params[0];
    let num_y_workgroups = params[1];
    let num_z_workgroups = params[2];
    let id = (gidx * num_y_workgroups + gidy) * num_z_workgroups + gidz;

    var p = get_p();
    var p_wide = get_p_wide();
    var scalar_p = get_scalar_p();
    var scalar_p_wide = get_scalar_p_wide();
    var mu_fp = get_mu_fp();
    var mu_fr = get_mu_fr();
    var r = get_r();
    var rinv = get_rinv();

    var u1g_pt = u1g[id];
    var u2r_pt = u2r[id];

    // Add u1g and u2r
    sum[id] = projective_add_2007_bl_unsafe(&u1g_pt, &u2r_pt, &p);
}
