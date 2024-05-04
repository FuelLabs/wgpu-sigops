{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "constants.wgsl" %}

@group(0) @binding(0) var<storage, read_write> xr: BigInt;
@group(0) @binding(1) var<storage, read_write> result_a: BigInt;
@group(0) @binding(2) var<storage, read_write> result_b: BigInt;

@compute
@workgroup_size(1)
fn test_mont_sqrt_case3mod4(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var xr_bigint = xr;
    var p_bigint = get_p();
    var result = mont_sqrt_case3mod4(&xr_bigint, &p_bigint);

    for (var i = 0u; i < {{ num_limbs }}u; i ++) {
        result_a.limbs[i] = result[0].limbs[i];
        result_b.limbs[i] = result[1].limbs[i];
    }
}
