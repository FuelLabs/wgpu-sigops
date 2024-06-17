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

@group(0) @binding(0) var<storage, read_write> u2: array<BigInt>;
@group(0) @binding(1) var<storage, read_write> recovered_r: array<Point>;
@group(0) @binding(2) var<storage, read_write> u2r: array<Point>;
@group(0) @binding(3) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn secp256k1_recover_2(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gidx = global_id.x; 
    let gidy = global_id.y; 
    let gidz = global_id.z; 
    let num_x_workgroups = params[0];
    let num_y_workgroups = params[1];
    let num_z_workgroups = params[2];
    let id = (gidx * num_y_workgroups + gidy) * num_z_workgroups + gidz;

    var p = get_p();
    var recovered_r_pt = recovered_r[id];

    // Multiply recovered_r by u2
    var u2_val = u2[id];
    u2r[id] = projective_mul(&recovered_r_pt, &u2_val, &p);
}
