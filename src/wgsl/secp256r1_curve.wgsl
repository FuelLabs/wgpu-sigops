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
