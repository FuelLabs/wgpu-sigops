use minijinja::{Environment, Template, context};
use std::path::PathBuf;
use num_bigint::BigUint;
use multiprecision::utils::calc_num_limbs;
use multiprecision::{ bigint, mont, ff, utils::calc_bitwidth };

fn read_from_file(
    path: &str,
    file: &str,
) -> String {
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path).join(file);
    std::fs::read_to_string(&input_path).unwrap()
}

pub fn gen_constant_bigint(
    var_name: &str,
    val: &BigUint,
    num_limbs: usize,
    log_limb_size: u32
) -> String {
    let r_limbs = bigint::from_biguint_le(val, num_limbs, log_limb_size);
    let mut result = format!("var {}: BigInt = BigInt(array<u32, {}>(", var_name, num_limbs).to_owned();

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

    let br = b * &r % p;
    let br_bigint = gen_constant_bigint("br", &br, num_limbs, log_limb_size);
    let mu_bigint = gen_constant_bigint("mu", &ff::gen_mu(&p), num_limbs, log_limb_size);

    let sqrt_case3mod4_exponent = (p + BigUint::from(1u32)) / BigUint::from(4u32);
    let sqrt_case3mod4_exponent_bigint = gen_constant_bigint("sqrt_case3mod4_exponent", &sqrt_case3mod4_exponent, num_limbs, log_limb_size);

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
        br_bigint => br_bigint,
        mu_bigint => mu_bigint,
        sqrt_case3mod4_exponent_bigint => sqrt_case3mod4_exponent_bigint,
    };
    template.render(context).unwrap()
}

pub fn render_tests(
    template_path: &str,
    template_file: &str,
    p: &BigUint,
    b: &BigUint,
    log_limb_size: u32,
) -> String {
    let mut env = Environment::new();

    let source = read_from_file(template_path, "bigint.wgsl");
    env.add_template("bigint.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "ff.wgsl");
    env.add_template("ff.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "mont.wgsl");
    env.add_template("mont.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "constants.wgsl");
    env.add_template("constants.wgsl", &source).unwrap();

    let source = read_from_file(template_path, template_file);
    env.add_template(template_file, &source).unwrap();

    let template = env.get_template(template_file).unwrap();
    do_render(p, b, log_limb_size, &template)
}

pub fn render_curve_tests(
    template_path: &str,
    template_file: &str,
    p: &BigUint,
    b: &BigUint,
    log_limb_size: u32,
) -> String {
    let mut env = Environment::new();

    let source = read_from_file(template_path, "bigint.wgsl");
    env.add_template("bigint.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "ff.wgsl");
    env.add_template("ff.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "mont.wgsl");
    env.add_template("mont.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "curve.wgsl");
    env.add_template("curve.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "constants.wgsl");
    env.add_template("constants.wgsl", &source).unwrap();

    let source = read_from_file(template_path, template_file);
    env.add_template(template_file, &source).unwrap();

    let template = env.get_template(template_file).unwrap();
    do_render(p, b, log_limb_size, &template)
}
