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

@group(0) @binding(0) var<storage, read_write> table: array<PointAffine>;
@group(0) @binding(1) var<storage, read_write> u1: array<BigInt>;
@group(0) @binding(2) var<storage, read_write> u1g: array<Point>;
@group(0) @binding(3) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn secp256k1_recover_1(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gidx = global_id.x; 
    let gidy = global_id.y; 
    let gidz = global_id.z; 
    let num_x_workgroups = params[0];
    let num_y_workgroups = params[1];
    let num_z_workgroups = params[2];
    let id = (gidx * num_y_workgroups + gidy) * num_z_workgroups + gidz;

    var p = get_p();
    var r = get_r();

    var table_size = {{ table_size }}u;
    var table_pts: array<PointAffine, {{ table_size }}>;
    for (var i = 0u; i < table_size; i ++) {
        table_pts[i] = table[i];
    }

    // Multiply g by u1
    var u1_val = u1[id];
    var result = projective_fixed_mul(&table_pts, &u1_val, &p, &r);

    /*
    var g = get_secp256k1_generator();
    var result = projective_mul(&g, &u1_val, &p);
    */
    u1g[id] = result;
}
