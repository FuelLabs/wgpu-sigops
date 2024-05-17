{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "ed25519_curve.wgsl" %}
{% include "constants.wgsl" %}
{% include "ed25519_constants.wgsl" %}
{% include "ed25519_utils.wgsl" %}

@group(0) @binding(0) var<storage, read_write> a: BigInt;
@group(0) @binding(1) var<storage, read_write> result: ETEPoint;
@group(0) @binding(2) var<storage, read_write> is_valid: BigInt;

@compute
@workgroup_size(1)
fn test_reconstruct_ete_from_uncompressed_y(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var yr_val = a;
    var p = get_p();

    var r = reconstruct_ete_from_y(&yr_val, false, &p);

    result = r.pt;
    if (r.is_valid_y_coord) {
        var one: BigInt;
        one.limbs[0] = 1u;
        is_valid = one;
    }
}
