use minijinja::{Environment, Template, context};
use std::path::PathBuf;
use num_bigint::BigUint;
use multiprecision::utils::calc_num_limbs;
use multiprecision::mont;

fn read_from_file(
    path: &str,
    file: &str,
) -> String {
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(path)
        .join(file);
    std::fs::read_to_string(&input_path).unwrap()
}

pub fn do_render(
    p: &BigUint,
    log_limb_size: u32,
    template: &Template,
) -> String {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let two_pow_word_size = 2u32.pow(log_limb_size);
    let mask = two_pow_word_size - 1u32;
    let nsafe = mont::calc_nsafe(log_limb_size);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let n0 = res.1;

    let context = context! {
        num_limbs => num_limbs,
        log_limb_size => log_limb_size,
        two_pow_word_size => two_pow_word_size,
        mask => mask,
        nsafe => nsafe,
        n0 => n0,
    };
    template.render(context).unwrap()
}

pub fn render_tests(
    template_path: &str,
    template_file: &str,
    p: &BigUint,
    log_limb_size: u32,
) -> String {
    let mut env = Environment::new();

    let source = read_from_file(template_path, "bigint.wgsl");
    env.add_template("bigint.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "ff.wgsl");
    env.add_template("ff.wgsl", &source).unwrap();

    let source = read_from_file(template_path, "mont.wgsl");
    env.add_template("mont.wgsl", &source).unwrap();

    let source = read_from_file(template_path, template_file);
    env.add_template(template_file, &source).unwrap();

    let template = env.get_template(template_file).unwrap();
    do_render(p, log_limb_size, &template)
}
