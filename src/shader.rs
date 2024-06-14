use crate::tests::{get_ed25519_d2, get_secp256k1_b, get_secp256r1_b};
use ark_ec::twisted_edwards::TECurveConfig;
use ark_ec::AffineRepr;
use ark_ed25519::EdwardsAffine;
use ark_ff::{BigInteger, Field, PrimeField};
use minijinja::{context, Environment, Template};
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, ff, mont, utils::calc_bitwidth};
use num_bigint::BigUint;
use std::path::PathBuf;

fn read_from_file(path: &str, file: &str) -> String {
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(path)
        .join(file);
    std::fs::read_to_string(&input_path).unwrap()
}

pub fn gen_constant_bigint(
    var_name: &str,
    val: &BigUint,
    num_limbs: usize,
    log_limb_size: u32,
) -> String {
    let r_limbs = bigint::from_biguint_le(val, num_limbs, log_limb_size);
    let mut result = format!(
        "var {}: BigInt = BigInt(array<u32, {}>(",
        var_name, num_limbs
    )
    .to_owned();

    for i in 0..num_limbs {
        result.push_str(format!("{}u", r_limbs[i]).as_str());
        if i < num_limbs - 1 {
            result.push_str(", ");
        }
    }

    result.push_str("));");
    result
}

pub fn do_render(
    p: &BigUint,
    scalar_p: &BigUint,
    b: &BigUint,
    log_limb_size: u32,
    template: &Template,
) -> String {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let two_pow_word_size = 2u32.pow(log_limb_size);
    let mask = two_pow_word_size - 1u32;
    let nsafe = mont::calc_nsafe(log_limb_size);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;
    let n0 = res.1;

    let p_bitlength = calc_bitwidth(&p);
    let slack = num_limbs * log_limb_size as usize - p_bitlength;

    let r_bigint = gen_constant_bigint("r", &(&r % p), num_limbs, log_limb_size);
    let rinv_bigint = gen_constant_bigint("rinv", &(&rinv % p), num_limbs, log_limb_size);
    let p_bigint = gen_constant_bigint("p", p, num_limbs, log_limb_size);
    let scalar_p_bigint = gen_constant_bigint("scalar_p", scalar_p, num_limbs, log_limb_size);

    let br = b * &r % p;
    let br_bigint = gen_constant_bigint("br", &br, num_limbs, log_limb_size);

    let br3 = (BigUint::from(3u32) * b * &r) % p;
    let br3_bigint = gen_constant_bigint("br3", &br3, num_limbs, log_limb_size);

    let mu_fp_bigint = gen_constant_bigint("mu_fp", &ff::gen_mu(&p), num_limbs, log_limb_size);
    let mu_fr_bigint =
        gen_constant_bigint("mu_fr", &ff::gen_mu(&scalar_p), num_limbs, log_limb_size);

    let secp256k1_generator_x =
        BigUint::from_bytes_be(&ark_secp256k1::G_GENERATOR_X.into_bigint().to_bytes_be());
    let secp256k1_generator_y =
        BigUint::from_bytes_be(&ark_secp256k1::G_GENERATOR_Y.into_bigint().to_bytes_be());
    let secp256k1_generator_xr = secp256k1_generator_x * &r % p;
    let secp256k1_generator_yr = secp256k1_generator_y * &r % p;
    let secp256k1_generator_xr_bigint = gen_constant_bigint(
        "secp256k1_generator_xr",
        &secp256k1_generator_xr,
        num_limbs,
        log_limb_size,
    );
    let secp256k1_generator_yr_bigint = gen_constant_bigint(
        "secp256k1_generator_yr",
        &secp256k1_generator_yr,
        num_limbs,
        log_limb_size,
    );

    let secp256r1_generator_x =
        BigUint::from_bytes_be(&ark_secp256r1::G_GENERATOR_X.into_bigint().to_bytes_be());
    let secp256r1_generator_y =
        BigUint::from_bytes_be(&ark_secp256r1::G_GENERATOR_Y.into_bigint().to_bytes_be());
    let secp256r1_generator_xr = secp256r1_generator_x * &r % p;
    let secp256r1_generator_yr = secp256r1_generator_y * &r % p;
    let secp256r1_generator_xr_bigint = gen_constant_bigint(
        "secp256r1_generator_xr",
        &secp256r1_generator_xr,
        num_limbs,
        log_limb_size,
    );
    let secp256r1_generator_yr_bigint = gen_constant_bigint(
        "secp256r1_generator_yr",
        &secp256r1_generator_yr,
        num_limbs,
        log_limb_size,
    );

    let sqrt_case3mod4_exponent = (p + BigUint::from(1u32)) / BigUint::from(4u32);
    let sqrt_case3mod4_exponent_bigint = gen_constant_bigint(
        "sqrt_case3mod4_exponent",
        &sqrt_case3mod4_exponent,
        num_limbs,
        log_limb_size,
    );

    let context = context! {
        num_limbs => num_limbs,
        log_limb_size => log_limb_size,
        two_pow_word_size => two_pow_word_size,
        mask => mask,
        nsafe => nsafe,
        n0 => n0,
        slack => slack,
        r_bigint => r_bigint,
        rinv_bigint => rinv_bigint,
        p_bigint => p_bigint,
        scalar_p_bigint => scalar_p_bigint,
        br_bigint => br_bigint,
        br3_bigint => br3_bigint,
        mu_fp_bigint => mu_fp_bigint,
        mu_fr_bigint => mu_fr_bigint,
        secp256k1_generator_xr_bigint => secp256k1_generator_xr_bigint,
        secp256k1_generator_yr_bigint => secp256k1_generator_yr_bigint,
        secp256r1_generator_xr_bigint => secp256r1_generator_xr_bigint,
        secp256r1_generator_yr_bigint => secp256r1_generator_yr_bigint,
        sqrt_case3mod4_exponent_bigint => sqrt_case3mod4_exponent_bigint,
    };
    template.render(context).unwrap()
}

pub fn render_buffer_test(
    template_file: &str,
) -> String {
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let source = read_from_file(tests_path, template_file);
    env.add_template(template_file, &source).unwrap();

    let template = env.get_template(template_file).unwrap();
    let context = context! {};
    template.render(context).unwrap()
}

use std::borrow::Cow;

pub fn add_source_to_env(
    template_path: &str,
    template_file: &str,
    env: &mut Environment,
) {
    let source = read_from_file(template_path, &template_file);
    let tf = Cow::from(template_file);
    let s = Cow::from(source);
    env.add_template_owned(tf.into_owned(), s.into_owned()).unwrap();
}

pub fn render_bytes_to_limbs_test(
    template_file: &str,
    p: &BigUint,
    b: &BigUint,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "bytes_be_to_limbs_le.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(p, p, b, log_limb_size, &template)
}

pub fn render_limbs_to_u32s_test(
    template_file: &str,
    p: &BigUint,
    b: &BigUint,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "limbs_le_to_u32s_be.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(p, p, b, log_limb_size, &template)
}

pub fn render_bigint_ff_mont_tests(
    template_file: &str,
    p: &BigUint,
    b: &BigUint,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";
    let mut env = Environment::new();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(p, p, b, log_limb_size, &template)
}

pub fn render_mont_sqrt_case3mod4_test(
    template_file: &str,
    p: &BigUint,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let b = get_secp256k1_b();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "secp256k1_curve.wgsl", &mut env);
    add_source_to_env(template_path, "secp_constants.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(&p, &p, &b, log_limb_size, &template)
}

pub fn render_secp256k1_curve_tests(
    template_file: &str,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let p = crate::moduli::secp256k1_fq_modulus_biguint();
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();

    let b = get_secp256k1_b();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "secp256k1_curve.wgsl", &mut env);
    add_source_to_env(template_path, "secp_constants.wgsl", &mut env);
    add_source_to_env(template_path, "secp_curve_utils.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(&p, &scalar_p, &b, log_limb_size, &template)
}

pub fn render_secp256k1_ecdsa_tests(
    template_file: &str,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let b = get_secp256k1_b();
    let p = crate::moduli::secp256k1_fq_modulus_biguint();
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "secp256k1_curve.wgsl", &mut env);
    add_source_to_env(template_path, "signature.wgsl", &mut env);
    add_source_to_env(template_path, "secp256k1_ecdsa.wgsl", &mut env);
    add_source_to_env(template_path, "secp_constants.wgsl", &mut env);
    add_source_to_env(template_path, "secp_curve_utils.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(template_path, "secp256k1_curve_generators.wgsl", &mut env);
    add_source_to_env(template_path, "bytes_be_to_limbs_le.wgsl", &mut env);
    add_source_to_env(template_path, "limbs_le_to_u32s_be.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(&p, &scalar_p, &b, log_limb_size, &template)
}

pub fn render_secp256r1_curve_tests(
    template_file: &str,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let p = crate::moduli::secp256r1_fq_modulus_biguint();
    let scalar_p = crate::moduli::secp256r1_fr_modulus_biguint();
    let b = get_secp256r1_b();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "secp256r1_curve.wgsl", &mut env);
    add_source_to_env(template_path, "secp_constants.wgsl", &mut env);
    add_source_to_env(template_path, "secp_curve_utils.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(&p, &scalar_p, &b, log_limb_size, &template)
}

pub fn render_secp256r1_ecdsa_tests(
    template_file: &str,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let p = crate::moduli::secp256r1_fq_modulus_biguint();
    let scalar_p = crate::moduli::secp256r1_fr_modulus_biguint();
    let b = get_secp256r1_b();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "secp256r1_curve.wgsl", &mut env);
    add_source_to_env(template_path, "signature.wgsl", &mut env);
    add_source_to_env(template_path, "secp256r1_ecdsa.wgsl", &mut env);
    add_source_to_env(template_path, "secp_constants.wgsl", &mut env);
    add_source_to_env(template_path, "secp_curve_utils.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(template_path, "secp256r1_curve_generators.wgsl", &mut env);
    add_source_to_env(template_path, "bytes_be_to_limbs_le.wgsl", &mut env);
    add_source_to_env(template_path, "limbs_le_to_u32s_be.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render(&p, &scalar_p, &b, log_limb_size, &template)
}

pub fn render_ed25519_curve_tests(
    template_file: &str,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let p = crate::moduli::ed25519_fq_modulus_biguint();
    let scalar_p = crate::moduli::ed25519_fr_modulus_biguint();
    let d2 = get_ed25519_d2();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_curve.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_constants.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render_ed25519(&p, &scalar_p, &d2, log_limb_size, &template)
}

pub fn do_render_ed25519(
    p: &BigUint,
    scalar_p: &BigUint,
    d2: &BigUint,
    log_limb_size: u32,
    template: &Template,
) -> String {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let two_pow_word_size = 2u32.pow(log_limb_size);
    let mask = two_pow_word_size - 1u32;
    let nsafe = mont::calc_nsafe(log_limb_size);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;
    let n0 = res.1;

    let p_bitlength = calc_bitwidth(&p);
    let slack = num_limbs * log_limb_size as usize - p_bitlength;

    let r_bigint = gen_constant_bigint("r", &(&r % p), num_limbs, log_limb_size);
    let rinv_bigint = gen_constant_bigint("rinv", &(&rinv % p), num_limbs, log_limb_size);
    let p_bigint = gen_constant_bigint("p", p, num_limbs, log_limb_size);
    let scalar_p_bigint = gen_constant_bigint("scalar_p", scalar_p, num_limbs, log_limb_size);

    let d2r = d2 * &r % p;
    let d2r_bigint = gen_constant_bigint("d2r", &d2r, num_limbs, log_limb_size);

    let mu_fp_bigint = gen_constant_bigint("mu_fp", &ff::gen_mu(&p), num_limbs, log_limb_size);
    let mu_fr_bigint =
        gen_constant_bigint("mu_fr", &ff::gen_mu(&scalar_p), num_limbs, log_limb_size);

    let p58_exponent = BigUint::parse_bytes(
        b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd",
        16,
    )
    .unwrap();
    assert_eq!(
        p58_exponent,
        (p - BigUint::from(5u32)) / BigUint::from(8u32)
    );
    let p58_exponent_bigint =
        gen_constant_bigint("p58_exponent", &p58_exponent, num_limbs, log_limb_size);

    let sqrt_m1 = ark_ed25519::Fq::from(-1i32).sqrt().unwrap();
    let sqrt_m1_bigint: BigUint = sqrt_m1.into_bigint().into();
    let sqrt_m1r_bigint = gen_constant_bigint(
        "sqrt_m1r",
        &(sqrt_m1_bigint * &r % p),
        num_limbs,
        log_limb_size,
    );

    let edwards_dr: BigUint = ark_ed25519::EdwardsConfig::COEFF_D.into_bigint().into();
    let edwards_dr_bigint = gen_constant_bigint(
        "edwards_dr",
        &(edwards_dr * &r % p),
        num_limbs,
        log_limb_size,
    );

    let generator = EdwardsAffine::generator();
    let ed25519_generator_x = BigUint::from_bytes_be(&generator.x.into_bigint().to_bytes_be());
    let ed25519_generator_y = BigUint::from_bytes_be(&generator.y.into_bigint().to_bytes_be());
    let ed25519_generator_xr = &ed25519_generator_x * &r % p;
    let ed25519_generator_yr = &ed25519_generator_y * &r % p;
    let ed25519_generator_tr = (&ed25519_generator_x * &ed25519_generator_y) * &r % p;
    let ed25519_generator_xr_bigint = gen_constant_bigint(
        "ed25519_generator_xr",
        &ed25519_generator_xr,
        num_limbs,
        log_limb_size,
    );
    let ed25519_generator_yr_bigint = gen_constant_bigint(
        "ed25519_generator_yr",
        &ed25519_generator_yr,
        num_limbs,
        log_limb_size,
    );
    let ed25519_generator_tr_bigint = gen_constant_bigint(
        "ed25519_generator_tr",
        &ed25519_generator_tr,
        num_limbs,
        log_limb_size,
    );

    let (fr_reduce_r_limbs_array, scalar_p_limbs_array) = gen_ed25519_reduce_fr_constants(scalar_p);

    let context = context! {
        num_limbs => num_limbs,
        log_limb_size => log_limb_size,
        two_pow_word_size => two_pow_word_size,
        mask => mask,
        nsafe => nsafe,
        n0 => n0,
        slack => slack,
        r_bigint => r_bigint,
        rinv_bigint => rinv_bigint,
        p_bigint => p_bigint,
        scalar_p_bigint => scalar_p_bigint,
        d2r_bigint => d2r_bigint,
        mu_fp_bigint => mu_fp_bigint,
        mu_fr_bigint => mu_fr_bigint,
        p58_exponent_bigint => p58_exponent_bigint,
        sqrt_m1r_bigint => sqrt_m1r_bigint,
        edwards_dr_bigint => edwards_dr_bigint,
        ed25519_generator_xr_bigint => ed25519_generator_xr_bigint,
        ed25519_generator_yr_bigint => ed25519_generator_yr_bigint,
        ed25519_generator_tr_bigint => ed25519_generator_tr_bigint,
        scalar_p_limbs_array => scalar_p_limbs_array,
        fr_reduce_r_limbs_array => fr_reduce_r_limbs_array,
    };

    template.render(context).unwrap()
}

pub fn render_ed25519_reduce_fr_tests(template_file: &str) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let scalar_p = crate::moduli::ed25519_fr_modulus_biguint();

    add_source_to_env(template_path, "ed25519_reduce_fr.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render_ed25519_reduce_fr_tests(&scalar_p, &template)
}

pub fn gen_ed25519_reduce_fr_constants(scalar_p: &BigUint) -> (String, String) {
    let r = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffeb2106215d086329a7ed9ce5a30a2c131b",
        16,
    )
    .unwrap();
    let r_bytes = multiprecision::utils::biguint_to_bytes_be(&r, 34);
    let r_limbs = multiprecision::reduce::bytes_34_to_limbs_32(&r_bytes);

    let mut fr_reduce_r_limbs_array = format!("var fr_reduce_r_limbs = array<u32, 32>(").to_owned();
    for i in 0..r_limbs.len() {
        fr_reduce_r_limbs_array.push_str(format!("{}u", r_limbs[i]).as_str());
        if i < r_limbs.len() - 1 {
            fr_reduce_r_limbs_array.push_str(", ");
        }
    }
    fr_reduce_r_limbs_array.push_str(");");

    let scalar_p_bytes = multiprecision::utils::biguint_to_bytes_be(scalar_p, 34);
    let scalar_p_limbs = multiprecision::reduce::bytes_34_to_limbs_32(&scalar_p_bytes);

    let mut scalar_p_limbs_array = format!("var scalar_p_limbs = array<u32, 32>(").to_owned();
    for i in 0..scalar_p_limbs.len() {
        scalar_p_limbs_array.push_str(format!("{}u", scalar_p_limbs[i]).as_str());
        if i < scalar_p_limbs.len() - 1 {
            scalar_p_limbs_array.push_str(", ");
        }
    }
    scalar_p_limbs_array.push_str(");");

    (fr_reduce_r_limbs_array, scalar_p_limbs_array)
}

pub fn do_render_ed25519_reduce_fr_tests(scalar_p: &BigUint, template: &Template) -> String {
    let (fr_reduce_r_limbs_array, scalar_p_limbs_array) = gen_ed25519_reduce_fr_constants(scalar_p);

    let context = context! {
        scalar_p_limbs_array => scalar_p_limbs_array,
        fr_reduce_r_limbs_array => fr_reduce_r_limbs_array,
    };

    template.render(context).unwrap()
}

pub fn render_ed25519_utils_tests(
    template_file: &str,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let p = crate::moduli::ed25519_fq_modulus_biguint();
    let scalar_p = crate::moduli::ed25519_fr_modulus_biguint();
    let d2 = get_ed25519_d2();

add_source_to_env(template_path, "bigint.wgsl", &mut env);

    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_curve.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_constants.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_utils.wgsl", &mut env);
    add_source_to_env(template_path, "bytes_be_to_limbs_le.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render_ed25519(&p, &scalar_p, &d2, log_limb_size, &template)
}

pub fn render_ed25519_eddsa_tests(
    template_file: &str,
    log_limb_size: u32,
) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    let p = crate::moduli::ed25519_fq_modulus_biguint();
    let scalar_p = crate::moduli::ed25519_fr_modulus_biguint();
    let d2 = get_ed25519_d2();

    add_source_to_env(template_path, "bigint.wgsl", &mut env);
    add_source_to_env(template_path, "ff.wgsl", &mut env);
    add_source_to_env(template_path, "mont.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_curve.wgsl", &mut env);
    add_source_to_env(template_path, "constants.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_constants.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_utils.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_eddsa.wgsl", &mut env);
    add_source_to_env(template_path, "bytes_be_to_limbs_le.wgsl", &mut env);
    add_source_to_env(template_path, "sha512.wgsl", &mut env);
    add_source_to_env(template_path, "ed25519_reduce_fr.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let template = env.get_template(template_file).unwrap();
    do_render_ed25519(&p, &scalar_p, &d2, log_limb_size, &template)
}

pub fn render_sha512_96_tests(template_file: &str) -> String {
    let template_path: &str = "src/wgsl/";
    let tests_path: &str = "src/wgsl/tests";

    let mut env = Environment::new();

    add_source_to_env(template_path, "sha512.wgsl", &mut env);
    add_source_to_env(tests_path, template_file, &mut env);

    let context = context! {};
    let template = env.get_template(template_file).unwrap();
    template.render(context).unwrap()
}
