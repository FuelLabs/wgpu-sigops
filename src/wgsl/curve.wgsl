struct Point {
    x: BigInt,
    y: BigInt,
    z: BigInt
}

fn projective_add_2007_bl_unsafe(
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

    var u1 = mont_mul(&x1, &z2, p);
    var u2 = mont_mul(&x2, &z1, p);
    var s1 = mont_mul(&y1, &z2, p);
    var s2 = mont_mul(&y2, &z1, p);
    var zz = mont_mul(&z1, &z2, p);
    var t = ff_add(&u1, &u2, p);
    var tt = mont_mul(&t, &t, p);
    var m = ff_add(&s1, &s2, p);
    var u1u2 = mont_mul(&u1, &u2, p);
    var r = ff_sub(&tt, &u1u2, p);
    var f = mont_mul(&zz, &m, p);
    var l = mont_mul(&m, &f, p);
    var ll = mont_mul(&l, &l, p);
    var ttll = ff_add(&tt, &ll, p);
    var tl = ff_add(&t, &l, p);
    var tl2 = mont_mul(&tl, &tl, p);
    var g = ff_sub(&tl2, &ttll, p);
    var r2 = mont_mul(&r, &r, p);
    var r22 = ff_add(&r2, &r2, p);
    var w = ff_sub(&r22, &g, p);
    var f2 = ff_add(&f, &f, p);
    var x3 = mont_mul(&f2, &w, p);
    var ll2 = ff_add(&ll, &ll, p);
    var w2 = ff_add(&w, &w, p);
    var g2w = ff_sub(&g, &w2, p);
    var rg2w = mont_mul(&r, &g2w, p);
    var y3 = ff_sub(&rg2w, &ll2, p);
    var ff = mont_mul(&f, &f, p);
    var f4 = ff_add(&f2, &f2, p);
    var z3 = mont_mul(&f4, &ff, p);

    return Point(x3, y3, z3);
}

fn projective_dbl_2007_bl_unsafe(
    pt: ptr<function, Point>,
    p: ptr<function, BigInt>
) -> Point {
    var x1 = (*pt).x;
    var y1 = (*pt).y;
    var z1 = (*pt).z;

    var xx = mont_mul(&x1, &x1, p);
    var xx2 = ff_add(&xx, &xx, p);
    var w = ff_add(&xx2, &xx, p);
    var y1z1 = mont_mul(&y1, &z1, p);
    var s = ff_add(&y1z1, &y1z1, p);
    var ss = mont_mul(&s, &s, p);
    var sss = mont_mul(&s, &ss, p);
    var r = mont_mul(&y1, &s, p);
    var rr = mont_mul(&r, &r, p);
    var xxrr = ff_add(&xx, &rr, p);
    var x1r = ff_add(&x1, &r, p);
    var x1r2 = mont_mul(&x1r, &x1r, p);
    var b = ff_sub(&x1r2, &xxrr, p);
    var b2 = ff_add(&b, &b, p);
    var w2 = mont_mul(&w, &w, p);
    var h = ff_sub(&w2, &b2, p);
    var x3 = mont_mul(&h, &s, p);
    var rr2 = ff_add(&rr, &rr, p);
    var bh = ff_sub(&b, &h, p);
    var wbh = mont_mul(&w, &bh, p);
    var y3 = ff_sub(&wbh, &rr2, p);
    var z3 = sss;

    return Point(x3, y3, z3);
}

fn jacobian_negate(
    a: ptr<function, Point>,
    p: ptr<function, BigInt>
) -> Point {
    var x = (*a).x;
    var y = (*a).y;
    var z = (*a).z;

    y = ff_sub(p, &y, p);

    Point(x, y, z);
}

fn jacobian_add_2007_bl_unsafe(
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

    var z1z1 = mont_mul(&z1, &z1, p);
    var z2z2 = mont_mul(&z2, &z2, p);
    var u1 = mont_mul(&x1, &z2z2, p);
    var u2 = mont_mul(&x2, &z1z1, p);
    var y1z2 = mont_mul(&y1, &z2, p);
    var s1 = mont_mul(&y1z2, &z2z2, p);
    var y2z1 = mont_mul(&y2, &z1, p);
    var s2 = mont_mul(&y2z1, &z1z1, p);
    var h = ff_sub(&u2, &u1, p);
    var h2 = ff_add(&h, &h, p);
    var i = mont_mul(&h2, &h2, p);
    var j = mont_mul(&h, &i, p);
    var s2s1 = ff_sub(&s2, &s1, p);
    var r = ff_add(&s2s1, &s2s1, p);
    var v = mont_mul(&u1, &i, p);
    var v2 = ff_add(&v, &v, p);
    var r2 = mont_mul(&r, &r, p);
    var jv2 = ff_add(&j, &v2, p);
    var x3 = ff_sub(&r2, &jv2, p);
    var vx3 = ff_sub(&v, &x3, p);
    var rvx3 = mont_mul(&r, &vx3, p);
    var s12 = ff_add(&s1, &s1, p);
    var s12j = mont_mul(&s12, &j, p);
    var y3 = ff_sub(&rvx3, &s12j, p);
    var z1z2 = mont_mul(&z1, &z2, p);
    var z1z2h = mont_mul(&z1z2, &h, p);
    var z3 = ff_add(&z1z2h, &z1z2h, p);

    return Point(x3, y3, z3);
}

fn jacobian_dbl_2009_l(
    pt: ptr<function, Point>,
    p: ptr<function, BigInt>
) -> Point {
    var x = (*pt).x;
    var y = (*pt).y;
    var z = (*pt).z;

    var a = mont_mul(&x, &x, p);
    var b = mont_mul(&y, &y, p);
    var c = mont_mul(&b, &b, p);
    var x1b = ff_add(&x, &b, p);
    var x1b2 = mont_mul(&x1b, &x1b, p);
    var ac = ff_add(&a, &c, p);
    var x1b2ac = ff_sub(&x1b2, &ac, p);
    var d = ff_add(&x1b2ac, &x1b2ac, p);
    var a2 = ff_add(&a, &a, p);
    var e = ff_add(&a2, &a, p);
    var f = mont_mul(&e, &e, p);
    var d2 = ff_add(&d, &d, p);
    var x3 = ff_sub(&f, &d2, p);
    var c2 = ff_add(&c, &c, p);
    var c4 = ff_add(&c2, &c2, p);
    var c8 = ff_add(&c4, &c4, p);
    var dx3 = ff_sub(&d, &x3, p);
    var edx3 = mont_mul(&e, &dx3, p);
    var y3 = ff_sub(&edx3, &c8, p);
    var y1z1 = mont_mul(&y, &z, p);
    var z3 = ff_add(&y1z1, &y1z1, p);

    return Point(x3, y3, z3);
}

/*
 * Return the two possible Y-coordinates of an affine point, given its X-coordinate
 */
fn recover_affine_ys_a0(
    xr: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> array<BigInt, 2> {
    // Assumes that a = 0
    var xr_squared = mont_mul(xr, xr, p);
    var xr_cubed = mont_mul(&xr_squared, xr, p);

    var br = get_br();
    var xr_cubed_plus_b = ff_add(&xr_cubed, &br, p);

    var ys = mont_sqrt_case3mod4(&xr_cubed_plus_b, p);

    return ys;
}

/*
 * Scalar multiplication using double-and-add
 */
fn jacobian_mul(
    pt: ptr<function, Point>,
    x: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> Point {
    var zero: BigInt;
    var one: BigInt;
    one.limbs[0] = 1u;

    var result = Point(one, one, zero);
    var result_is_inf = true;

    var s = *x;
    var temp = *pt;

    while (!bigint_is_zero(&s)) {
        if (!bigint_is_even(&s)) {
            if (result_is_inf) {
                // This check is needed to prevent jacobian_add_2007_bl_unsafe
                // from getting the point at infinity as an input.
                result = temp;
                result_is_inf = false;
            } else {
                result = jacobian_add_2007_bl_unsafe(&result, &temp, p);
            }
        }
        temp = jacobian_dbl_2009_l(&temp, p);
        s = bigint_div2(&s);
    }

    return result;
}

struct BitsResult {
    bits: array<bool, 256>,
    num_bits: u32
}

/*
 * Calculate the binary expansion of x, which must not be in Montgomery form.
 * Supports up to 256 bits.
 */
fn bigint_to_bits_le(
    x: ptr<function, BigInt>
) -> BitsResult {
    var bits: array<bool, 256>;
    var num_bits = 0u;

    var s = *x;

    while (!bigint_is_zero(&s)) {
        if (!bigint_is_even(&s)) {
            bits[num_bits] = true;
        }
        s = bigint_div2(&s);
        num_bits += 1u;
    }

    return BitsResult(bits, num_bits);
}

/*
 * Determine ax + by where x and y are scalars and a and b are points.
 * x and y must not be in Montgomery form.
 */
fn jacobian_strauss_shamir_mul(
    a: ptr<function, Point>,
    b: ptr<function, Point>,
    x: ptr<function, BigInt>,
    y: ptr<function, BigInt>,
    p: ptr<function, BigInt>
) -> Point {
    // From https://github.com/mratsim/constantine/issues/36
    var zero: BigInt;
    var one: BigInt;
    one.limbs[0] = 1u;

    var result = Point(one, one, zero);
    var result_is_inf = true;

    var s0 = *x;
    var s1 = *y;

    // Compute the bit decomposition of the scalars
    var s0_bitsresult = bigint_to_bits_le(&s0);
    var s1_bitsresult = bigint_to_bits_le(&s1);

    // Precompute a + b
    var ab = jacobian_add_2007_bl_unsafe(a, b, p);
    var point_to_add: Point;

    // Determine the length of the longest bitstring to avoid doing more loop iterations than necessary
    var max_bits = max(s0_bitsresult.num_bits, s1_bitsresult.num_bits);

    for (var idx = 0u; idx < max_bits; idx ++) {
        var i = max_bits - 1u - idx;

        let a_bit = s0_bitsresult.bits[i];
        let b_bit = s1_bitsresult.bits[i];

        if (!result_is_inf) {
            result = jacobian_dbl_2009_l(&result, p);
        }

        if (a_bit && !b_bit) {
            point_to_add = *a;
        } else if (!a_bit && b_bit) {
            point_to_add = *b;
        } else if (a_bit && b_bit) {
            point_to_add = ab;
        }

        if (a_bit || b_bit) {
            if (result_is_inf) {
                // Assign instead of adding point_to_add to the point at
                // infinity, which jacobian_add_2007_bl_unsafe doesn't support
                result = point_to_add;
                result_is_inf = false;
            } else {
                result = jacobian_add_2007_bl_unsafe(&result, &point_to_add, p);
            }
        }
    }

    return result;
}
