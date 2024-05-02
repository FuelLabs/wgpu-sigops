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

// From "Efficient Software-Implementation of Finite Fields with Applications
// to Cryptography" by Guajardo, et al, Algorithm 16
// https://www.sandeep.de/my/papers/2006_ActaApplMath_EfficientSoftFiniteF.pdf
fn ff_inverse(
    x: ptr<function, BigInt>,
    p: ptr<function, BigInt>,
) -> BigInt {
    var c: BigInt;
    var b: BigInt;
    b.limbs[0] = 1u;

    var u: BigInt;
    var v: BigInt;
    for (var i = 0u; i < {{ num_limbs }}u; i ++) {
        u.limbs[i] = (*x).limbs[i];
        v.limbs[i] = (*p).limbs[i];
    }

    while (!bigint_is_one(&u) && !bigint_is_one(&v)) {
        while (bigint_is_even(&u)) {
            u = bigint_div2(&u);

            if (bigint_is_even(&b)) {
                b = bigint_div2(&b);
            } else {
                var bp = bigint_add_unsafe(&b, p);
                b = bigint_div2(&bp);
            }
        }

        while (bigint_is_even(&v)) {
            v = bigint_div2(&v);

            if (bigint_is_even(&c)) {
                c = bigint_div2(&c);
            } else {
                var cp = bigint_add_unsafe(&c, p);
                c = bigint_div2(&cp);
            }
        }

        if (bigint_gte(&u, &v)) {
            u = ff_sub(&u, &v, p);
            b = ff_sub(&b, &c, p);
        } else {
            v = ff_sub(&v, &u, p);
            c = ff_sub(&c, &b, p);
        }
    }

    var result: BigInt;
    if (bigint_is_one(&u)) {
        result = b;
    } else {
        result = c;
    }

    if bigint_gte(&result, p) {
        result = bigint_sub(&result, p);
    }

    return result;
}
