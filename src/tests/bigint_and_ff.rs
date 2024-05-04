use crate::shader::render_tests;
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
use num_bigint::{ BigUint, RandomBits };
use multiprecision::{ bigint, ff };
use multiprecision::utils::calc_num_limbs;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use ark_secp256k1::Fq;
use ark_ff::{ Field, PrimeField, BigInteger };
use crate::tests::{ get_secp256k1_b, do_test };

const NUM_RUNS_PER_TEST: usize = 8;

fn gen_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(2)
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_bigint_div2() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f", 16).unwrap();
    for log_limb_size in 11..16 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..10 {
            let mut a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;

            if &a % BigUint::from(2u32) != BigUint::from(0u32) {
                a += BigUint::from(1u32);
            }
            let expected = &a / BigUint::from(2u32);

            do_expected_test(&a, &p, &expected, log_limb_size, num_limbs, "bigint_and_ff_tests.wgsl", "test_bigint_div_2").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_ff_inverse() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f", 16).unwrap();
    for log_limb_size in 11..16 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..10 {
            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;

            let expected = Fq::from_be_bytes_mod_order(&a.to_bytes_be()).inverse().unwrap();
            let expected = BigUint::from_bytes_be(&expected.into_bigint().to_bytes_be()) % &p;

            do_expected_test(&a, &p, &expected, log_limb_size, num_limbs, "bigint_and_ff_tests.wgsl", "test_ff_inverse").await;
        }
    }
}

pub async fn do_expected_test(
    a: &BigUint,
    p: &BigUint,
    expected: &BigUint,
    log_limb_size: u32,
    num_limbs: usize,
    filename: &str,
    entrypoint: &str,
) {
    let a_limbs = bigint::from_biguint_le(a, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &a_limbs);
    let b_buf = create_empty_sb(&device, a_buf.size());
    let result_buf = create_empty_sb(&device, a_buf.size() + 4);

    let source = render_tests("src/wgsl/", filename, &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(
        &results[0][0..num_limbs].to_vec(),
        num_limbs,
        log_limb_size,
    );

    assert_eq!(&result, expected);
}

#[serial_test::serial]
#[tokio::test]
pub async fn ff_add() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    fn biguint_func(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint { 
        (a + b) % p
    }

    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..NUM_RUNS_PER_TEST {
            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;

            do_test(a, b, p.clone(), log_limb_size, num_limbs, num_limbs, ff::add, biguint_func, "bigint_and_ff_tests.wgsl", "test_ff_add").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn ff_sub() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    fn biguint_func(a: &BigUint, b: &BigUint, p: &BigUint) -> BigUint { 
        if a > b {
            a - b
        } else {
            let r = b - a;
            p - r
        }
    }

    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..NUM_RUNS_PER_TEST {
            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
    
            do_test(a, b, p.clone(), log_limb_size, num_limbs, num_limbs, ff::sub, biguint_func, "bigint_and_ff_tests.wgsl", "test_ff_sub").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn ff_mul() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    for log_limb_size in 11..15 {
        for _ in 0..NUM_RUNS_PER_TEST {
            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
    
            do_ff_mul_test(&a, &b, &p, log_limb_size, "bigint_and_ff_tests.wgsl", "test_ff_mul").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_wide_add() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        fn biguint_func(a: &BigUint, b: &BigUint, _p: &BigUint) -> BigUint { a + b }
        let max = BigUint::from(2u32).pow(256);

        for _ in 0..NUM_RUNS_PER_TEST {
            let mut a: BigUint;
            let mut b: BigUint;

            loop {
                a = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
                b = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;

                // We are testing add_wide, so the sum should overflow
                if &a + &b > max {
                    break;
                }
            }

            fn bigint_func(a: &Vec<u32>, b: &Vec<u32>, _p: &Vec<u32>, log_limb_size: u32) -> Vec<u32> { 
                bigint::add_wide(a, b, log_limb_size)
            }
    
            do_test(a, b, p.clone(), log_limb_size, num_limbs, num_limbs + 1, bigint_func, biguint_func, "bigint_and_ff_tests.wgsl", "test_bigint_wide_add").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_add_unsafe() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    fn bigint_func(a: &Vec<u32>, b: &Vec<u32>, _p: &Vec<u32>, log_limb_size: u32) -> Vec<u32> { 
        bigint::add_unsafe(a, b, log_limb_size)
    }

    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        fn biguint_func(a: &BigUint, b: &BigUint, _p: &BigUint) -> BigUint { a + b }

        for _ in 0..NUM_RUNS_PER_TEST {
            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
    
            do_test(a, b, p.clone(), log_limb_size, num_limbs, num_limbs, bigint_func, biguint_func, "bigint_and_ff_tests.wgsl", "test_bigint_add_unsafe").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_sub() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    fn biguint_func(a: &BigUint, b: &BigUint, _p: &BigUint) -> BigUint { a - b }

    fn bigint_func(a: &Vec<u32>, b: &Vec<u32>, _p: &Vec<u32>, log_limb_size: u32) -> Vec<u32> { 
        bigint::sub(a, b, log_limb_size)
    }

    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..NUM_RUNS_PER_TEST {
            let x: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let y: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;

            let (a, b) = if x > y {
                (x, y)
            } else {
                (y, x)
            };

            do_test(a, b, p.clone(), log_limb_size, num_limbs, num_limbs, bigint_func, biguint_func, "bigint_and_ff_tests.wgsl", "test_bigint_sub").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_wide_sub() {
    let mut rng = gen_rng();

    for log_limb_size in 13..14 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..NUM_RUNS_PER_TEST {
            let x: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let y: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));

            let (a, b) = if x > y {
                (x, y)
            } else {
                (y, x)
            };

            do_bigint_wide_sub_test(a, b, log_limb_size, num_limbs + 1, "bigint_and_ff_tests.wgsl", "test_bigint_wide_sub").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_gte() {
    let mut rng = gen_rng();
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    fn biguint_func(a: &BigUint, b: &BigUint, _p: &BigUint) -> BigUint { 
        if a >= b {
            BigUint::from(1u32)
        } else {
            BigUint::from(0u32)
        }
    }

    fn bigint_func(a: &Vec<u32>, b: &Vec<u32>, _p: &Vec<u32>, _: u32) -> Vec<u32> { 
        let mut result = Vec::<u32>::with_capacity(a.len());
        for _ in 0..a.len() { result.push(0u32); }
        if bigint::gte(a, b) {
            result[0] = 1u32;
        }
        result
    }

    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for _ in 0..NUM_RUNS_PER_TEST {
            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));

            do_test(a, b, p.clone(), log_limb_size, num_limbs, num_limbs, bigint_func, biguint_func, "bigint_and_ff_tests.wgsl", "test_bigint_gte").await;
        }
    }
}

fn biguint_gte_func(a: &BigUint, b: &BigUint) -> BigUint { 
    if a >= b {
        BigUint::from(1u32)
    } else {
        BigUint::from(0u32)
    }
}

fn bigint_gte_func(a: &Vec<u32>, b: &Vec<u32>, _: u32) -> Vec<u32> { 
    let mut result = Vec::<u32>::with_capacity(a.len());
    for _ in 0..a.len() { result.push(0u32); }
    if bigint::gte(a, b) {
        result[0] = 1u32;
    }
    result
}

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_wide_gte() {
    let mut rng = gen_rng();

    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256) + 1;
        for _ in 0..NUM_RUNS_PER_TEST {
            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));

            do_bigint_wide_gte_test(a, b, log_limb_size, num_limbs, bigint_gte_func, biguint_gte_func, "bigint_and_ff_tests.wgsl", "test_bigint_wide_gte").await;
        }
    }
}

async fn do_bigint_wide_sub_test(
    a: BigUint,
    b: BigUint,
    log_limb_size: u32,
    num_limbs: usize,
    filename: &str,
    entrypoint: &str,
) {
    let expected = &a - &b;
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();
    let a_limbs = bigint::from_biguint_le(&a, num_limbs, log_limb_size);
    let b_limbs = bigint::from_biguint_le(&b, num_limbs, log_limb_size);
    let expected_limbs = bigint::from_biguint_le(&expected, num_limbs, log_limb_size);
    let expected_limbs_2 = bigint::sub(&a_limbs, &b_limbs, log_limb_size);

    assert!(bigint::eq(&expected_limbs, &expected_limbs_2));

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &a_limbs);
    let b_buf = create_sb_with_data(&device, &b_limbs);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);

    let source = render_tests("src/wgsl/", filename, &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(
        &results[0][0..num_limbs].to_vec(),
        num_limbs,
        log_limb_size,
    );

    assert_eq!(result, expected);
}

async fn do_bigint_wide_gte_test(
    a: BigUint,
    b: BigUint,
    log_limb_size: u32,
    num_limbs: usize,
    func: fn(&Vec<u32>, &Vec<u32>, u32) -> Vec<u32>,
    biguint_func: fn(&BigUint, &BigUint) -> BigUint,
    filename: &str,
    entrypoint: &str,
) {
    let expected = biguint_func(&a, &b);
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();
    let a_limbs = bigint::from_biguint_le(&a, num_limbs, log_limb_size);
    let b_limbs = bigint::from_biguint_le(&b, num_limbs, log_limb_size);
    let expected_limbs = bigint::from_biguint_le(&expected, num_limbs, log_limb_size);
    let expected_limbs_2 = func(&a_limbs, &b_limbs, log_limb_size);

    assert!(bigint::eq(&expected_limbs, &expected_limbs_2));

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &a_limbs);
    let b_buf = create_sb_with_data(&device, &b_limbs);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);

    let source = render_tests("src/wgsl/", filename, &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(
        &results[0][0..num_limbs].to_vec(),
        num_limbs,
        log_limb_size,
    );

    assert_eq!(result, expected);
}

async fn do_ff_mul_test(
    a: &BigUint,
    b: &BigUint,
    p: &BigUint,
    log_limb_size: u32,
    filename: &str,
    entrypoint: &str,
) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let expected = a * b % p;
    let a_limbs = bigint::from_biguint_le(&a, num_limbs, log_limb_size);
    let b_limbs = bigint::from_biguint_le(&b, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &a_limbs);
    let b_buf = create_sb_with_data(&device, &b_limbs);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);

    let source = render_tests("src/wgsl/", filename, &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(
        &results[0][0..num_limbs].to_vec(),
        num_limbs,
        log_limb_size,
    );

    assert_eq!(result, expected);
}
