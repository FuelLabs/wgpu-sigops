fn ed25519_verify(
    s: ptr<function, BigInt>,
    k: ptr<function, BigInt>,
    ayr: ptr<function, BigInt>,
    x_sign: bool,
    p: ptr<function, BigInt>,
) -> ETEPoint {
    var g_xr = get_ed25519_generator_xr();
    var g_yr = get_ed25519_generator_yr();
    var g_tr = get_ed25519_generator_tr();
    var g_zr = get_r();
    var g = ETEPoint(g_xr, g_yr, g_tr, g_zr);

    var r = reconstruct_ete_from_y(ayr, x_sign, p);

    var is_valid_y_coord = r.is_valid_y_coord;

    if (!is_valid_y_coord) {
        var empty: ETEPoint;
        return empty;
    }

    var a_pt = r.pt;

    var a_pt_x = a_pt.x;
    var a_pt_t = a_pt.t;
    var neg_a_pt_x = ff_negate(&a_pt_x, p);
    var neg_a_pt_t = ff_negate(&a_pt_t, p);
    var neg_a_pt = ETEPoint(
        neg_a_pt_x,
        a_pt.y,
        neg_a_pt_t,
        a_pt.z,
    );

    return ete_strauss_shamir_mul(&g, &neg_a_pt, s, k, p);
}
