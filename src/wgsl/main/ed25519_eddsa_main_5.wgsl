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

@group(0) @binding(0) var<storage, read_write> pt: array<ETEAffinePoint>;
@group(0) @binding(1) var<storage, read_write> is_valid: array<u32>;
@group(0) @binding(2) var<storage, read_write> sig: array<u32>;
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

    var result_affine = pt[id];

    var compressed_y_u32s = compress_eteaffine(&result_affine, {{ log_limb_size }}u);

    var v = 1u;
    for (var i = 0u; i < 8u; i ++) {
        if (compressed_y_u32s[7u - i] != u32_be_to_le(sig[id * 16u + i])) {
            v = 0u;
            break;
        }
    }

    is_valid[id] = v;
}
