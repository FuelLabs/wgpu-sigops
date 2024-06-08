use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, create_ub_with_data,execute_pipeline, finish_encoder_and_read_from_gpu,
    get_device_and_queue,
};
use crate::shader::render_secp256k1_ecdsa_tests;
use crate::tests::secp256k1_curve::projective_to_affine_func;
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, PrimeField};
use ark_secp256k1::{Affine, Fq};
use fuel_crypto::{Message, SecretKey, Signature};
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, mont};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;

#[serial_test::serial]
#[tokio::test]
pub async fn secp256k1_ecrecover_benchmarks() {
    let log_limb_size = 13u32;
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(2);

    let num_signatures = 2u32.pow(16u32) as usize;
    let workgroup_size = 64;
    let num_x_workgroups = 16;
    let num_y_workgroups = 16;
    let num_z_workgroups = 4;
    assert_eq!(num_signatures, num_x_workgroups * num_y_workgroups * num_z_workgroups * workgroup_size);

    let mut signatures = Vec::with_capacity(num_signatures);
    let mut messages = Vec::with_capacity(num_signatures);
    let mut expected_pks = Vec::with_capacity(num_signatures);

    let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
    let message = Message::new(hex::encode(msg.to_bytes_be()));
    let secret = SecretKey::random(&mut rng);
    let pk = secret.public_key();
    for _ in 0..num_signatures {
        // Generate a random message
        //let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
        //let message = Message::new(hex::encode(msg.to_bytes_be()));
        //let secret = SecretKey::random(&mut rng);
        //let pk = secret.public_key();

        let pk_affine_bytes = pk.as_slice();
        let pk_x = pk_affine_bytes[0..32].to_vec();
        let pk_y = pk_affine_bytes[32..64].to_vec();
        let pk_x = BigUint::from_bytes_be(&pk_x);
        let pk_y = BigUint::from_bytes_be(&pk_y);
        let pk_x = Fq::from_be_bytes_mod_order(&pk_x.to_bytes_be());
        let pk_y = Fq::from_be_bytes_mod_order(&pk_y.to_bytes_be());
        let pk = Affine::new(pk_x, pk_y);

        let fuel_signature = Signature::sign(&secret, &message);

        signatures.push(fuel_signature);
        messages.push(message);
        expected_pks.push(pk);
    }

    // Perform signature recovery using the CPU
    let sw = Stopwatch::start_new();
    for i in 0..num_signatures {
        let _ = signatures[i]
            .recover(&messages[i])
            .expect("Failed to recover PK");
    }
    let elapsed = sw.elapsed_ms();

    println!("CPU took {}ms to recover {} secp256k1 ECDSA signatures in serial.", elapsed, num_signatures);

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
    let source = render_secp256k1_ecdsa_tests("src/wgsl/", "secp256k1_ecdsa_benchmarks.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "benchmark_secp256k1_recover");

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
    let elapsed = sw.elapsed_ms();

    println!("GPU took {}ms to recover {} secp256k1 ECDSA signatures in parallel (including data transfer cost).", elapsed, num_signatures);

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result = bigint::to_biguint_le(&data, num_limbs, log_limb_size);

        let result = &result * &rinv % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    //println!("{:?}", results);
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