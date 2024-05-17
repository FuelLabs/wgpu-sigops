fn get_d2r() -> BigInt {
    {{ d2r_bigint }}
    return d2r;
}

fn get_p58_exponent() -> BigInt {
    {{ p58_exponent_bigint }}
    return p58_exponent;
}

fn get_sqrt_m1r() -> BigInt {
    {{ sqrt_m1r_bigint }}
    return sqrt_m1r;
}

fn get_edwards_dr() -> BigInt {
    {{ edwards_dr_bigint }}
    return edwards_dr;
}

fn get_ed25519_generator_xr() -> BigInt {
    {{ ed25519_generator_xr_bigint }}
    return ed25519_generator_xr;
}

fn get_ed25519_generator_yr() -> BigInt {
    {{ ed25519_generator_yr_bigint }}
    return ed25519_generator_yr;
}

fn get_ed25519_generator_tr() -> BigInt {
    {{ ed25519_generator_tr_bigint }}
    return ed25519_generator_tr;
}
