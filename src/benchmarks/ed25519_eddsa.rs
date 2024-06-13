use crate::benchmarks::compute_num_workgroups;
use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
    create_ub_with_data,
};
use crate::shader::render_ed25519_eddsa_tests;
use ark_ed25519::{EdwardsAffine as Affine, Fq};
use ark_ff::PrimeField;
use ed25519_dalek::{Signature, Signer, SigningKey};
use crate::curve_algos::ed25519_eddsa::{
    ark_ecverify, curve25519_ecverify,
};
use fuel_crypto::Message;
use multiprecision::utils::calc_num_limbs;
use multiprecision::bigint;
use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_multiple_benchmarks() {
    let check = false;
    let log_limb_size = 13u32;
    let start = 10;
    let end = 18;

    let mut data = Vec::with_capacity(end - start);
    for i in start..end {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!("ed25519 signature verification benchmarks: \n{}\n\n", table);
}

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_benchmarks() {
    let check = true;
    let log_limb_size = 13u32;
    let num_signatures = 2u32.pow(13u32) as usize;

    let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures).await;

    println!("CPU took {}ms to recover {} ed25519 EdDSA signatures in serial.", cpu_ms, num_signatures);
    println!("GPU took {}ms to recover {} ed25519 EdDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

pub async fn do_benchmark(
    check: bool,
    log_limb_size: u32,
    num_signatures: usize,
) -> (u32, u32) {
    let p = crate::moduli::ed25519_fq_modulus_biguint();
    let num_limbs = calc_num_limbs(log_limb_size, 256);

    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let workgroup_size = 256;
    let (num_x_workgroups, num_y_workgroups, num_z_workgroups) = compute_num_workgroups(num_signatures, workgroup_size);

    let mut signatures = Vec::with_capacity(num_signatures);
    let mut verifying_keys = Vec::with_capacity(num_signatures);
    let mut messages = Vec::with_capacity(num_signatures);
    let mut expected_pks = Vec::with_capacity(num_signatures);

    for _ in 0..num_signatures {
        let signing_key: SigningKey = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let mut message = [0u8; 100];
        rng.fill_bytes(&mut message);

        let message = Message::new(&message);
        messages.push(message);

        let message = message.as_slice();

        let signature: Signature = signing_key.sign(&message);

        let expected_pk = ark_ecverify(&verifying_key, &signature, &message);

        signatures.push(signature);
        verifying_keys.push(verifying_key);
        expected_pks.push(expected_pk);
    }

    let sw = Stopwatch::start_new();
    for i in 0..num_signatures {
        let _ = curve25519_ecverify(&verifying_keys[i], &signatures[i], &messages[i].as_slice());
    }
    let cpu_ms = sw.elapsed_ms();

    // Start the GPU stopwatch
    let sw = Stopwatch::start_new();

    // Set up data for the input buffers
    let mut all_sig_bytes = Vec::with_capacity(num_signatures * 64 * 8);
    let mut all_pk_bytes = Vec::with_capacity(num_signatures * 32 * 8);
    let mut all_msg_bytes: Vec<u8> = Vec::with_capacity(num_signatures * 32 * 8);

    for i in 0..num_signatures {
        let sig_bytes_be = signatures[i].to_bytes();
        let pk_bytes_be = verifying_keys[i].to_bytes();
        let msg_bytes = messages[i].as_slice();

        all_sig_bytes.extend(sig_bytes_be);
        all_pk_bytes.extend(pk_bytes_be);
        all_msg_bytes.extend(msg_bytes);
    }

    let all_sig_u32s: Vec<u32> = bytemuck::cast_slice(&all_sig_bytes).to_vec();
    let all_pk_u32s: Vec<u32> = bytemuck::cast_slice(&all_pk_bytes).to_vec();
    let all_msg_u32s: Vec<u32> = bytemuck::cast_slice(&all_msg_bytes).to_vec();

    let (device, queue) = get_device_and_queue().await;
    let params = &[num_x_workgroups as u32, num_y_workgroups as u32, num_z_workgroups as u32];

    let sig_buf = create_sb_with_data(&device, &all_sig_u32s);
    let pk_buf = create_sb_with_data(&device, &all_pk_u32s);
    let msg_buf = create_sb_with_data(&device, &all_msg_u32s);
    let result_buf = create_empty_sb(&device, (num_signatures * num_limbs * 2 * std::mem::size_of::<u32>()) as u64);
    let params_buf = create_ub_with_data(&device, params);

    let source = render_ed25519_eddsa_tests("ed25519_eddsa_benchmarks.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "benchmark_verify");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sig_buf, &pk_buf, &msg_buf, &result_buf, &params_buf],
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
        let result_r = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        let result = &result_r % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    if check {
        for i in 0..num_signatures {
            let offset = i * num_limbs * 2;
            let result_x = convert_result_coord(&results[0][
                offset..(offset + num_limbs)
            ].to_vec());
            let result_y = convert_result_coord(&results[0][
                (offset + num_limbs)..(offset + num_limbs * 2)
            ].to_vec());

            let recovered = Affine::new(result_x, result_y);
            assert_eq!(recovered, expected_pks[i]);
        }
    }

    (cpu_ms as u32, gpu_ms as u32)
}
