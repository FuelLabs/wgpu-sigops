struct BigInt {
    limbs: array<u32, {{ num_limbs }}>
}

struct BigIntMediumWide {
    limbs: array<u32, {{ num_limbs + 1 }}>
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
 * Returns true if lhs <= rhs
 */
fn bigint_gte(
    lhs: ptr<function, BigInt>,
    rhs: ptr<function, BigInt>,
) -> bool {
    for (var i: u32 = 0u; i < {{ num_limbs }}u; i ++) {
        let idx = {{ num_limbs }}u - 1u - i;
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
    for (var i: u32 = 0u; i < {{ num_limbs + 1 }}u; i ++) {
        let idx = {{ num_limbs + 1 }}u - 1u - i;
        if ((*lhs).limbs[idx] < (*rhs).limbs[idx]) {
            return false;
        } else if ((*lhs).limbs[idx] > (*rhs).limbs[idx]) {
            return true;
        }
    }
    return true;
}
