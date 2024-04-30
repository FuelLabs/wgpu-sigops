struct Point {
    x: BigInt,
    y: BigInt,
    z: BigInt
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
