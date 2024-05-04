struct BigInt {
    limbs: array<u32, 20>
}

struct BigIntMediumWide {
    limbs: array<u32, 21>
}

fn bigint_wide_add(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> BigIntMediumWide {
    var result: BigIntMediumWide;

    var carry: u32 = 0u;

    for (var i: u32 = 0u; i < 20u; i ++) {
        let c: u32 = (*lhs).limbs[i] + (*rhs).limbs[i] + carry;

        result.limbs[i] = c & 8191u;
        carry = c >> 13u;
    }
    result.limbs[20] = carry;

    return result;
}

fn bigint_add_unsafe(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> BigInt {
    var result: BigInt;

    var carry: u32 = 0u;

    for (var i: u32 = 0u; i < 20u; i ++) {
        let c: u32 = (*lhs).limbs[i] + (*rhs).limbs[i] + carry;

        result.limbs[i] = c & 8191u;
        carry = c >> 13u;
    }

    return result;
}

fn bigint_sub(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> BigInt {
    var borrow: u32 = 0u;

    var result: BigInt;

    for (var i: u32 = 0u; i < 20u; i ++) {
        result.limbs[i] = (*lhs).limbs[i] - (*rhs).limbs[i] - borrow;
        if ((*lhs).limbs[i] < ((*rhs).limbs[i] + borrow)) {
            result.limbs[i] += 8192u;
            borrow = 1u;
        } else {
            borrow = 0u;
        }
    }
    return result;
}

fn bigint_wide_sub(
    lhs: ptr<function, BigIntMediumWide>,
    rhs: ptr<function, BigIntMediumWide>,
) -> BigIntMediumWide {
    var result: BigIntMediumWide;

    var borrow: u32 = 0u;
    for (var i: u32 = 0u; i < 21u; i ++) {
        result.limbs[i] = (*lhs).limbs[i] - (*rhs).limbs[i] - borrow;
        if ((*lhs).limbs[i] < ((*rhs).limbs[i] + borrow)) {
            result.limbs[i] += 8192u;
            borrow = 1u;
        } else {
            borrow = 0u;
        }
    }
    return result;
}

/*
 * Returns true if lhs <= rhs
 */
fn bigint_gte(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> bool {
    for (var i: u32 = 0u; i < 20u; i ++) {
        let idx = 20u - 1u - i;
        if ((*lhs).limbs[idx] < (*rhs).limbs[idx]) {
            return false;
        } else if ((*lhs).limbs[idx] > (*rhs).limbs[idx]) {
            return true;
        }
    }
    return true;
}

/*
 * Returns true if lhs <= rhs
 */
fn bigint_wide_gte(
    lhs: ptr<function, BigIntMediumWide>,
    rhs: ptr<function, BigIntMediumWide>,
) -> bool {
    for (var i: u32 = 0u; i < 21u; i ++) {
        let idx = 21u - 1u - i;
        let l_limb = (*lhs).limbs[idx];
        let r_limb = (*rhs).limbs[idx];

        if (l_limb > r_limb) {
            return true;
        } else if (l_limb < r_limb) {
            return false;
        }
    }
    return true;
}

fn bigint_is_even(
    val: ptr<function, BigInt>
) -> bool {
    return (*val).limbs[0] % 2u == 0u;
}

fn bigint_is_zero(
    val: ptr<function, BigInt>
) -> bool {
    for (var i: u32  = 0u; i < 20u; i ++) {
        if ((*val).limbs[i] != 0u) {
            return false;
        }
    }

    return true;
}
fn bigint_is_one(
    val: ptr<function, BigInt>
) -> bool {
    if ((*val).limbs[0] != 1u) {
        return false;
    }

    for (var i: u32  = 1u; i < 20u; i ++) {
        if ((*val).limbs[i] != 0u) {
            return false;
        }
    }

    return true;
}

fn bigint_div2(
    v: ptr<function, BigInt>
) -> BigInt {
    var result: BigInt;

    var rem = 0u;

    let m = 8192u;

    for (var idx: u32 = 0u; idx < 20u; idx ++) {
        var i = 20u - idx - 1u;

        var d = (*v).limbs[i] + rem * m;
        var q = d / 2u;
        rem = d % 2u;
        result.limbs[i] = q;
    }

    return result;
}
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
    for (var i: u32 = 0u; i < 20u; i ++) {
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

    for (var i: u32 = 0u; i < 20u; i ++) {
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
    for (var i = 0u; i < 20u; i ++) {
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
/*
 * An optimised variant of the Montgomery product algorithm from
 * https://github.com/mitschabaude/montgomery#13-x-30-bit-multiplication.
 */
fn mont_mul(
    x: ptr<function, BigInt>,
    y: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> BigInt {
    var s: BigInt;

    // -------------------------------------------------------------------------------------------
    
    for (var i = 0u; i < 20u; i ++) {
        var t = s.limbs[0] + (*x).limbs[i] * (*y).limbs[0];
        var tprime = t & 8191u;
        var qi = (5425u * tprime) & 8191u;
        var c = (t + qi * (*p).limbs[0]) >> 13u;
        s.limbs[0] = s.limbs[1] + (*x).limbs[i] * (*y).limbs[1] + qi * (*p).limbs[1] + c;

        // Since nSafe = 32 when num_limbs = 20, we can perform the following
        // iterations without performing a carry.
        for (var j = 2u; j < 20u; j ++) {
            s.limbs[j - 1u] = s.limbs[j] + (*x).limbs[i] * (*y).limbs[j] + qi * (*p).limbs[j];
        }
        s.limbs[20u - 2u] = (*x).limbs[i] * (*y).limbs[20u - 1u] + qi * (*p).limbs[20u - 1u];
    }

    // To paraphrase mitschabaude: a last round of carries to ensure that each
    // limb is at most 13u bits.
    var c = 0u;
    for (var i = 0u; i < 20u; i ++) {
        var v = s.limbs[i] + c;
        c = v >> 13u;
        s.limbs[i] = v & 8191u;
    }

    // -------------------------------------------------------------------------------------------
    

    return conditional_reduce(&s, p);
}

fn conditional_reduce(x: ptr<function, BigInt>, y: ptr<function, BigInt>) -> BigInt {
    if (bigint_gte(x, y)) {
        return bigint_sub(x, y);
    }

    return *x;
}

struct SqrtResult {
    a: BigInt,
    b: BigInt
}

fn mont_sqrt_case3mod4(
    xr: ptr<function, BigInt>,
    exponent: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> SqrtResult {
    var r: BigInt = BigInt(array<u32, 20>(7440u, 1u, 1024u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u));
    var a = modpow(xr, &r, exponent, p);
    var b = ff_sub(p, &a, p);
    return SqrtResult(a, b);
}

fn modpow(
    xr: ptr<function, BigInt>,
    r: ptr<function, BigInt>,
    exponent: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> BigInt {
    var result = *r;
    var temp = *xr;
    var s = *exponent;

    while (!bigint_is_zero(&s)) {
        if (!bigint_is_even(&s)) {
            result = mont_mul(&result, &temp, p);
        }
        temp = mont_mul(&temp, &temp, p);
        s = bigint_div2(&s);
    }

    return result;
}

@group(0) @binding(0) var<storage, read_write> xr: BigInt;
@group(0) @binding(1) var<storage, read_write> exponent: BigInt;
@group(0) @binding(2) var<storage, read_write> p: BigInt;
@group(0) @binding(3) var<storage, read_write> result_a: BigInt;
@group(0) @binding(4) var<storage, read_write> result_b: BigInt;


@compute
@workgroup_size(1)
fn test_mont_sqrt_case3mod4(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var xr_bigint = xr;
    var exponent_bigint = exponent;
    var p_bigint = p;
    var result: SqrtResult = mont_sqrt_case3mod4(&xr_bigint, &exponent_bigint, &p_bigint);

    for (var i = 0u; i < 20u; i ++) {
        result_a.limbs[i] = result.a.limbs[i];
        result_b.limbs[i] = result.b.limbs[i];
    }
}
