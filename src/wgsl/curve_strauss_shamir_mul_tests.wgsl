{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "curve.wgsl" %}
{% include "constants.wgsl" %}

@group(0) @binding(0) var<storage, read_write> pt_a: Point;
@group(0) @binding(1) var<storage, read_write> pt_b: Point;
@group(0) @binding(2) var<storage, read_write> xr: BigInt;
@group(0) @binding(3) var<storage, read_write> yr: BigInt;
@group(0) @binding(4) var<storage, read_write> result: Point;

@compute
@workgroup_size(1)
fn test_strauss_shamir_mul(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var p = get_p();
    var p_wide = get_p_wide();
    var mu_fp = get_mu_fp();

    var pt_a_point = pt_a;
    var pt_b_point = pt_b;
    var xr_bigint = xr;
    var yr_bigint = yr;

    var rinv = get_rinv();

    var x = ff_mul(&xr_bigint, &rinv, &p, &p_wide, &mu_fp);
    var y = ff_mul(&yr_bigint, &rinv, &p, &p_wide, &mu_fp);

    result = jacobian_strauss_shamir_mul(&pt_a_point, &pt_b_point, &x, &y, &p);
}
