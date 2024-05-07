fn get_secp256k1_generator() -> Point {
    {{ generator_xr_bigint }}
    {{ generator_yr_bigint }}
    var r = get_r();

    return Point(generator_xr, generator_yr, r);
}
