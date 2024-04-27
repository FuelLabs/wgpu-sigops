struct BigInt {
    limbs: array<u32, {{ num_limbs }}>
}

struct BigIntMediumWide {
    limbs: array<u32, {{ num_limbs + 1 }}>
}

@group(0) @binding(0)
var<storage, read_write> a: BigInt;
@group(0) @binding(1)
var<storage, read_write> b: BigInt;
@group(0) @binding(2)
var<storage, read_write> c: BigIntMediumWide;

fn bigint_add_wide(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> BigIntMediumWide {
    var result: BigIntMediumWide;

    var carry: u32 = 0u;

    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
        let c: u32 = (*lhs).limbs[i] + (*rhs).limbs[i] + carry;

        result.limbs[i] = c & {{ mask }}u;
        carry = c >> {{ log_limb_size }}u;
    }
    result.limbs[{{ num_limbs }}] = carry;

    return result;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var a_bigint = a;
    var b_bigint = b;
    var result: BigIntMediumWide = bigint_add_wide(&a_bigint, &b_bigint);
    c = result;
}
