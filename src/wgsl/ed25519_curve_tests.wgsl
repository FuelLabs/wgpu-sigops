{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "ed25519_curve.wgsl" %}
{% include "constants.wgsl" %}
{% include "ed25519_constants.wgsl" %}

@group(0) @binding(0) var<storage, read_write> a: ETEPoint;
@group(0) @binding(1) var<storage, read_write> b: ETEPoint;
@group(0) @binding(2) var<storage, read_write> result: ETEPoint;

@compute
@workgroup_size(1)
fn test_ete_add_2008_hwcd_3(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var p_bigint = get_p();
    var a_pt = a;
    var b_pt = b;
    var result_pt = ete_add_2008_hwcd_3(&a_pt, &b_pt, &p_bigint);
    result = result_pt;
}

@compute
@workgroup_size(1)
fn test_ete_dbl_2008_hwcd(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var p_bigint = get_p();
    var a_pt = a;
    var b_pt = b;
    var result_pt = ete_dbl_2008_hwcd(&a_pt, &p_bigint);
    result = result_pt;
}
