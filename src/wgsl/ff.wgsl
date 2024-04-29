// Requires bigint.wgsl

/*
 * Returns lhs + rhs % p
 */
fn ff_add(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
    p: ptr<function, BigInt>,
) -> BigInt {
    var p_wide: BigIntMediumWide;
    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
        p_wide.limbs[i] = (*p).limbs[i];
    }

    // Compute a + b
    var sum_wide = bigint_wide_add(lhs, rhs);
    
    var result_wide: BigIntMediumWide;
    if (bigint_wide_gte(&sum_wide, &p_wide)) {
        result_wide = bigint_wide_sub(&sum_wide, &p_wide);
    } else {
        result_wide = sum_wide;
    }

    var result: BigInt;

    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
        result.limbs[i] = result_wide.limbs[i];
    }
    return result;
}

/*
 * Returns lhs - rhs % p
 */
fn ff_sub(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
    p: ptr<function, BigInt>,
) -> BigInt {
    if (bigint_gte(lhs, rhs)) {
        return bigint_sub(lhs, rhs);
    } else {
        var r = bigint_sub(rhs, lhs);
        return bigint_sub(p, &r);
    }
}
