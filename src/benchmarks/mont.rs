use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::moduli;
use crate::shader::render_bigint_ff_mont_tests;
use crate::tests::get_secp256k1_b;
use multiprecision::mont::calc_rinv_and_n0;
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, mont};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;

const NUM_RUNS_PER_BENCHMARK: usize = 8;

fn gen_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(2)
}

#[serial_test::serial]
#[tokio::test]
pub async fn mont_mul_benchmarks() {
    let mut rng = gen_rng();

    let p = moduli::secp256k1_fq_modulus_biguint();

    let cost = 8192;

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .unwrap();
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
        for i in 0..NUM_RUNS_PER_BENCHMARK + 1 {
            let elapsed = do_mont_benchmark(
                &ar,
                &br,
                &p,
                &r,
                log_limb_size,
                num_limbs,
                "mont_mul_benchmarks.wgsl",
                "benchmark_mont_mul",
                cost,
            )
            .await;

            if i > 0 {
                total += elapsed;
            }
        }

        let average = total as u32 / NUM_RUNS_PER_BENCHMARK as u32;
        println!(
            "for {}-bit limbs, mont_mul took an average of {}ms over {} runs",
            log_limb_size, average, NUM_RUNS_PER_BENCHMARK
        );
    }
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
    let ar_limbs = bigint::from_biguint_le(&ar, num_limbs, log_limb_size);
    let br_limbs = bigint::from_biguint_le(&br, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &ar_limbs);
    let b_buf = create_sb_with_data(&device, &br_limbs);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);
    let cost_buf = create_sb_with_data(&device, &[cost]);

    let source =
        render_bigint_ff_mont_tests("src/wgsl/", filename, &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &result_buf, &cost_buf],
    );

    let sw = Stopwatch::start_new();
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
    let elapsed = sw.elapsed_ms();

    let result =
        bigint::to_biguint_le(&results[0][0..num_limbs].to_vec(), num_limbs, log_limb_size);

    assert_eq!(result, expected);

    elapsed as u32
}

fn expensive_computation(
    ar: &BigUint,
    br: &BigUint,
    p: &BigUint,
    rinv: &BigUint,
    cost: u32,
) -> BigUint {
    let mut result = ar.clone();
    for _ in 1..cost {
        result = (ar * &result * rinv) % p;
    }
    return (&result * br * rinv) % p;
}

