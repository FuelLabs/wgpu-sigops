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

@group(0) @binding(0) var<storage, read_write> gs: array<ETEPoint>;
@group(0) @binding(1) var<storage, read_write> neg_ak: array<ETEPoint>;
@group(0) @binding(2) var<storage, read_write> pt: array<ETEAffinePoint>;
@group(0) @binding(3) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn ed25519_verify_main_5(@builtin(global_invocation_id) global_id: vec3<u32>) {
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

    var gs_pt = gs[id];
    var neg_ak_pt = neg_ak[id];

    var result_ete_pt = ete_add_2008_hwcd_3(&gs_pt, &neg_ak_pt, &p);

    pt[id] = ete_to_affine_non_mont(&result_ete_pt, &p, &p_wide, &r, &rinv, &mu_fp);
}
