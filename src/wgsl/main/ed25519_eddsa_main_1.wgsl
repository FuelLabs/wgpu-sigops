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

@group(0) @binding(0) var<storage, read_write> preimage: array<u32>;
@group(0) @binding(1) var<storage, read_write> k: array<BigInt>;
@group(0) @binding(2) var<uniform> params: vec3<u32>;

@compute
@workgroup_size(256)
fn ed25519_verify_main_1(@builtin(global_invocation_id) global_id: vec3<u32>) {
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

    var k_u32s: array<u32, 24>;
    for (var i = 0u; i < 24u; i ++) {
        k_u32s[i] = preimage[id * 24u + i];
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
    k[id] = k_val;
}
