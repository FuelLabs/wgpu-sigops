{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "secp256r1_curve.wgsl" %}
{% include "constants.wgsl" %}

@group(0) @binding(0) var<storage, read_write> a: Point;
@group(0) @binding(1) var<storage, read_write> b: Point;
@group(0) @binding(2) var<storage, read_write> result: Point;

@compute
@workgroup_size(1)
fn test_projective_add_2015_rcb_unsafe(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var p_bigint = get_p();
    var a_pt = a;
    var b_pt = b;
    var result_pt = projective_add_2015_rcb_unsafe(&a_pt, &b_pt, &p_bigint);
    result = result_pt;
}

@compute
@workgroup_size(1)
fn test_projective_dbl_2015_rcb(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var p_bigint = get_p();
    var a_pt = a;
    var b_pt = b;
    var result_pt = projective_dbl_2015_rcb(&a_pt, &p_bigint);
    result = result_pt;
}
