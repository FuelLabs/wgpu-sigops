use stopwatch::Stopwatch;
use multiprecision::{ bigint, mont };
use multiprecision::mont::{ calc_nsafe, calc_rinv_and_n0 };
use multiprecision::utils::calc_num_limbs;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use num_bigint::{ BigUint, RandomBits };
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
use crate::shader::render_tests;

fn gen_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(2)
}

const NUM_RUNS_PER_TEST: usize = 8;

#[serial_test::serial]
#[tokio::test]
pub async fn mont_mul_benchmarks() {
    let mut rng = gen_rng();

    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    let cost = 8192;

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await.unwrap();
    println!("{:?}", adapter.get_info());

    println!("Benchmarks for {} serial Montgomery multiplications:", cost);

    for log_limb_size in 11..16 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);
        let r = mont::calc_mont_radix(num_limbs, log_limb_size);

        let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
        let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
        let ar = &a * &r % &p;
        let br = &b * &r % &p;

        let mut total = 0;
        for i in 0..NUM_RUNS_PER_TEST + 1 {
            let elapsed = do_mont_benchmark(&ar, &br, &p, &r, log_limb_size, num_limbs, "benchmarks.wgsl", "benchmark_mont_mul", cost).await;

            if i > 0 {
                total += elapsed;
            }
        }

        let average = total as u32 / NUM_RUNS_PER_TEST as u32;
        println!("for {}-bit limbs, mont_mul took an average of {}ms over {} runs", log_limb_size, average, NUM_RUNS_PER_TEST);
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn mont_mul() {
    let mut rng = gen_rng();

    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();

    for log_limb_size in 12..16 {
        for _ in 0..NUM_RUNS_PER_TEST {
            let num_limbs = calc_num_limbs(log_limb_size, 256);
            let r = mont::calc_mont_radix(num_limbs, log_limb_size);

            let a: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let b: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let ar = &a * &r % &p;
            let br = &b * &r % &p;

            do_mont_test(&ar, &br, &p, &r, log_limb_size, num_limbs, "tests.wgsl", "test_mont_mul").await;
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
        mont::mont_mul_modified(&ar_limbs, &br_limbs, &p_limbs, n0, num_limbs, log_limb_size, nsafe)
    } else {
        unimplemented!();
    };

    assert!(bigint::eq(&expected_limbs, &expected_limbs_2));

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &ar_limbs);
    let b_buf = create_sb_with_data(&device, &br_limbs);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);
    let p_buf = create_sb_with_data(&device, &p_limbs);

    let source = render_tests("src/wgsl/", filename, &p, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &p_buf, &result_buf],
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

fn expensive_computation(
    ar: &BigUint,
    br: &BigUint,
    p: &BigUint,
    rinv: &BigUint,
    cost: u32
) -> BigUint {
    let mut result = ar.clone();
    for _ in 1..cost {
        result = (ar * &result * rinv) % p;
    }
    return (&result * br * rinv) % p;
}

pub async fn do_mont_benchmark(
    ar: &BigUint,
    br: &BigUint,
    p: &BigUint,
    r: &BigUint,
    log_limb_size: u32,
    num_limbs: usize,
    filename: &str,
    entrypoint: &str,
    cost: u32,
) -> u32 {
    let res = calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let expected = expensive_computation(&ar, &br, &p, &rinv, cost);
    let p_limbs = bigint::from_biguint_le(&p, num_limbs, log_limb_size);
    let ar_limbs = bigint::from_biguint_le(&ar, num_limbs, log_limb_size);
    let br_limbs = bigint::from_biguint_le(&br, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &ar_limbs);
    let b_buf = create_sb_with_data(&device, &br_limbs);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);
    let p_buf = create_sb_with_data(&device, &p_limbs);
    let cost_buf = create_sb_with_data(&device, &[cost]);

    let source = render_tests("src/wgsl/", filename, &p, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &p_buf, &result_buf, &cost_buf],
    );

    let sw = Stopwatch::start_new();
    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;
    let elapsed = sw.elapsed_ms();

    let result = bigint::to_biguint_le(
        &results[0][0..num_limbs].to_vec(),
        num_limbs,
        log_limb_size,
    );

    assert_eq!(result, expected);

    elapsed as u32
}
