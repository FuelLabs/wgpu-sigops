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

@group(0) @binding(0) var<storage, read_write> table: array<ETEXYT>;
@group(0) @binding(1) var<storage, read_write> s: array<BigInt>;
@group(0) @binding(2) var<storage, read_write> ayr: array<BigInt>;
@group(0) @binding(3) var<storage, read_write> k: array<BigInt>;
@group(0) @binding(4) var<storage, read_write> compressed_sign_bit: array<u32>;
@group(0) @binding(5) var<storage, read_write> gs: array<ETEPoint>;
@group(0) @binding(6) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn ed25519_verify_main_2(@builtin(global_invocation_id) global_id: vec3<u32>) {
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

    var table_size = {{ table_size }}u;
    var table_pts: array<ETEXYT, {{ table_size }}>;
    for (var i = 0u; i < table_size; i ++) {
        table_pts[i] = table[i];
    }

    var s_val = s[id];
    var ayr_val = ayr[id];
    var k_val = k[id];
    var x_sign = compressed_sign_bit[id] == 1u;

    gs[id] = ete_fixed_mul(&table_pts, &s_val, &p, &r);
}
