use num_bigint::{RandomBits, BigUint};
use ark_ff::{PrimeField};
use ark_ed25519::Fq;
use fuel_algos::ed25519_eddsa::{
    is_negative,
    conditional_assign,
    conditional_negate,
    sqrt_ratio_i,
};
use multiprecision::{mont, bigint};
use multiprecision::utils::calc_num_limbs;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use crate::shader::render_ed25519_utils_tests;
use crate::gpu::{
    create_empty_sb,
    execute_pipeline,
    create_bind_group,
    create_sb_with_data,
    get_device_and_queue,
    create_command_encoder,
    create_compute_pipeline,
    finish_encoder_and_read_from_gpu,
};

#[serial_test::serial]
#[tokio::test]
pub async fn is_negative_test() {
    let a_val = BigUint::parse_bytes(b"7525073331273976790771568375528135302506060854772922661176563997672455312353", 10).unwrap();
    let expected = is_negative(Fq::from_be_bytes_mod_order(&a_val.to_bytes_be()));
    assert!(expected);

    let log_limb_size = 13;
    let num_limbs = 20;

    let a_val_limbs = bigint::from_biguint_le(&a_val, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let a_val_buf = create_sb_with_data(&device, &a_val_limbs);
    let b_val_buf = create_empty_sb(&device, a_val_buf.size());
    let result_buf = create_empty_sb(&device, a_val_buf.size());
    let result2_buf = create_empty_sb(&device, a_val_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_is_negative");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_val_buf, &b_val_buf, &result_buf, &result2_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size);
    let expected = if expected {
        BigUint::from(1u32)
    } else {
        BigUint::from(0u32)
    };
    assert_eq!(result, expected);
}

#[serial_test::serial]
#[tokio::test]
pub async fn conditional_assign_test() {
    do_conditional_assign_test(true).await;
    do_conditional_assign_test(false).await;
}

#[serial_test::serial]
#[tokio::test]
pub async fn conditional_negate_test() {
    do_conditional_negate_test(true).await;
    do_conditional_negate_test(false).await;
}

#[serial_test::serial]
#[tokio::test]
pub async fn pow_p58_test() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(2);

    for log_limb_size in 11..15 {
        for _ in 0..10 {
            let x: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            do_pow_p58_test(&x, &p, log_limb_size).await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn sqrt_ratio_i_test() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(3);

    for log_limb_size in 11..15 {
        for _ in 0..20 {
            let u: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let v: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            do_sqrt_ratio_i_test(&u, &v, &p, log_limb_size).await;
        }
    }
}

pub async fn do_sqrt_ratio_i_test(u: &BigUint, v: &BigUint, p: &BigUint, log_limb_size: u32) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let u_fq = Fq::from_be_bytes_mod_order(&u.to_bytes_be());
    let v_fq = Fq::from_be_bytes_mod_order(&v.to_bytes_be());
    let expected = sqrt_ratio_i(&u_fq, &v_fq);

    let ur_limbs = bigint::from_biguint_le(&(u * &r % p), num_limbs, log_limb_size);
    let vr_limbs = bigint::from_biguint_le(&(v * &r % p), num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let ur_buf = create_sb_with_data(&device, &ur_limbs);
    let vr_buf = create_sb_with_data(&device, &vr_limbs);
    let result_buf = create_empty_sb(&device, ur_buf.size());
    let result2_buf = create_empty_sb(&device, ur_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_sqrt_ratio_i");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&ur_buf, &vr_buf, &result_buf, &result2_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf, result2_buf],
    ).await;

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size) * rinv % p;
    let was_nonzero_square = bigint::to_biguint_le(&results[1], num_limbs, log_limb_size);

    let was_nonzero_square = was_nonzero_square == BigUint::from(1u32);

    assert_eq!(result, expected.1.into_bigint().into());
    assert_eq!(was_nonzero_square, expected.0);
}

pub async fn do_pow_p58_test(x: &BigUint, p: &BigUint, log_limb_size: u32) {
    let p58_exponent = BigUint::parse_bytes(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd", 16).unwrap();

    let expected = x.modpow(&p58_exponent, p);

    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);

    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let xr = x * r % p;
    let xr_limbs = bigint::from_biguint_le(&xr, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let xr_buf = create_sb_with_data(&device, &xr_limbs);
    let b_val_buf = create_empty_sb(&device, xr_buf.size());
    let result_buf = create_empty_sb(&device, xr_buf.size());
    let result2_buf = create_empty_sb(&device, xr_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_pow_p58");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&xr_buf, &b_val_buf, &result_buf, &result2_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size) * rinv % p;
    assert_eq!(result, expected);
}

pub async fn do_conditional_assign_test(choice: bool) {
    let a_val = BigUint::parse_bytes(b"123", 10).unwrap();
    let b_val = BigUint::parse_bytes(b"456", 10).unwrap();
    let a = Fq::from_be_bytes_mod_order(&a_val.to_bytes_be());
    let b = Fq::from_be_bytes_mod_order(&b_val.to_bytes_be());
    let expected = conditional_assign(a, b, choice);

    let log_limb_size = 13;
    let num_limbs = 20;

    let a_val_limbs = bigint::from_biguint_le(&a_val, num_limbs, log_limb_size);
    let b_val_limbs = bigint::from_biguint_le(&b_val, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let a_val_buf = create_sb_with_data(&device, &a_val_limbs);
    let b_val_buf = create_sb_with_data(&device, &b_val_limbs);
    let result_buf = create_empty_sb(&device, a_val_buf.size());
    let result2_buf = create_empty_sb(&device, a_val_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let entrypoint = if choice {
        "test_conditional_assign_true"
    } else {
        "test_conditional_assign_false"
    };
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_val_buf, &b_val_buf, &result_buf, &result2_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size);
    assert_eq!(result, expected.into_bigint().into());
}

pub async fn do_conditional_negate_test(choice: bool) {
    let a_val = BigUint::parse_bytes(b"123", 10).unwrap();
    let a = Fq::from_be_bytes_mod_order(&a_val.to_bytes_be());
    let expected = conditional_negate(a, choice);

    let log_limb_size = 13;
    let num_limbs = 20;

    let a_val_limbs = bigint::from_biguint_le(&a_val, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let a_val_buf = create_sb_with_data(&device, &a_val_limbs);
    let b_val_buf = create_empty_sb(&device, a_val_buf.size());
    let result_buf = create_empty_sb(&device, a_val_buf.size());
    let result2_buf = create_empty_sb(&device, a_val_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let entrypoint = if choice {
        "test_conditional_negate_true"
    } else {
        "test_conditional_negate_false"
    };
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_val_buf, &b_val_buf, &result_buf, &result2_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size);
    assert_eq!(result, expected.into_bigint().into());
}
