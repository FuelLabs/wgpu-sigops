struct Point {
    x: BigInt,
    y: BigInt,
    z: BigInt
}

fn mont_mul_neg_3(
    val: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> BigInt {
    var val2 = ff_add(val, val, p);
    var val3 = ff_add(&val2, val, p);
    var neg_val_3 = ff_negate(&val3, p);
    return neg_val_3;
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-shortw-projective-3.html#addition-add-2015-rcb
fn projective_add_2015_rcb_unsafe(
    a: ptr<function, Point>,
    b: ptr<function, Point>,
    p: ptr<function, BigInt>
) -> Point {
    var x1 = (*a).x;
    var y1 = (*a).y;
    var z1 = (*a).z;
    var x2 = (*b).x;
    var y2 = (*b).y;
    var z2 = (*b).z;

    if (bigint_is_zero(&x1) && bigint_is_zero(&z1)) {
        return *b;
    } else if (bigint_is_zero(&x2) && bigint_is_zero(&z2)) {
        return *a;
    }

    var b3 = get_br3();

    var t0 = mont_mul(&x1, &x2, p);
    var t1 = mont_mul(&y1, &y2, p);
    var t2 = mont_mul(&z1, &z2, p);
    var t3 = ff_add(&x1, &y1, p);
    var t4 = ff_add(&x2, &y2, p);
    t3 = mont_mul(&t3, &t4, p);
    t4 = ff_add(&t0, &t1, p);
    t3 = ff_sub(&t3, &t4, p);
    t4 = ff_add(&x1, &z1, p);
    var t5 = ff_add(&x2, &z2, p);
    t4 = mont_mul(&t4, &t5, p);
    t5 = ff_add(&t0, &t2, p);
    t4 = ff_sub(&t4, &t5, p);
    t5 = ff_add(&y1, &z1, p);
    var x3 = ff_add(&y2, &z2, p);
    t5 = mont_mul(&t5, &x3, p);
    x3 = ff_add(&t1, &t2, p);
    t5 = ff_sub(&t5, &x3, p);
    var z3 = mont_mul_neg_3(&t4, p);
    x3 = mont_mul(&b3, &t2, p);
    z3 = ff_add(&x3, &z3, p);
    x3 = ff_sub(&t1, &z3, p);
    z3 = ff_add(&t1, &z3, p);
    var y3 = mont_mul(&x3, &z3, p);
    t1 = ff_add(&t0, &t0, p);
    t1 = ff_add(&t1, &t0, p);
    t2 = mont_mul_neg_3(&t2, p);
    t4 = mont_mul(&b3, &t4, p);
    t1 = ff_add(&t1, &t2, p);
    t2 = ff_sub(&t0, &t2, p);
    t2 = mont_mul_neg_3(&t2, p);
    t4 = ff_add(&t4, &t2, p);
    t0 = mont_mul(&t1, &t4, p);
    y3 = ff_add(&y3, &t0, p);
    t0 = mont_mul(&t5, &t4, p);
    x3 = mont_mul(&t3, &x3, p);
    x3 = ff_sub(&x3, &t0, p);
    t0 = mont_mul(&t3, &t1, p);
    z3 = mont_mul(&t5, &z3, p);
    z3 = ff_add(&z3, &t0, p);

    return Point(x3, y3, z3);
}

/// https://www.hyperelliptic.org/EFD/g1p/auto-shortw-projective-3.html#doubling-dbl-2015-rcb
fn projective_dbl_2015_rcb(
    a: ptr<function, Point>,
    p: ptr<function, BigInt>
) -> Point {
    var x1 = (*a).x;
    var y1 = (*a).y;
    var z1 = (*a).z;

    if (bigint_is_zero(&x1) && bigint_is_zero(&z1)) {
        return *a;
    }

    var b3 = get_br3();

    var t0 = mont_mul(&x1, &x1, p);
    var t1 = mont_mul(&y1, &y1, p);
    var t2 = mont_mul(&z1, &z1, p);
    var t3 = mont_mul(&x1, &y1, p);
    t3 = ff_add(&t3, &t3, p);
    var z3 = mont_mul(&x1, &z1, p);
    z3 = ff_add(&z3, &z3, p);
    var x3 = mont_mul_neg_3(&z3, p);
    var y3 = mont_mul(&b3, &t2, p);
    y3 = ff_add(&x3, &y3, p);
    x3 = ff_sub(&t1, &y3, p);
    y3 = ff_add(&t1, &y3, p);
    y3 = mont_mul(&x3, &y3, p);
    x3 = mont_mul(&t3, &x3, p);
    z3 = mont_mul(&b3, &z3, p);
    t2 = mont_mul_neg_3(&t2, p);
    t3 = ff_sub(&t0, &t2, p);
    t3 = mont_mul_neg_3(&t3, p);
    t3 = ff_add(&t3, &z3, p);
    z3 = ff_add(&t0, &t0, p);
    t0 = ff_add(&z3, &t0, p);
    t0 = ff_add(&t0, &t2, p);
    t0 = mont_mul(&t0, &t3, p);
    y3 = ff_add(&y3, &t0, p);
    t2 = mont_mul(&y1, &z1, p);
    t2 = ff_add(&t2, &t2, p);
    t0 = mont_mul(&t2, &t3, p);
    x3 = ff_sub(&x3, &t0, p);
    z3 = mont_mul(&t2, &t1, p);
    z3 = ff_add(&z3, &z3, p);
    z3 = ff_add(&z3, &z3, p);

    return Point(x3, y3, z3);
}

/*
 * Return the two possible Y-coordinates of an affine point, given its X-coordinate
 * a = 3
 */
fn secp256r1_recover_affine_ys(
    xr: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> array<BigInt, 2> {
    var xr_squared = mont_mul(xr, xr, p);
    var xr_cubed = mont_mul(&xr_squared, xr, p);

    var xr2 = ff_add(xr, xr, p);
    var xr3 = ff_add(&xr2, xr, p);
    var axr = ff_negate(&xr3, p);
    var xr_cubed_plus_ar = ff_add(&xr_cubed, &axr, p);

    var br = get_br();
    var xr_cubed_plus_ar_plus_br = ff_add(&xr_cubed_plus_ar, &br, p);

    var ys = mont_sqrt_case3mod4(&xr_cubed_plus_ar_plus_br, p);

    return ys;
}
