{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "ed25519_curve.wgsl" %}
{% include "constants.wgsl" %}
{% include "ed25519_constants.wgsl" %}
{% include "ed25519_utils.wgsl" %}
{% include "ed25519_eddsa.wgsl" %}

@group(0) @binding(0) var<storage, read_write> s: BigInt;
@group(0) @binding(1) var<storage, read_write> k: BigInt;
@group(0) @binding(2) var<storage, read_write> ayr: BigInt;
@group(0) @binding(3) var<storage, read_write> x_sign: u32;
@group(0) @binding(4) var<storage, read_write> result: ETEPoint;

@compute
@workgroup_size(1)
fn test_verify(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var s_val = s;
    var k_val = k;
    var ayr_val = ayr;
    var compressed = x_sign == 1u;
    var p = get_p();
    result = ed25519_verify(&s_val, &k_val, &ayr_val, compressed, &p);
}
