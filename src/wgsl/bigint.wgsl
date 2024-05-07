struct BigInt {
    limbs: array<u32, {{ num_limbs }}>
}

struct BigIntMediumWide {
    limbs: array<u32, {{ num_limbs + 1 }}>
}

struct BigIntWide {
    limbs: array<u32, {{ num_limbs * 2 }}>
}

fn bigint_wide_add(
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

fn bigint_add_unsafe(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> BigInt {
    var result: BigInt;

    var carry: u32 = 0u;

    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
        let c: u32 = (*lhs).limbs[i] + (*rhs).limbs[i] + carry;

        result.limbs[i] = c & {{ mask }}u;
        carry = c >> {{ log_limb_size }}u;
    }

    return result;
}

fn bigint_sub(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> BigInt {
    var borrow: u32 = 0u;

    var result: BigInt;

    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
        result.limbs[i] = (*lhs).limbs[i] - (*rhs).limbs[i] - borrow;
        if ((*lhs).limbs[i] < ((*rhs).limbs[i] + borrow)) {
            result.limbs[i] += {{ two_pow_word_size }}u;
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
    for (var i: u32 = 0u; i < {{ num_limbs + 1 }}u; i ++) {
        result.limbs[i] = (*lhs).limbs[i] - (*rhs).limbs[i] - borrow;
        if ((*lhs).limbs[i] < ((*rhs).limbs[i] + borrow)) {
            result.limbs[i] += {{ two_pow_word_size }}u;
            borrow = 1u;
        } else {
            borrow = 0u;
        }
    }
    return result;
}

/*
 * Returns true if lhs >= rhs
 */
fn bigint_gte(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> bool {
    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
        let idx = {{ num_limbs }}u - 1u - i;
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

/*
 * Returns true if lhs <= rhs
 */
fn bigint_wide_gte(
    lhs: ptr<function, BigIntMediumWide>,
    rhs: ptr<function, BigIntMediumWide>,
) -> bool {
    for (var i: u32 = 0u; i < {{ num_limbs + 1 }}u; i ++) {
        let idx = {{ num_limbs + 1 }}u - 1u - i;
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
    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
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

    for (var i: u32  = 1u; i < {{ num_limbs }}u; i ++) {
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

    let m = {{ two_pow_word_size }}u;

    for (var idx: u32 = 0u; idx < {{ num_limbs }}u; idx ++) {
        var i = {{ num_limbs }}u - idx - 1u;

        var d = (*v).limbs[i] + rem * m;
        var q = d / 2u;
        rem = d % 2u;
        result.limbs[i] = q;
    }

    return result;
}
