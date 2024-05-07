struct DecodedSig {
    sig: array<u32, 32>,
    is_y_odd: bool
}

fn decode_signature(
    sig_s: ptr<function, array<u32, 32>>
) -> DecodedSig {
    var s: array<u32, 32>;
    for (var i = 0u; i < 32; i ++) {
        s[i] = (*sig_s)[i];
    }
    
    var is_y_odd = (s[0] & 0x80u) != 0;
    s[0] &= 0x7fu;

    /*let is_y_odd = (sig[32] & 0x80) != 0;*/
    /*sig.as_mut()[32] &= 0x7f;*/

    return DecodedSig(s, is_y_odd);
}

fn secp256k1_ecrecover(
    sig_r_bytes: ptr<function, array<u32, 32>>,
    sig_s_bytes: ptr<function, array<u32, 32>>,
    msg: ptr<function, BigInt>,
    p: ptr<function, BigInt>,
    p_wide: ptr<function, BigIntWide>,
    scalar_p: ptr<function, BigInt>,
    scalar_p_wide: ptr<function, BigIntWide>,
    r: ptr<function, BigInt>,
    rinv: ptr<function, BigInt>,
    mu_fp: ptr<function, BigInt>,
    mu_fr: ptr<function, BigInt>,
) -> Point {
    // Assumes that z < Fr::MODULUS and that is_reduced is always false
    var decoded = decode_signature(sig_s_bytes);
    var ds = decoded.sig;
    var is_y_odd = decoded.is_y_odd;

    var sig_r = bytes_be_to_limbs_le(sig_r_bytes);
    var sig_s = bytes_be_to_limbs_le(&ds);

    var z = *msg;
    var r_x = sig_r;

    var rxr = ff_mul(&r_x, r, p, p_wide, mu_fr);
    var yrs = recover_affine_ys_a0(&rxr, p);
    var yr0 = yrs[0];
    var yr1 = yrs[1];

    var y0 = ff_mul(&yr0, rinv, p, p_wide, mu_fp);
    var y1 = ff_mul(&yr1, rinv, p, p_wide, mu_fp);

    // TODO: could checking if yr_0 or yr_1 is odd be optimised?
    var y0_is_odd = !bigint_is_even(&y0);

    var yr: BigInt;

    if (is_y_odd) {
        if (y0_is_odd) {
            yr = yr0;
        } else {
            yr = yr1;
        }
    } else { // y is even
        if (y0_is_odd) {
            yr = yr1;
        } else {
            yr = yr0;
        }
    }

    var recovered_r = Point(rxr, yr, *r);

    // reduce r_x to the scalar field
    if (bigint_gte(&rxr, scalar_p)) {
        rxr = bigint_sub(&rxr, scalar_p);
    }

    // compute inverse(r_x) in the scalar field
    var r_x_inv = ff_inverse(&r_x, scalar_p);

    // compute u1 = -(r_inv * z);
    var r_x_inv_z = ff_mul(&r_x_inv, &z, scalar_p, scalar_p_wide, mu_fr);
    var u1 = ff_negate(&r_x_inv_z, scalar_p);

    // compute u2 = r_inv * s;
    var u2 = ff_mul(&r_x_inv, &sig_s, scalar_p, scalar_p_wide, mu_fr);
    var g = get_secp256k1_generator();

    return jacobian_strauss_shamir_mul(&g, &recovered_r, &u1, &u2, p);
}
