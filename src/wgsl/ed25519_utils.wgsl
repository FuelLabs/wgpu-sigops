fn is_negative(val: ptr<function, BigInt>) -> bool {
    return ((*val).limbs[0] & 1u) == 1u;
}

fn conditional_assign(
    a: ptr<function, BigInt>,
    b: ptr<function, BigInt>,
    choice: bool
) -> BigInt {
    if (choice) {
        return *b;
    }
    return *a;
}

fn conditional_negate(
    a: ptr<function, BigInt>,
    p: ptr<function, BigInt>,
    choice: bool
) -> BigInt {
    if (choice) {
        return ff_negate(a, p);
    }
    return *a;
}

fn mont_pow_p58(
    xr: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> BigInt {
    var exponent = get_p58_exponent();
    var r = get_r();

    return modpow(xr, &r, &exponent, p);
}

struct SqrtRatioIResult {
    was_nonzero_square: bool,
    r: BigInt,
}

fn sqrt_ratio_i(
    ur_val: ptr<function, BigInt>,
    vr_val: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> SqrtRatioIResult {
    var ur = *ur_val;
    var vr = *vr_val;

    var v2r = mont_mul(&vr, &vr, p);
    var v3r = mont_mul(&v2r, &vr, p);
    var v6r = mont_mul(&v3r, &v3r, p);
    var v7r = mont_mul(&v6r, &vr, p);

    var uv3r = mont_mul(&ur, &v3r, p);
    var uv7r = mont_mul(&ur, &v7r, p);
    var uv7r_pow_p58 = mont_pow_p58(&uv7r, p);
    var r = mont_mul(&uv3r, &uv7r_pow_p58, p);
    var r2r = mont_mul(&r, &r, p);  
    var check = mont_mul(&vr, &r2r, p);

    var ir = get_sqrt_m1r();

    var neg_ur = ff_negate(&ur, p);
    var neg_uir = mont_mul(&neg_ur, &ir, p);

    var correct_sign_sqrt = bigint_eq(&check, &ur);
    var flipped_sign_sqrt = bigint_eq(&check, &neg_ur);
    var flipped_sign_sqrt_i = bigint_eq(&check, &neg_uir);

    var r_prime = mont_mul(&ir, &r, p);

    r = conditional_assign(&r, &r_prime, flipped_sign_sqrt | flipped_sign_sqrt_i);

    var p_wide = get_p_wide();
    var rinv = get_rinv();
    var mu_fp = get_mu_fp();

    var r_rinv = ff_mul(&r, &rinv, p, &p_wide, &mu_fp);

    var r_is_negative = is_negative(&r_rinv);

    r = conditional_negate(&r, p, r_is_negative);

    var was_nonzero_square = correct_sign_sqrt | flipped_sign_sqrt;

    return SqrtRatioIResult(was_nonzero_square, r);
}
