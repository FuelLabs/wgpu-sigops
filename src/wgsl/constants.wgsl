fn get_p() -> BigInt {
    {{ p_bigint }}
    return p;
}

fn get_p_wide() -> BigIntWide {
    {{ p_bigint }}
    var p_wide: BigIntWide;
    for (var i = 0u; i < {{ num_limbs }}u; i ++) {
        p_wide.limbs[i] = p.limbs[i];
    }
    return p_wide;
}

fn get_r() -> BigInt {
    {{ r_bigint }}
    return r;
}

fn get_rinv() -> BigInt {
    {{ rinv_bigint }}
    return rinv;
}

fn get_mu() -> BigInt {
    {{ mu_bigint }}
    return mu;
}

fn get_br() -> BigInt {
    {{ br_bigint }}
    return br;
}

fn get_sqrt_case3mod4_exponent() -> BigInt {
    {{ sqrt_case3mod4_exponent_bigint }}
    return sqrt_case3mod4_exponent;
}

