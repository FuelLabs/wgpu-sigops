{% include "bigint.wgsl" %}
{% include "ff.wgsl" %}
{% include "mont.wgsl" %}
{% include "ed25519_curve.wgsl" %}
{% include "constants.wgsl" %}
{% include "ed25519_constants.wgsl" %}
{% include "ed25519_utils.wgsl" %}

@group(0) @binding(1) var<storage, read_write> compressed_y: array<u32, 8>;
@group(0) @binding(2) var<storage, read_write> result: ETEPoint;
@group(0) @binding(3) var<storage, read_write> is_valid: u32;

@compute
@workgroup_size(1)
fn test_compressed_y_to_eteprojective(@builtin(global_invocation_id) global_id: vec3<u32>) {
}
