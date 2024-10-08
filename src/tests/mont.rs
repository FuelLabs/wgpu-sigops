use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::moduli;
use crate::shader::{render_bigint_ff_mont_tests, render_mont_sqrt_case3mod4_test};
use crate::tests::get_secp256k1_b;
use multiprecision::mont::{calc_nsafe, calc_rinv_and_n0};
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, mont};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn gen_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(2)
}

const NUM_RUNS_PER_TEST: usize = 8;

#[serial_test::serial]
#[tokio::test]
pub async fn mont_mul() {
    let mut rng = gen_rng();

    let p0 = moduli::secp256k1_fq_modulus_biguint();
    let p1 = moduli::secp256k1_fr_modulus_biguint();
    let p2 = moduli::secp256r1_fq_modulus_biguint();
    let p3 = moduli::secp256r1_fr_modulus_biguint();
    let p4 = moduli::ed25519_fq_modulus_biguint();

    for p in &[&p0, &p1, &p2, &p3, &p4] {
        for log_limb_size in 13..14 {
            for _ in 0..NUM_RUNS_PER_TEST {
                let num_limbs = calc_num_limbs(log_limb_size, 256);
                let r = mont::calc_mont_radix(num_limbs, log_limb_size);

                let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % *p;
                let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % *p;
                let ar = &a * &r % *p;
                let br = &b * &r % *p;

                do_mont_test(
                    &ar,
                    &br,
                    &p,
                    &r,
                    log_limb_size,
                    num_limbs,
                    "mont_tests.wgsl",
                    "test_mont_mul",
                )
                .await;
            }
        }
    }
}

pub async fn do_mont_test(
    ar: &BigUint,
    br: &BigUint,
    p: &BigUint,
    r: &BigUint,
    log_limb_size: u32,
    num_limbs: usize,
    filename: &str,
    entrypoint: &str,
) {
    let res = calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;
    let n0 = res.1;

    let expected = (ar * br * rinv) % p;
    let p_limbs = bigint::from_biguint_le(p, num_limbs, log_limb_size);
    let ar_limbs = bigint::from_biguint_le(ar, num_limbs, log_limb_size);
    let br_limbs = bigint::from_biguint_le(br, num_limbs, log_limb_size);
    let expected_limbs = bigint::from_biguint_le(&expected, num_limbs, log_limb_size);

    let expected_limbs_2 = if log_limb_size == 12 || log_limb_size == 13 {
        mont::mont_mul_optimised(&ar_limbs, &br_limbs, &p_limbs, n0, num_limbs, log_limb_size)
    } else if log_limb_size == 14 || log_limb_size == 15 {
        let nsafe = calc_nsafe(log_limb_size);
        mont::mont_mul_modified(
            &ar_limbs,
            &br_limbs,
            &p_limbs,
            n0,
            num_limbs,
            log_limb_size,
            nsafe,
        )
    } else {
        unimplemented!();
    };

    assert!(bigint::eq(&expected_limbs, &expected_limbs_2));

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &ar_limbs);
    let b_buf = create_sb_with_data(&device, &br_limbs);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);

    let source = render_bigint_ff_mont_tests(filename, &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &result_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;

    let result =
        bigint::to_biguint_le(&results[0][0..num_limbs].to_vec(), num_limbs, log_limb_size);

    assert_eq!(result, expected);
}

#[serial_test::serial]
#[tokio::test]
pub async fn mont_sqrt_case3mod4() {
    // Given xr, find sqrt(x)r
    // Note that sqrt(xy) = sqrt(x) * sqrt(y)
    //
    // sqrt(xr) = sqrt(x) * sqrt(r)
    // sqrt(x)r = sqrt(xr) * sqrt(r)
    //          = sqrt(x) * sqrt(r) * sqrt(r)
    //          = sqrt(x)r
    let mut rng = gen_rng();

    let p0 = moduli::secp256k1_fq_modulus_biguint();
    let p1 = moduli::secp256k1_fr_modulus_biguint();
    let p2 = moduli::secp256r1_fq_modulus_biguint();
    let p3 = moduli::secp256r1_fr_modulus_biguint();

    for p in &[&p0, &p1, &p2, &p3] {
        for log_limb_size in 11..16 {
            for _ in 0..NUM_RUNS_PER_TEST {
                let num_limbs = calc_num_limbs(log_limb_size, 256);
                let r = mont::calc_mont_radix(num_limbs, log_limb_size);

                let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
                let x: BigUint = &s * &s % *p;

                do_mont_sqrt_case3mod4_test(
                    &x,
                    &p,
                    &r,
                    log_limb_size,
                    num_limbs,
                    "mont_sqrt_case3mod4_tests.wgsl",
                    "test_mont_sqrt_case3mod4",
                )
                .await
            }
        }
    }
}

pub async fn do_mont_sqrt_case3mod4_test(
    x: &BigUint,
    p: &BigUint,
    r: &BigUint,
    log_limb_size: u32,
    num_limbs: usize,
    filename: &str,
    entrypoint: &str,
) {
    let exponent = (p + BigUint::from(1u32)) / BigUint::from(4u32);
    let xr = x * r % p;
    let expected_a = x.modpow(&exponent, p) * r % p;
    let expected_b = p - &expected_a % p;

    let xr_limbs = bigint::from_biguint_le(&xr, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;

    let xr_buf = create_sb_with_data(&device, &xr_limbs);
    let result_a_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);
    let result_b_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);

    let source = render_mont_sqrt_case3mod4_test(filename, &p, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&xr_buf, &result_a_buf, &result_b_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_a_buf, result_b_buf],
    )
    .await;

    let result_a =
        bigint::to_biguint_le(&results[0][0..num_limbs].to_vec(), num_limbs, log_limb_size);
    let result_b =
        bigint::to_biguint_le(&results[1][0..num_limbs].to_vec(), num_limbs, log_limb_size);

    assert!(result_a == expected_a || result_a == expected_b);
    assert!(result_b == expected_b || result_b == expected_a);
}
