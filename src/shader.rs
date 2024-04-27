use minijinja::{Environment, context};
use std::path::PathBuf;
use num_bigint::BigUint;
use multiprecision::utils::calc_num_limbs;
use multiprecision::mont;

pub fn render(
    template_path: &str,
    template_file: &str,
    p: &BigUint,
    log_limb_size: u32,
) -> String {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let two_pow_word_size = 2u32.pow(log_limb_size);
    let mask = two_pow_word_size - 1u32;
    let nsafe = mont::calc_nsafe(log_limb_size);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let n0 = res.1;

    let mut env = Environment::new();

    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(template_path)
        .join(template_file);

    let source = std::fs::read_to_string(&input_path).unwrap();
    env.add_template(template_file, &source).unwrap();

    let template = env.get_template(template_file).unwrap();
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

#[test]
pub fn test_render() {
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();
    let log_limb_size = 13;
    assert_eq!(
        render("src/tests/", "test_template.md", &p, log_limb_size),
        format!("num_limbs + 1: {}", 21)
    );
}
