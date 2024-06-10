use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, create_ub_with_data,execute_pipeline, finish_encoder_and_read_from_gpu,
    get_device_and_queue,
};
use crate::benchmarks::compute_num_workgroups;
use crate::shader::render_secp256r1_ecdsa_tests;
use crate::tests::secp256r1_curve::projective_to_affine_func;
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, PrimeField};
use ark_secp256r1::{Affine, Fq};
use fuel_crypto::secp256r1::p256::{recover, sign_prehashed};
use fuel_crypto::Message;
use p256::ecdsa::SigningKey;
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, mont};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;

#[serial_test::serial]
#[tokio::test]
pub async fn secp256r1_ecrecover_multiple_benchmarks() {
    let check = false;
    let log_limb_size = 13u32;
    let start = 8;
    let end = 18;

    let mut data = Vec::with_capacity(end - start);
    for i in start..end {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!("secp256r1 signature verification benchmarks: \n{}\n\n", table);
}

#[serial_test::serial]
#[tokio::test]
pub async fn secp256r1_ecrecover_benchmarks() {
    let check = true;
    let log_limb_size = 13u32;
    let num_signatures = 2u32.pow(13u32) as usize;

    let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures).await;

    println!("CPU took {}ms to recover {} secp256r1 ECDSA signatures in serial.", cpu_ms, num_signatures);
    println!("GPU took {}ms to recover {} secp256r1 ECDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

pub async fn do_benchmark(
    check: bool,
    log_limb_size: u32,
    num_signatures: usize,
) -> (u32, u32) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;
    let scalar_p = crate::moduli::secp256r1_fr_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(2);

    let workgroup_size = 256;
    let (num_x_workgroups, num_y_workgroups, num_z_workgroups) = compute_num_workgroups(num_signatures, workgroup_size);

    let mut signatures = Vec::with_capacity(num_signatures);
    let mut messages = Vec::with_capacity(num_signatures);
    let mut expected_pks = Vec::with_capacity(num_signatures);

    for _ in 0..num_signatures {
        // Generate a random message
        let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
        let message = Message::new(hex::encode(msg.to_bytes_be()));
        let signing_key = SigningKey::random(&mut rng);
        let verifying_key = signing_key.verifying_key();

        let pk_affine_bytes = &verifying_key.to_sec1_bytes()[1..65];
        let pk_x = pk_affine_bytes[0..32].to_vec();
        let pk_x = BigUint::from_bytes_be(&pk_x);
        let pk_x = Fq::from_be_bytes_mod_order(&pk_x.to_bytes_be());
        let pk_y = pk_affine_bytes[32..64].to_vec();
        let pk_y = BigUint::from_bytes_be(&pk_y);
        let pk_y = Fq::from_be_bytes_mod_order(&pk_y.to_bytes_be());
        let pk = Affine::new(pk_x, pk_y);

        let fuel_signature = sign_prehashed(&signing_key, &message).expect("Couldn't sign");

        signatures.push(fuel_signature);
        messages.push(message);
        expected_pks.push(pk);
    }

    // Perform signature recovery using the CPU
    let sw = Stopwatch::start_new();
    for i in 0..num_signatures {
        let _ = recover(&signatures[i], &messages[i]);
    }
    let cpu_ms = sw.elapsed_ms();

    // Start the GPU stopwatch
    let sw = Stopwatch::start_new();

    // Set up data for the input buffers
    let mut all_sig_bytes = Vec::<u8>::with_capacity(num_signatures * 64);
    let mut all_msg_bytes = Vec::<u8>::with_capacity(num_signatures * 32);
    for sig in signatures {
        let sig_bytes = sig.as_slice();
        all_sig_bytes.extend(sig_bytes);
    }

    for msg in messages {
        let msg_bytes = msg.as_slice();
        all_msg_bytes.extend(msg_bytes);
    }

    let all_sig_u32s: Vec<u32> = bytemuck::cast_slice(&all_sig_bytes).to_vec();
    let all_msg_u32s: Vec<u32> = bytemuck::cast_slice(&all_msg_bytes).to_vec();

    let params = &[num_x_workgroups as u32, num_y_workgroups as u32, num_z_workgroups as u32];

    let (device, queue) = get_device_and_queue().await;
    let source = render_secp256r1_ecdsa_tests("secp256r1_ecdsa_benchmarks.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "benchmark_secp256r1_recover");

    let sig_buf = create_sb_with_data(&device, &all_sig_u32s);
    let msg_buf = create_sb_with_data(&device, &all_msg_u32s);
    let result_buf = create_empty_sb(&device, (num_limbs * 3 * num_signatures * std::mem::size_of::<u32>()) as u64);
    let params_buf = create_ub_with_data(&device, params);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sig_buf, &msg_buf, &result_buf, &params_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    );

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;
    let gpu_ms = sw.elapsed_ms();

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result = bigint::to_biguint_le(&data, num_limbs, log_limb_size);

        let result = &result * &rinv % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    if check {
        for i in 0..num_signatures {
            let result_x = convert_result_coord(&results[0][
                (i * num_limbs * 3)..(i * num_limbs * 3 + num_limbs)
            ].to_vec());
            let result_y = convert_result_coord(&results[0][
                (i * num_limbs * 3 + num_limbs)..(i * num_limbs * 3 + num_limbs * 2)
            ].to_vec());
            let result_z = convert_result_coord(&results[0][
                (i * num_limbs * 3 + num_limbs * 2)..(i * num_limbs * 3 + num_limbs * 3)
            ].to_vec());

            //println!("{}", i);
            //println!("result_x: {}", result_x);
            //println!("result_y: {}", result_y);
            //println!("result_z: {}", result_z);

            let result_affine = projective_to_affine_func(result_x, result_y, result_z);

            if result_affine.is_zero() {
                println!("i: {}", i);
            }

            assert_eq!(result_affine, expected_pks[i]);
        }
    }

    (cpu_ms as u32, gpu_ms as u32)
}
