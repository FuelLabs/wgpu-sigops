{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "curve.wgsl" %}

@group(0) @binding(0) var<storage, read_write> p: BigInt;
@group(0) @binding(1) var<storage, read_write> a: Point;
@group(0) @binding(2) var<storage, read_write> b: Point;
@group(0) @binding(3) var<storage, read_write> result: Point;

@compute
@workgroup_size(1)
fn test_jacobian_add_2007_bl_unsafe(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var p_bigint = p;
    var a_pt = a;
    var b_pt = b;
    var result_pt = jacobian_add_2007_bl_unsafe(&a_pt, &b_pt, &p_bigint);
    result = result_pt;
}

@compute
@workgroup_size(1)
fn test_jacobian_dbl_2009_l(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var p_bigint = p;
    var a_pt = a;
    var b_pt = b;
    var result_pt = jacobian_dbl_2009_l(&a_pt, &p_bigint);
    result = result_pt;
}
